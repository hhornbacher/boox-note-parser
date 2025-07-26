use std::path::Path;

use zip::{ZipArchive, read::ZipFile};

use crate::error::{Error, Result};

pub enum ContainerType {
    SingleNote,
    MultiNote,
}

pub struct Container<R: std::io::Read + std::io::Seek> {
    container_type: ContainerType,
    archive: ZipArchive<R>,
    root_path: String,
}

impl<R: std::io::Read + std::io::Seek> Container<R> {
    pub fn open(file: R) -> Result<Self> {
        let mut archive = ZipArchive::new(file).expect("Failed to open zip archive");

        let first_file = archive.by_index(0)?.name().to_string();
        let first_file_path = Path::new(&first_file);
        let root_path = first_file_path
            .iter()
            .next()
            .and_then(|s| s.to_str())
            .ok_or(Error::InvalidContainerFormat)?
            .to_string();

        let container_type = if archive.by_name(&format!("{}/note_tree", root_path)).is_ok() {
            ContainerType::MultiNote
        } else {
            ContainerType::SingleNote
        };

        Ok(Self {
            container_type,
            archive,
            root_path,
        })
    }

    pub fn container_type(&self) -> &ContainerType {
        &self.container_type
    }

    fn get_file_path(&self, path: &str) -> String {
        format!("{}/{}", self.root_path, path)
    }

    pub fn list_directory(&self, path: &str) -> Vec<String> {
        let prefixed_path = self.get_file_path(path);
        self.archive
            .file_names()
            .filter_map(|name| {
                if name.starts_with(&prefixed_path) && !name.ends_with("/") {
                    name.to_string().split('/').next().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_file(&mut self, path: &str) -> Result<ZipFile<R>> {
        let file_path = self.get_file_path(path);
        self.archive.by_name(&file_path).map_err(Error::Zip)
    }
}
