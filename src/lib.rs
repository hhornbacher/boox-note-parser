use std::{collections::HashMap, io::Read};

use raqote::{DrawOptions, DrawTarget, Source, StrokeStyle};

use crate::{
    error::{Error, Result},
    id::{NoteUuid, PageModelUuid, PageUuid, PointsUuid, ShapeGroupUuid, VirtualPageUuid},
    note_tree::{NoteMetadata, NoteTree},
    page_model::{PageModel, PageModelGroup},
    shape::ShapeGroup,
    utils::convert_timestamp_to_datetime,
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
}

impl<R: std::io::Read + std::io::Seek> Note<R> {
    fn new(container: container::Container<R>, metadata: NoteMetadata) -> Self {
        Self {
            container,
            metadata,
            virtual_doc: None,
            virtual_pages: None,
            page_models: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    pub fn active_pages(&self) -> &[PageUuid] {
        &self.metadata.active_pages
    }

    pub fn reserved_pages(&self) -> &[PageUuid] {
        &self.metadata.reserved_pages
    }

    pub fn detached_pages(&self) -> &[PageUuid] {
        &self.metadata.detached_pages
    }

    pub fn created(&self) -> chrono::DateTime<chrono::Utc> {
        self.metadata.created
    }

    pub fn modified(&self) -> chrono::DateTime<chrono::Utc> {
        self.metadata.modified
    }

    pub fn flag(&self) -> u32 {
        self.metadata.flag
    }

    pub fn pen_width(&self) -> f32 {
        self.metadata.pen_width
    }

    pub fn scale_factor(&self) -> f32 {
        self.metadata.scale_factor
    }

    pub fn fill_color(&self) -> &u32 {
        &self.metadata.fill_color
    }

    pub fn pen_type(&self) -> &u32 {
        &self.metadata.pen_type
    }

    pub fn pen_settings_fill_color(&self) -> &u32 {
        &self.metadata.pen_settings.fill_color
    }

    pub fn pen_settings_graphics_shape_color(&self) -> &u32 {
        &self.metadata.pen_settings.graphics_shape_color
    }

    pub fn get_page(&mut self, page_id: &PageUuid) -> Option<Page<R>> {
        let virtual_page = {
            let virtual_pages = self
                .virtual_pages()
                .inspect_err(|_| {
                    log::error!("Failed to get virtual pages for page ID: {}", page_id);
                })
                .ok()?;

            virtual_pages
                .values()
                .find(|vp| &vp.page_id == page_id)
                .cloned()
        };

        let page_model = {
            let page_models = self
                .page_models()
                .inspect_err(|_| {
                    log::error!("Failed to get page models for page ID: {}", page_id);
                })
                .ok()?;

            page_models
                .values()
                .find_map(|pm| pm.page_models.iter().find(|p| p.page_id == *page_id))?
                .clone()
        };

        Some(Page::new(
            self.container.clone(),
            page_id.clone(),
            self.metadata.note_id.clone(),
            virtual_page,
            page_model,
        ))
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
}

impl<R: std::io::Read + std::io::Seek> std::fmt::Debug for Note<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Note")
            .field("metadata", &self.metadata)
            .finish()
    }
}

pub struct Page<R: std::io::Read + std::io::Seek> {
    container: container::Container<R>,
    note_id: NoteUuid,
    page_id: PageUuid,
    virtual_page: Option<VirtualPage>,
    page_model: PageModel,
    shape_groups: Option<HashMap<ShapeGroupUuid, ShapeGroup>>,
    points_files: Option<HashMap<PointsUuid, Vec<points::PointsFile>>>,
}

impl<R: std::io::Read + std::io::Seek> Page<R> {
    fn new(
        container: container::Container<R>,
        page_id: PageUuid,
        note_id: NoteUuid,
        virtual_page: Option<VirtualPage>,
        page_model: PageModel,
    ) -> Self {
        Self {
            container,
            page_id,
            note_id,
            virtual_page,
            page_model,
            shape_groups: None,
            points_files: None,
        }
    }

    pub fn virtual_page(&self) -> &Option<VirtualPage> {
        &self.virtual_page
    }

    pub fn page_model(&self) -> &PageModel {
        &self.page_model
    }

