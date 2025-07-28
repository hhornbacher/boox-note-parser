use std::collections::HashMap;

use crate::{
    error::Result,
    id::{NoteUuid, PageModelUuid, ShapeGroupUuid, VirtualPageUuid},
    note_tree::{NoteMetadata, NoteTree},
    page_model::PageModelGroup,
    shape::ShapeGroup,
    virtual_doc::VirtualDoc,
    virtual_page::VirtualPage,
};

mod container;
mod json;
mod note_tree;
mod page_model;
mod utils;
mod virtual_doc;

pub mod error;
pub mod id;
pub mod points;
pub mod shape;
pub mod virtual_page;

pub struct NoteFile<R: std::io::Read + std::io::Seek> {
    container: container::Container<R>,
    note_tree: NoteTree,
}

impl<R: std::io::Read + std::io::Seek> NoteFile<R> {
    pub fn read(reader: R) -> Result<Self> {
        let mut container = container::Container::open(reader).expect("Failed to open container");

        let note_tree = if *container.container_type() == container::ContainerType::MultiNote {
            container.get_file_relative("note_tree", |reader| NoteTree::read(reader))?
        } else {
            container.get_file_relative(
                &format!("{}/note/pb/note_info", container.root_path()),
                |reader| NoteTree::read(reader),
            )?
        };

        Ok(Self {
            container,
            note_tree,
        })
    }

    pub fn list_notes(&self) -> HashMap<NoteUuid, String> {
        self.note_tree
            .notes
            .iter()
            .map(|(id, metadata)| (*id, metadata.name.clone()))
            .collect()
    }

    pub fn get_note(&self, note_id: &NoteUuid) -> Option<Note<R>> {
        self.note_tree
            .notes
            .get(note_id)
            .map(|metadata| Note::new(self.container.clone(), metadata.clone()))
    }

    // pub fn get_shapes(
    //     &mut self,
    //     note_id: &NoteUuid,
    //     page_id: &PageUuid,
    // ) -> Result<HashMap<ShapeGroupUuid, Vec<Shape>>> {
    //     let note_id = note_id.to_simple_string();
    //     let page_id = page_id.to_simple_string();

    //     let mut shapes = HashMap::new();

    //     for shape_path in self
    //         .container
    //         .list_directory(&format!("{}/shape/{}#", note_id, page_id))
    //     {
    //         let path_tail = shape_path.rsplit('/').next().unwrap();
    //         let parts = path_tail.split('#').collect::<Vec<_>>();
    //         let shape_group_id = ShapeGroupUuid::from_str(parts[1])?;
    //         let _timestamp = convert_timestamp_to_datetime(
    //             parts[2].replace(".zip", "").parse::<u64>().map_err(|e| {
    //                 Error::InvalidTimestampFormat(format!("Failed to parse timestamp: {}", e))
    //             })?,
    //         );

    //         let mut buffer = Vec::new();
    //         {
    //             let mut shape_container = self.container.get_file_absolute(&shape_path)?;
    //             shape_container.read_to_end(&mut buffer)?;
    //         }
    //         let buffer_cursor = std::io::Cursor::new(buffer);
    //         let mut shape_archive = zip::ZipArchive::new(buffer_cursor).map_err(Error::Zip)?;

    //         let shape_file = shape_archive.by_index(0).map_err(Error::Zip)?;

    //         let shape_container = shape::protobuf::ShapeContainer::read(shape_file)?;
    //         shapes.insert(shape_group_id, Vec::new());

    //         for shape in shape_container.shapes {
    //             let shape = shape::Shape::from_protobuf(shape)?;
    //             shapes.get_mut(&shape_group_id).unwrap().push(shape);
    //         }
    //     }

    //     Ok(shapes)
    // }

    // pub fn get_points_files(
    //     &mut self,
    //     note_id: &NoteUuid,
    //     page_id: &PageUuid,
    // ) -> Result<HashMap<PointsUuid, Vec<points::PointsFile>>> {
    //     let note_id = note_id.to_simple_string();
    //     let page_id = page_id.to_simple_string();

    //     let mut points_files = HashMap::new();

    //     for stroke_path in self
    //         .container
    //         .list_directory(&format!("{}/point/{}/{}#", note_id, page_id, page_id))
    //     {
    //         let path_tail = stroke_path.rsplit('/').next().unwrap();
    //         let parts = path_tail.split('#').collect::<Vec<_>>();

    //         let shape_id = PointsUuid::from_str(parts[1])?;

    //         let mut buffer = Vec::new();
    //         {
    //             let mut stroke_container = self.container.get_file_absolute(&stroke_path)?;
    //             stroke_container.read_to_end(&mut buffer)?;
    //         }
    //         let buffer_cursor = std::io::Cursor::new(buffer);

    //         let points_file = PointsFile::read(buffer_cursor)?;

    //         points_files
    //             .entry(shape_id)
    //             .or_insert_with(Vec::new)
    //             .push(points_file);
    //     }

