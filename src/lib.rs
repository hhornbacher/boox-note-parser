use std::{collections::HashMap, io::Read};

use crate::{
    error::{Error, Result},
    id::{NoteUuid, PageModelUuid, PageUuid, ShapeUuid, VirtualPageUuid},
    note_tree::{NoteMetadata, protobuf::NoteTree},
    shape::Shape,
    utils::convert_timestamp_to_datetime,
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
            NoteTree::read(container.get_file_relative("note_tree")?)?
        } else {
            NoteTree::read(
                container
                    .get_file_relative(&format!("{}/note/pb/note_info", container.root_path()))?,
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
                .get_file_relative(&format!("{}/virtual/doc/pb/{}", note_id, note_id))?,
        )?;

        Ok(VirtualDoc::from_protobuf(&virtual_doc)?)
    }

    pub fn get_virtual_pages(
        &mut self,
        note_id: &NoteUuid,
    ) -> Result<HashMap<VirtualPageUuid, VirtualPage>> {
        let note_id = note_id.to_simple_string();

        let mut virtual_pages = HashMap::new();

        for virtual_page_path in self
            .container
            .list_directory(&format!("{}/virtual/page/pb", note_id))
        {
            let virtual_page_id =
                VirtualPageUuid::from_str(&virtual_page_path.rsplit('/').next().unwrap())?;
            let virtual_page_container = crate::virtual_page::protobuf::VirtualPageContainer::read(
                self.container.get_file_absolute(&virtual_page_path)?,
            )?;
            virtual_pages.insert(
                virtual_page_id,
                VirtualPage::from_protobuf(&virtual_page_container.virtual_page)?,
            );
        }

        Ok(virtual_pages)
    }

    pub fn get_page_models(
        &mut self,
        note_id: &NoteUuid,
    ) -> Result<HashMap<PageModelUuid, page_model::PageModel>> {
        let note_id = note_id.to_simple_string();

        let mut page_models = HashMap::new();

        for page_model_path in self
            .container
            .list_directory(&format!("{}/pageModel/pb", note_id))
        {
            let page_model_id =
                PageModelUuid::from_str(&page_model_path.rsplit('/').next().unwrap())?;
            let page_model_container = page_model::protobuf::PageModelContainer::read(
                self.container.get_file_absolute(&page_model_path)?,
            )?;
            page_models.insert(
                page_model_id,
                page_model::PageModel::from_protobuf(&page_model_container.page_model)?,
            );
        }

        Ok(page_models)
    }

    pub fn get_shapes(
        &mut self,
        note_id: &NoteUuid,
        page_id: &PageUuid,
    ) -> Result<HashMap<ShapeUuid, Vec<Shape>>> {
        let note_id = note_id.to_simple_string();
        let page_id = page_id.to_simple_string();

        let mut shapes = HashMap::new();

        for shape_path in self
            .container
            .list_directory(&format!("{}/shape/{}#", note_id, page_id))
        {
            let path_tail = shape_path.rsplit('/').next().unwrap();
            let parts = path_tail.split('#').collect::<Vec<_>>();
            let shape_id = ShapeUuid::from_str(parts[1])?;
            let _timestamp = convert_timestamp_to_datetime(
                parts[2].replace(".zip", "").parse::<u64>().map_err(|e| {
                    Error::InvalidTimestampFormat(format!("Failed to parse timestamp: {}", e))
                })?,
            );

            let mut buffer = Vec::new();
            {
                let mut shape_container = self.container.get_file_absolute(&shape_path)?;
                shape_container.read_to_end(&mut buffer)?;
            }
            let buffer_cursor = std::io::Cursor::new(buffer);
            let mut shape_archive = zip::ZipArchive::new(buffer_cursor).map_err(Error::Zip)?;

            let shape_file = shape_archive.by_index(0).map_err(Error::Zip)?;

            let shape_container = shape::protobuf::ShapeContainer::read(shape_file)?;
            shapes.insert(shape_id, Vec::new());

            for shape in shape_container.shapes {
                let shape = shape::Shape::from_protobuf(shape)?;
                shapes.get_mut(&shape_id).unwrap().push(shape);
            }
        }

        Ok(shapes)
    }
}
