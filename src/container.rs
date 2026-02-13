use std::sync::{Arc, RwLock};

use zip::ZipArchive;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerType {
    SingleNote,
    MultiNote,
}

#[derive(Debug)]
pub struct Container<R: std::io::Read + std::io::Seek> {
    container_type: Arc<ContainerType>,
    archive: Arc<RwLock<ZipArchive<R>>>,
    root_path: Arc<String>,
}

impl<R: std::io::Read + std::io::Seek> Container<R> {
    pub fn open(reader: R) -> Result<Self> {
        let archive = ZipArchive::new(reader).map_err(Error::Zip)?;
        let file_names: Vec<String> = archive.file_names().map(str::to_string).collect();

        if file_names.is_empty() {
            return Err(Error::InvalidContainerFormat);
        }

        let (container_type, root_path) = if let Some(root_path) =
            Self::detect_root_path(&file_names, "note_tree")
        {
            (ContainerType::MultiNote, root_path)
        } else if let Some(root_path) = Self::detect_root_path(&file_names, "note/pb/note_info") {
            (ContainerType::SingleNote, root_path)
        } else {
            return Err(Error::InvalidContainerFormat);
        };

        Ok(Self {
            container_type: Arc::new(container_type),
            archive: Arc::new(RwLock::new(archive)),
            root_path: Arc::new(root_path),
        })
    }

    pub fn container_type(&self) -> &ContainerType {
        &self.container_type
    }

    fn detect_root_path(file_names: &[String], marker: &str) -> Option<String> {
        let suffix = format!("/{}", marker);
        file_names.iter().find_map(|name| {
            if name == marker {
                Some(String::new())
            } else {
                name.strip_suffix(&suffix)
                    .map(|prefix| prefix.trim_end_matches('/').to_string())
            }
        })
    }

    fn get_file_path(&self, path: &str) -> String {
        let normalized_path = path.trim_start_matches('/');

        if self.root_path.is_empty() {
            return normalized_path.to_string();
        }
        if normalized_path.is_empty() {
            return self.root_path.to_string();
        }
        format!("{}/{}", self.root_path, normalized_path)
    }

    pub fn list_directory(&self, path: &str) -> Vec<String> {
        let normalized_path = path.trim_start_matches('/');
        let prefixed_path = self.get_file_path(normalized_path);
        let enforce_segment_boundary =
            !normalized_path.ends_with('#') && !normalized_path.ends_with('/');

        self.archive
            .read()
            .unwrap()
            .file_names()
            .filter_map(|name| {
                if name.ends_with('/') || !name.starts_with(&prefixed_path) {
                    return None;
                }

                if !enforce_segment_boundary
                    || name.len() == prefixed_path.len()
                    || name.as_bytes().get(prefixed_path.len()) == Some(&b'/')
                {
                    return Some(name.to_string());
                }

                None
            })
            .collect()
    }

    pub fn get_file_relative<T, F>(&mut self, path: &str, file_op_fn: F) -> Result<T>
    where
        F: FnOnce(zip::read::ZipFile<'_, R>) -> Result<T>,
    {
        let file_path = self.get_file_path(path);
        let mut archive = self.archive.write().unwrap();
        let file = archive.by_name(&file_path).map_err(Error::Zip)?;
        file_op_fn(file)
    }

    pub fn get_file_absolute<T, F>(&mut self, path: &str, file_op_fn: F) -> Result<T>
    where
        F: FnOnce(zip::read::ZipFile<'_, R>) -> Result<T>,
    {
        let mut archive = self.archive.write().unwrap();
        let file = archive.by_name(path).map_err(Error::Zip)?;
        file_op_fn(file)
    }

    pub fn root_path(&self) -> &str {
        &self.root_path
    }
}

impl<R: std::io::Read + std::io::Seek> Clone for Container<R> {
    fn clone(&self) -> Self {
        Self {
            container_type: self.container_type.clone(),
            archive: self.archive.clone(),
            root_path: self.root_path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    use super::{Container, ContainerType};

    fn build_archive(file_names: &[&str]) -> Cursor<Vec<u8>> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        for file_name in file_names {
            writer.start_file(file_name, options).unwrap();
            writer.write_all(b"x").unwrap();
        }

        writer.finish().unwrap()
    }

    #[test]
    fn detects_rootless_multi_note_container() {
        let reader = build_archive(&["note_tree"]);
        let container = Container::open(reader).unwrap();
        assert_eq!(*container.container_type(), ContainerType::MultiNote);
        assert_eq!(container.root_path(), "");
    }

    #[test]
    fn detects_rooted_multi_note_container() {
        let reader = build_archive(&["backup/note_tree"]);
        let container = Container::open(reader).unwrap();
        assert_eq!(*container.container_type(), ContainerType::MultiNote);
        assert_eq!(container.root_path(), "backup");
    }

    #[test]
    fn detects_rootless_single_note_container() {
        let reader = build_archive(&["note/pb/note_info"]);
        let container = Container::open(reader).unwrap();
        assert_eq!(*container.container_type(), ContainerType::SingleNote);
        assert_eq!(container.root_path(), "");
    }

    #[test]
    fn detects_rooted_single_note_container() {
        let reader = build_archive(&["7a960ca753b0420ea2d5b88d57f7bf62/note/pb/note_info"]);
        let container = Container::open(reader).unwrap();
        assert_eq!(*container.container_type(), ContainerType::SingleNote);
        assert_eq!(container.root_path(), "7a960ca753b0420ea2d5b88d57f7bf62");
    }

    #[test]
    fn list_directory_enforces_segment_boundary_for_directory_paths() {
        let reader = build_archive(&[
            "backup/note_tree",
            "backup/dir/a/file1",
            "backup/dir/ab/file2",
        ]);
        let container = Container::open(reader).unwrap();

        let files = container.list_directory("dir/a");
        assert_eq!(files, vec!["backup/dir/a/file1".to_string()]);
    }

    #[test]
    fn list_directory_keeps_prefix_behavior_for_shape_and_point_prefixes() {
        let reader = build_archive(&[
            "backup/note_tree",
            "backup/note/shape/page#group#1.zip",
            "backup/note/shape/page-other#group#1.zip",
            "backup/note/point/page/page#points#points",
            "backup/note/point/page/page-other#points#points",
        ]);
        let container = Container::open(reader).unwrap();

        let shape_files = container.list_directory("note/shape/page#");
        assert_eq!(
            shape_files,
            vec!["backup/note/shape/page#group#1.zip".to_string()]
        );

        let point_files = container.list_directory("note/point/page/page#");
        assert_eq!(
            point_files,
            vec!["backup/note/point/page/page#points#points".to_string()]
        );
    }
}