    //     Ok(points_files)
    // }
}

impl<R: std::io::Read + std::io::Seek> std::fmt::Debug for NoteFile<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoteFile")
            .field("container_type", &self.container.container_type())
            .field("note_tree", &self.note_tree)
            .finish()
    }
}

pub struct Note<R: std::io::Read + std::io::Seek> {
    container: container::Container<R>,
    metadata: NoteMetadata,
    virtual_doc: Option<VirtualDoc>,
    virtual_pages: Option<HashMap<VirtualPageUuid, VirtualPage>>,
    page_models: Option<HashMap<PageModelUuid, PageModelGroup>>,
    shape_groups: Option<HashMap<ShapeGroupUuid, ShapeGroup>>,
}

impl<R: std::io::Read + std::io::Seek> Note<R> {
    fn new(container: container::Container<R>, metadata: NoteMetadata) -> Self {
        Self {
            container,
            metadata,
            virtual_doc: None,
            virtual_pages: None,
            page_models: None,
            shape_groups: None,
        }
    }

    pub fn virtual_doc(&mut self) -> Result<&VirtualDoc> {
        if self.virtual_doc.is_none() {
            let note_id = self.metadata.note_id.to_simple_string();
            let virtual_doc = self.container.get_file_relative(
                &format!("{}/virtual/doc/pb/{}", note_id, note_id),
                |reader| VirtualDoc::read(reader),
            )?;
            self.virtual_doc = Some(virtual_doc);
        }
        Ok(self.virtual_doc.as_ref().unwrap())
    }

    pub fn virtual_pages(&mut self) -> Result<&HashMap<VirtualPageUuid, VirtualPage>> {
        if self.virtual_pages.is_none() {
            let note_id = self.metadata.note_id.to_simple_string();

            let mut virtual_pages = HashMap::new();

            for virtual_page_path in self
                .container
                .list_directory(&format!("{}/virtual/page/pb", note_id))
            {
                let virtual_page_id =
                    VirtualPageUuid::from_str(&virtual_page_path.rsplit('/').next().unwrap())?;
                let virtual_page = self
                    .container
                    .get_file_absolute(&virtual_page_path, |reader| VirtualPage::read(reader))?;
                virtual_pages.insert(virtual_page_id, virtual_page);
            }
            self.virtual_pages = Some(virtual_pages);
        }
        Ok(self.virtual_pages.as_ref().unwrap())
    }

    pub fn page_models(&mut self) -> Result<&HashMap<PageModelUuid, PageModelGroup>> {
        if self.page_models.is_none() {
            let note_id = self.metadata.note_id.to_simple_string();

            let mut page_models = HashMap::new();

            for page_model_path in self
                .container
                .list_directory(&format!("{}/pageModel/pb", note_id))
            {
                let page_model_id =
                    PageModelUuid::from_str(&page_model_path.rsplit('/').next().unwrap())?;
                let page_model = self
                    .container
                    .get_file_absolute(&page_model_path, |reader| PageModelGroup::read(reader))?;
                page_models.insert(page_model_id, page_model);
            }
            self.page_models = Some(page_models);
        }
        Ok(self.page_models.as_ref().unwrap())
    }

    // pub fn shape_groups(&mut self) -> Result<&HashMap<ShapeGroupUuid, ShapeGroup>> {
    //     if self.shape_groups.is_none() {
    //         let note_id = self.metadata.note_id.to_simple_string();

    //         let mut shape_groups = HashMap::new();

    //         for shape_group_path in self
    //             .container
    //             .list_directory(&format!("{}/shape/{}#", note_id, page_id))
    //         {
    //             let path_tail = shape_group_path.rsplit('/').next().unwrap();
    //             let parts = path_tail.split('#').collect::<Vec<_>>();
    //             let shape_group_id = ShapeGroupUuid::from_str(parts[1])?;
    //             let _timestamp = convert_timestamp_to_datetime(
    //                 parts[2].replace(".zip", "").parse::<u64>().map_err(|e| {
    //                     Error::InvalidTimestampFormat(format!("Failed to parse timestamp: {}", e))
    //                 })?,
    //             );
    //             let shape_group_id =
    //                 ShapeGroupUuid::from_str(&shape_group_path.rsplit('/').next().unwrap())?;
    //             let shape_group = self
    //                 .container
    //                 .get_file_absolute(&shape_group_path, |reader| ShapeGroup::read(reader))?;
    //             shape_groups.insert(shape_group_id, shape_group);
    //         }
    //         self.shape_groups = Some(shape_groups);
    //     }
    //     Ok(self.shape_groups.as_ref().unwrap())
    // }
}

impl<R: std::io::Read + std::io::Seek> std::fmt::Debug for Note<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Note")
            .field("metadata", &self.metadata)
            .field("virtual_doc", &self.virtual_doc)
            .field("virtual_pages", &self.virtual_pages)
            .finish()
    }
}
