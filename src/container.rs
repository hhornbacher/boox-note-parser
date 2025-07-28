use std::{
    path::Path,
    sync::{Arc, RwLock},
};

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
        let mut archive = ZipArchive::new(reader).expect("Failed to open zip archive");

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
            container_type: Arc::new(container_type),
            archive: Arc::new(RwLock::new(archive)),
            root_path: Arc::new(root_path),
        })
    }

    pub fn container_type(&self) -> &ContainerType {
        &self.container_type
    }

    fn get_file_path(&self, path: &str) -> String {
        if self.container_type.as_ref() == &ContainerType::SingleNote {
            return path.to_string();
        }
        format!("{}/{}", self.root_path, path)
    }

    pub fn list_directory(&self, path: &str) -> Vec<String> {
        let prefixed_path = self.get_file_path(path);
        self.archive
            .read()
            .unwrap()
            .file_names()
            .filter_map(|name| {
                if name.starts_with(&prefixed_path) && !name.ends_with("/") {
                    Some(name.to_string())
                } else {
                    None
                }
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
