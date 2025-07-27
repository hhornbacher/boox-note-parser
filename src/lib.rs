use std::collections::HashMap;

use crate::{
    error::Result,
    id::NoteId,
    note_tree::{NoteMetadata, protobuf::NoteTree},
};

mod container;
mod id;
mod note_tree;
mod utils;
mod virtual_doc;
mod virtual_page;
mod shape;
mod page_model;
mod json;

pub mod error;

pub struct NoteFile<R: std::io::Read + std::io::Seek> {
    container: container::Container<R>,
    note_metadata_map: HashMap<NoteId, NoteMetadata>,
}

impl<R: std::io::Read + std::io::Seek> NoteFile<R> {
    pub fn read(reader: R) -> Result<Self> {
        let mut container = container::Container::open(reader).expect("Failed to open container");

        let mut note_metadata_map = HashMap::new();

        let note_tree = if *container.container_type() == container::ContainerType::MultiNote {
            NoteTree::read(container.get_file("note_tree")?)?
        } else {
            NoteTree::read(container.get_file("note/pb/note_info")?)?
        };

        for note in note_tree.notes {
            let note_metadata = NoteMetadata::from_protobuf(&note)?;
            note_metadata_map.insert(note_metadata.note_id, note_metadata);
        }

        Ok(Self {
            container,
            note_metadata_map,
        })
    }

    pub fn container_type(&self) -> &container::ContainerType {
        self.container.container_type()
    }

    pub fn list_notes(&self) -> HashMap<NoteId, String> {
        self.note_metadata_map.iter().map(|(id, metadata)| (*id, metadata.name.clone())).collect()
    }

    pub fn get_note_metadata(&self, note_id: &NoteId) -> Option<&NoteMetadata> {
        self.note_metadata_map.get(note_id)
    }
}
