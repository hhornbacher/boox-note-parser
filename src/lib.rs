use std::collections::HashMap;

use crate::{
    error::Result,
    id::{NoteUuid, VirtualPageUuid},
    note_tree::{NoteMetadata, protobuf::NoteTree},
    virtual_doc::VirtualDoc,
    virtual_page::VirtualPage,
};

mod container;
mod id;
mod json;
mod note_tree;
mod page_model;
mod shape;
mod utils;
mod virtual_doc;
mod virtual_page;

pub mod error;

pub struct NoteFile<R: std::io::Read + std::io::Seek> {
    container: container::Container<R>,
    note_metadata_map: HashMap<NoteUuid, NoteMetadata>,
}

impl<R: std::io::Read + std::io::Seek> NoteFile<R> {
    pub fn read(reader: R) -> Result<Self> {
        let mut container = container::Container::open(reader).expect("Failed to open container");

        let mut note_metadata_map = HashMap::new();

        let note_tree = if *container.container_type() == container::ContainerType::MultiNote {
            NoteTree::read(container.get_file("note_tree")?)?
        } else {
            NoteTree::read(
                container.get_file(&format!("{}/note/pb/note_info", container.root_path()))?,
            )?
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

    pub fn list_notes(&self) -> HashMap<NoteUuid, String> {
        self.note_metadata_map
            .iter()
            .map(|(id, metadata)| (*id, metadata.name.clone()))
            .collect()
    }

    pub fn get_note_metadata(&self, note_id: &NoteUuid) -> Option<&NoteMetadata> {
        self.note_metadata_map.get(note_id)
    }

    pub fn get_virtual_doc(&mut self, note_id: &NoteUuid) -> Result<VirtualDoc> {
        let note_id = note_id.to_simple_string();
        let virtual_doc = crate::virtual_doc::protobuf::VirtualDoc::read(
            self.container
                .get_file(&format!("{}/virtual/doc/pb/{}", note_id, note_id))?,
        )?;

        Ok(VirtualDoc::from_protobuf(&virtual_doc)?)
    }

    pub fn get_virtual_pages(
        &mut self,
        note_id: &NoteUuid,
    ) -> Result<HashMap<VirtualPageUuid, VirtualPage>> {
        let note_id = note_id.to_simple_string();

        let mut virtual_pages = HashMap::new();

        for page in self
            .container
            .list_directory(&format!("{}/virtual/page/pb", note_id))
        {
            let virtual_page_id = VirtualPageUuid::from_str(&page.rsplit('/').next().unwrap())?;
            let virtual_page_container = crate::virtual_page::protobuf::VirtualPageContainer::read(
                self.container.get_file(&page)?,
            )?;
            virtual_pages.insert(
                virtual_page_id,
                VirtualPage::from_protobuf(&virtual_page_container.virtual_page)?,
            );
        }

        Ok(virtual_pages)
    }
}