    pub fn shape_groups(&mut self) -> Result<&HashMap<ShapeGroupUuid, ShapeGroup>> {
        if self.shape_groups.is_none() {
            let note_id = self.note_id.to_simple_string();
            let page_id = self.page_id.to_simple_string();

            let mut shape_groups = HashMap::new();

            for shape_group_path in self
                .container
                .list_directory(&format!("{}/shape/{}#", note_id, page_id))
            {
                let path_tail = shape_group_path.rsplit('/').next().unwrap();
                let parts = path_tail.split('#').collect::<Vec<_>>();
                let shape_group_id = ShapeGroupUuid::from_str(parts[1])?;
                let _timestamp = convert_timestamp_to_datetime(
                    parts[2].replace(".zip", "").parse::<u64>().map_err(|e| {
                        Error::InvalidTimestampFormat(format!("Failed to parse timestamp: {}", e))
                    })?,
                );
                let shape_group = self
                    .container
                    .get_file_absolute(&shape_group_path, |reader| ShapeGroup::read(reader))?;
                shape_groups.insert(shape_group_id, shape_group);
            }
            self.shape_groups = Some(shape_groups);
        }
        Ok(self.shape_groups.as_ref().unwrap())
    }

    pub fn points_files(&mut self) -> Result<&HashMap<PointsUuid, Vec<points::PointsFile>>> {
        if self.points_files.is_none() {
            let note_id = self.note_id.to_simple_string();
            let page_id = self.page_id.to_simple_string();

            let mut points_files = HashMap::new();

            for stroke_path in self
                .container
                .list_directory(&format!("{}/point/{}/{}#", note_id, page_id, page_id))
            {
                let path_tail = stroke_path.rsplit('/').next().unwrap();
                let parts = path_tail.split('#').collect::<Vec<_>>();
                let shape_id = PointsUuid::from_str(parts[1])?;

                let file_data = self
                    .container
                    .get_file_absolute(&stroke_path, |mut reader| {
                        let mut buffer = Vec::new();
                        reader.read_to_end(&mut buffer).map_err(Error::Io)?;
                        Ok(buffer)
                    })?;

                let buffer_cursor = std::io::Cursor::new(file_data);
                let points_file = points::PointsFile::read(buffer_cursor)?;

                points_files
                    .entry(shape_id)
                    .or_insert_with(Vec::new)
                    .push(points_file);
            }
            self.points_files = Some(points_files);
        }
        Ok(self.points_files.as_ref().unwrap())
    }

    pub fn render(&mut self) -> Result<DrawTarget> {
        let page_id = self.page_id.to_hyphenated_string();
        let width = self.page_model.dimensions.right - self.page_model.dimensions.left;
        let height = self.page_model.dimensions.bottom - self.page_model.dimensions.top;
        let mut draw_target = DrawTarget::new(width as i32, height as i32);
        let draw_options = DrawOptions::new();

        draw_target.fill_rect(
            0.0,
            0.0,
            width,
            height,
            &Source::Solid(raqote::Color::new(255, 255, 255, 255).into()),
            &DrawOptions::new(),
        );

        // Extract shape_groups and points_files into local variables to avoid multiple mutable borrows.
        let shape_groups = {
            let sg = self.shape_groups().inspect_err(|_| {
                log::error!("Failed to get shape groups for page ID: {}", page_id)
            })?;
            sg.clone()
        };

        let points_files_vec = {
            let pf = self.points_files().inspect_err(|_| {
                log::error!("Failed to get points files for page ID: {}", page_id)
            })?;
            pf.values().flatten().collect::<Vec<_>>()
        };

        for (shape_group_id, shape_group) in &shape_groups {
            let mut shapes = shape_group.shapes().to_vec();
            shapes.sort_by(|a, b| a.z_order.cmp(&b.z_order));

            for shape in shapes {
                if let Some(points_id) = shape.points_id {
                    if let Some(points) = points_files_vec
                        .iter()
                        .find(|pf| pf.header().points_id == points_id)
                    {
                        points
                            .get_stroke(&shape.stroke_id)
                            .ok_or_else(|| {
                                log::error!("Failed to get stroke for shape");
                                Error::StrokeNotFound
                            })
                            .and_then(|stroke| {
                                log::debug!("Rendering stroke for shape");
                                log::debug!(
                                    "Shape Group ID: {}, Stroke ID: {}",
                                    shape_group_id.to_hyphenated_string(),
                                    shape.stroke_id.to_hyphenated_string()
                                );
                                log::debug!("Shape: {:#x?}", shape);
                                stroke.render(
                                    &mut draw_target,
                                    &draw_options,
                                    &StrokeStyle::default(),
                                )
                            })?;
                    } else {
                        log::warn!(
                            "No points files found for shape group: {}",
                            shape_group_id.to_hyphenated_string()
                        );
                    }
                }
            }
        }

        Ok(draw_target)
    }
}

impl<R: std::io::Read + std::io::Seek> std::fmt::Debug for Page<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Page")
            .field("virtual_page", &self.virtual_page)
            .field("page_model", &self.page_model)
            .finish()
    }
}
