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

pub mod error;

pub struct NoteFile<R: std::io::Read + std::io::Seek> {
    container: container::Container<R>,
    note_name_map: HashMap<NoteId, String>,
}

impl<R: std::io::Read + std::io::Seek> NoteFile<R> {
    pub fn read(reader: R) -> Result<Self> {
        let mut container = container::Container::open(reader).expect("Failed to open container");

        let mut note_name_map = HashMap::new();


            let note_tree = if *container.container_type() == container::ContainerType::MultiNote {
                NoteTree::read(container.get_file("note_tree")?)?
            } else {
                NoteTree::read(container.get_file("note/pb/note_info")?)?
            };

            for note in note_tree.notes {
                let note_metadata = NoteMetadata::from_protobuf(&note)?;
                note_name_map.insert(note_metadata.note_id, note_metadata.name);
            }
        

        println!("Note name map: {:#?}", note_name_map);

        Ok(Self {
            container,
            note_name_map,
        })
    }

    pub fn container_type(&self) -> &container::ContainerType {
        self.container.container_type()
    }
}
