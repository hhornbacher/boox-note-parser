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
pub mod extra;
pub mod id;
pub mod note_assets;
pub mod points;
pub mod resource;
pub mod shape;
pub mod template;
pub mod virtual_page;

pub struct NoteFile<R: std::io::Read + std::io::Seek> {
    container: container::Container<R>,
    note_tree: NoteTree,
}

impl<R: std::io::Read + std::io::Seek> NoteFile<R> {
    pub fn read(reader: R) -> Result<Self> {
        let mut container = container::Container::open(reader)?;

        let note_tree = if *container.container_type() == container::ContainerType::MultiNote {
            container.get_file_relative("note_tree", |reader| NoteTree::read(reader))?
        } else {
            container.get_file_relative("note/pb/note_info", |reader| NoteTree::read(reader))?
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
        self.note_tree.notes.get(note_id).map(|metadata| {
            let note_path_prefix =
                if *self.container.container_type() == container::ContainerType::MultiNote {
                    metadata.note_id.to_simple_string()
                } else {
                    String::new()
                };
            Note::new(self.container.clone(), metadata.clone(), note_path_prefix)
        })
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
    note_path_prefix: String,
    virtual_doc: Option<VirtualDoc>,
    virtual_pages: Option<HashMap<VirtualPageUuid, VirtualPage>>,
    page_models: Option<HashMap<PageModelUuid, PageModelGroup>>,
    extra_metadata: Option<Option<extra::ExtraMetadata>>,
    templates: Option<template::TemplateSet>,
    resources: Option<Vec<resource::ResourceRecord>>,
    assets_metadata: Option<note_assets::NoteAssetsMetadata>,
}

impl<R: std::io::Read + std::io::Seek> Note<R> {
    fn new(
        container: container::Container<R>,
        metadata: NoteMetadata,
        note_path_prefix: String,
    ) -> Self {
        Self {
            container,
            metadata,
            note_path_prefix,
            virtual_doc: None,
            virtual_pages: None,
            page_models: None,
            extra_metadata: None,
            templates: None,
            resources: None,
            assets_metadata: None,
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
            self.metadata.note_id,
            page_id.clone(),
            self.note_path_prefix.clone(),
            virtual_page,
            page_model,
        ))
    }

    pub fn virtual_doc(&mut self) -> Result<&VirtualDoc> {
        if self.virtual_doc.is_none() {
            let note_id = self.metadata.note_id.to_simple_string();
            let virtual_doc_path = join_archive_path(
                &self.note_path_prefix,
                &format!("virtual/doc/pb/{}", note_id),
            );
            let virtual_doc = self
                .container
                .get_file_relative(&virtual_doc_path, |reader| VirtualDoc::read(reader))?;
            self.virtual_doc = Some(virtual_doc);
        }
        Ok(self.virtual_doc.as_ref().unwrap())
    }

    pub fn virtual_pages(&mut self) -> Result<&HashMap<VirtualPageUuid, VirtualPage>> {
        if self.virtual_pages.is_none() {
            let mut virtual_pages = HashMap::new();

            for virtual_page_path in self.container.list_directory(&join_archive_path(
                &self.note_path_prefix,
                "virtual/page/pb",
            )) {
                let virtual_page_id =
                    VirtualPageUuid::from_str(file_name_from_path(&virtual_page_path)?)?;
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
            let mut page_models = HashMap::new();

            for page_model_path in self
                .container
                .list_directory(&join_archive_path(&self.note_path_prefix, "pageModel/pb"))
            {
                let page_model_id =
                    PageModelUuid::from_str(file_name_from_path(&page_model_path)?)?;
                let page_model = self
                    .container
                    .get_file_absolute(&page_model_path, |reader| PageModelGroup::read(reader))?;
                page_models.insert(page_model_id, page_model);
            }
            self.page_models = Some(page_models);
        }
        Ok(self.page_models.as_ref().unwrap())
    }

    pub fn extra_metadata(&mut self) -> Result<Option<&extra::ExtraMetadata>> {
        if self.extra_metadata.is_none() {
            let extra_path = join_archive_path(&self.note_path_prefix, "extra/pb/extra");
            let parsed_extra =
                if self.container.has_entry_relative(&extra_path) {
                    Some(self.container.get_file_relative(&extra_path, |reader| {
                        extra::ExtraMetadata::read(reader)
                    })?)
                } else {
                    None
                };
            self.extra_metadata = Some(parsed_extra);
        }

        Ok(self.extra_metadata.as_ref().unwrap().as_ref())
    }

    pub fn templates(&mut self) -> Result<&template::TemplateSet> {
        if self.templates.is_none() {
            let mut templates = template::TemplateSet::new();

            let primary_templates_path = join_archive_path(&self.note_path_prefix, "template/json");
            self.load_templates_from_directory(&primary_templates_path, &mut templates)?;

            let document_templates_path = join_archive_path(
                &self.note_path_prefix,
                &format!(
                    "document/{}/template/json",
                    self.metadata.note_id.to_simple_string()
                ),
            );
            self.load_templates_from_directory(&document_templates_path, &mut templates)?;

            self.templates = Some(templates);
        }

        Ok(self.templates.as_ref().unwrap())
    }

    pub fn default_template(&mut self) -> Result<Option<&template::TemplateDescriptor>> {
        Ok(self.templates()?.default.as_ref())
    }

    pub fn template_for_page(
        &mut self,
        page_id: &PageUuid,
    ) -> Result<Option<&template::TemplateDescriptor>> {
        Ok(self.templates()?.for_page(page_id))
    }

    pub fn resources(&mut self) -> Result<&Vec<resource::ResourceRecord>> {
        if self.resources.is_none() {
            let mut resources = Vec::new();
            let resource_path = join_archive_path(&self.note_path_prefix, "resource/pb");

            if self.container.has_entry_relative(&resource_path) {
                for archive_path in self.container.list_directory(&resource_path) {
                    let container_relative_path =
                        to_container_relative_path(self.container.root_path(), &archive_path);
                    let note_relative_path =
                        to_note_relative_path(&self.note_path_prefix, &container_relative_path);
                    let size_bytes = self
                        .container
                        .file_size_relative(&container_relative_path)?;

                    let resource = if size_bytes == 0 {
                        resource::ResourceRecord::read(
                            note_relative_path,
                            std::io::Cursor::new(Vec::<u8>::new()),
                            size_bytes,
                        )?
                    } else {
                        let bytes =
                            self.container
                                .get_file_absolute(&archive_path, |mut reader| {
                                    let mut buffer = Vec::new();
                                    reader.read_to_end(&mut buffer).map_err(Error::Io)?;
                                    Ok(buffer)
                                })?;
                        resource::ResourceRecord::read(
                            note_relative_path,
                            std::io::Cursor::new(bytes),
                            size_bytes,
                        )?
                    };
                    resources.push(resource);
                }
            }

            self.resources = Some(resources);
        }

        Ok(self.resources.as_ref().unwrap())
    }

    pub fn assets_metadata(&mut self) -> Result<&note_assets::NoteAssetsMetadata> {
        if self.assets_metadata.is_none() {
            let toc_path = join_archive_path(&self.note_path_prefix, "toc");
            let toc_exists = self.container.has_entry_relative(&toc_path);

            let mut toc_files = Vec::new();
            for archive_path in self.container.list_directory(&toc_path) {
                let container_relative_path =
                    to_container_relative_path(self.container.root_path(), &archive_path);
                let note_relative_path =
                    to_note_relative_path(&self.note_path_prefix, &container_relative_path);
                let size_bytes = self
                    .container
                    .file_size_relative(&container_relative_path)?;
                toc_files.push(note_assets::FileMetadata {
                    path: note_relative_path,
                    size_bytes,
                });
            }

            let preferred_preview_path = join_archive_path(
                &self.note_path_prefix,
                &format!("{}.png", self.metadata.note_id.to_simple_string()),
            );
            let preview = if self.container.has_entry_relative(&preferred_preview_path) {
                let size_bytes = self.container.file_size_relative(&preferred_preview_path)?;
                Some(note_assets::FileMetadata {
                    path: to_note_relative_path(&self.note_path_prefix, &preferred_preview_path),
                    size_bytes,
                })
            } else {
                self.discover_fallback_preview()?
            };

            self.assets_metadata = Some(note_assets::NoteAssetsMetadata {
                toc: note_assets::TocMetadata {
                    exists: toc_exists,
                    files: toc_files,
                },
                preview,
            });
        }

        Ok(self.assets_metadata.as_ref().unwrap())
    }

    fn load_templates_from_directory(
        &mut self,
        directory_path: &str,
        templates: &mut template::TemplateSet,
    ) -> Result {
        if !self.container.has_entry_relative(directory_path) {
            return Ok(());
        }

        for template_path in self.container.list_directory(directory_path) {
            let file_name = file_name_from_path(&template_path)?;
            let Some(key) = template::classify_template_file_name(file_name) else {
                continue;
            };

            let descriptor = self.container.get_file_absolute(&template_path, |reader| {
                template::TemplateDescriptor::read(reader)
            })?;
            templates.insert_if_absent(key, descriptor);
        }

        Ok(())
    }

    fn discover_fallback_preview(&mut self) -> Result<Option<note_assets::FileMetadata>> {
        let note_root_path = self.note_path_prefix.as_str();
        for archive_path in self.container.list_directory(note_root_path) {
            let container_relative_path =
                to_container_relative_path(self.container.root_path(), &archive_path);
            let note_relative_path =
                to_note_relative_path(&self.note_path_prefix, &container_relative_path);

            if !note_relative_path.ends_with(".png") || note_relative_path.contains('/') {
                continue;
            }

            let size_bytes = self
                .container
                .file_size_relative(&container_relative_path)?;
            return Ok(Some(note_assets::FileMetadata {
                path: note_relative_path,
                size_bytes,
            }));
        }

        Ok(None)
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
    note_path_prefix: String,
    page_id: PageUuid,
    virtual_page: Option<VirtualPage>,
    page_model: PageModel,
    shape_groups: Option<HashMap<ShapeGroupUuid, ShapeGroup>>,
    points_files: Option<HashMap<PointsUuid, Vec<points::PointsFile>>>,
    template: Option<Option<template::TemplateDescriptor>>,
}

impl<R: std::io::Read + std::io::Seek> Page<R> {
    fn new(
        container: container::Container<R>,
        note_id: NoteUuid,
        page_id: PageUuid,
        note_path_prefix: String,
        virtual_page: Option<VirtualPage>,
        page_model: PageModel,
    ) -> Self {
        Self {
            container,
            note_id,
            page_id,
            note_path_prefix,
            virtual_page,
            page_model,
            shape_groups: None,
            points_files: None,
            template: None,
        }
    }

    pub fn virtual_page(&self) -> &Option<VirtualPage> {
        &self.virtual_page
    }

    pub fn page_model(&self) -> &PageModel {
        &self.page_model
    }

    pub fn template(&mut self) -> Result<Option<&template::TemplateDescriptor>> {
        if self.template.is_none() {
            let page_id_simple = self.page_id.to_simple_string();
            let page_id_hyphenated = self.page_id.to_hyphenated_string();
            let note_id_simple = self.note_id.to_simple_string();

            let template_paths = [
                join_archive_path(
                    &self.note_path_prefix,
                    &format!("template/json/{}.template_json", page_id_simple),
                ),
                join_archive_path(
                    &self.note_path_prefix,
                    &format!("template/json/{}.template_json", page_id_hyphenated),
                ),
                join_archive_path(
                    &self.note_path_prefix,
                    &format!(
                        "document/{}/template/json/{}.template_json",
                        note_id_simple, page_id_simple
                    ),
                ),
                join_archive_path(
                    &self.note_path_prefix,
                    &format!(
                        "document/{}/template/json/{}.template_json",
                        note_id_simple, page_id_hyphenated
                    ),
                ),
            ];

            let mut matched_template = None;
            for template_path in template_paths {
                if !self.container.has_entry_relative(&template_path) {
                    continue;
                }

                matched_template =
                    Some(self.container.get_file_relative(&template_path, |reader| {
                        template::TemplateDescriptor::read(reader)
                    })?);
                break;
            }

            self.template = Some(matched_template);
        }

        Ok(self.template.as_ref().unwrap().as_ref())
    }

    pub fn shape_groups(&mut self) -> Result<&HashMap<ShapeGroupUuid, ShapeGroup>> {
        if self.shape_groups.is_none() {
            let page_id = self.page_id.to_simple_string();

            let mut shape_groups = HashMap::new();

            let shape_prefix =
                join_archive_path(&self.note_path_prefix, &format!("shape/{}#", page_id));
            for shape_group_path in self.container.list_directory(&shape_prefix) {
                let path_tail = file_name_from_path(&shape_group_path)?;
                let (shape_group_id, timestamp) = parse_shape_group_file_name(path_tail)?;
                let _timestamp = convert_timestamp_to_datetime(timestamp)?;
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
            let page_id = self.page_id.to_simple_string();

            let mut points_files = HashMap::new();

            let points_prefix = join_archive_path(
                &self.note_path_prefix,
                &format!("point/{}/{}#", page_id, page_id),
            );
            for stroke_path in self.container.list_directory(&points_prefix) {
                let path_tail = file_name_from_path(&stroke_path)?;
                let shape_id = parse_points_file_name(path_tail)?;

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

fn join_archive_path(prefix: &str, tail: &str) -> String {
    if prefix.is_empty() {
        tail.to_string()
    } else {
        format!("{}/{}", prefix, tail)
    }
}

fn file_name_from_path(path: &str) -> Result<&str> {
    path.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| Error::InvalidArchiveEntryName(path.to_string()))
}

fn to_container_relative_path(root_path: &str, archive_path: &str) -> String {
    if root_path.is_empty() {
        return archive_path.trim_start_matches('/').to_string();
    }

    archive_path
        .trim_start_matches('/')
        .strip_prefix(&format!("{}/", root_path))
        .unwrap_or_else(|| archive_path.trim_start_matches('/'))
        .to_string()
}

fn to_note_relative_path(note_path_prefix: &str, container_relative_path: &str) -> String {
    if note_path_prefix.is_empty() {
        return container_relative_path.to_string();
    }

    container_relative_path
        .strip_prefix(&format!("{}/", note_path_prefix))
        .unwrap_or(container_relative_path)
        .to_string()
}

fn parse_shape_group_file_name(file_name: &str) -> Result<(ShapeGroupUuid, u64)> {
    let mut segments = file_name.split('#');
    let _page_id = segments.next();
    let shape_group_uuid = segments.next();
    let timestamp_with_suffix = segments.next();
    let extra = segments.next();

    if shape_group_uuid.is_none() || timestamp_with_suffix.is_none() || extra.is_some() {
        return Err(Error::InvalidArchiveEntryName(file_name.to_string()));
    }

    let shape_group_id = ShapeGroupUuid::from_str(shape_group_uuid.unwrap())?;

    let timestamp_with_suffix = timestamp_with_suffix.unwrap();
    let timestamp_str = timestamp_with_suffix
        .strip_suffix(".zip")
        .ok_or_else(|| Error::InvalidArchiveEntryName(file_name.to_string()))?;
    let timestamp = timestamp_str.parse::<u64>().map_err(|e| {
        Error::InvalidTimestampFormat(format!(
            "Failed to parse timestamp in '{}': {}",
            file_name, e
        ))
    })?;

    Ok((shape_group_id, timestamp))
}

fn parse_points_file_name(file_name: &str) -> Result<PointsUuid> {
    let mut segments = file_name.split('#');
    let _page_id = segments.next();
    let points_uuid = segments.next();
    let marker = segments.next();
    let extra = segments.next();

    if points_uuid.is_none() || marker != Some("points") || extra.is_some() {
        return Err(Error::InvalidArchiveEntryName(file_name.to_string()));
    }

    PointsUuid::from_str(points_uuid.unwrap())
}

impl<R: std::io::Read + std::io::Seek> std::fmt::Debug for Page<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Page")
            .field("virtual_page", &self.virtual_page)
            .field("page_model", &self.page_model)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        id::{PointsUuid, ShapeGroupUuid},
        parse_points_file_name, parse_shape_group_file_name,
    };

    #[test]
    fn parses_shape_group_filename() {
        let file_name = "ba338e220eda49268c7126a02970a160#537164a1-9052-496a-80d3-a3aadcff339b#1753456222637.zip";
        let (shape_group_uuid, timestamp) = parse_shape_group_file_name(file_name).unwrap();

        assert_eq!(
            shape_group_uuid,
            ShapeGroupUuid::from_str("537164a1-9052-496a-80d3-a3aadcff339b").unwrap()
        );
        assert_eq!(timestamp, 1753456222637);
    }

    #[test]
    fn rejects_invalid_shape_group_filename() {
        let file_name =
            "ba338e220eda49268c7126a02970a160#537164a1-9052-496a-80d3-a3aadcff339b#1753456222637";
        assert!(parse_shape_group_file_name(file_name).is_err());
    }

    #[test]
    fn parses_points_filename() {
        let file_name =
            "ba338e220eda49268c7126a02970a160#e858c829-d2f3-4994-b9fb-95bcc8003fa3#points";
        let points_uuid = parse_points_file_name(file_name).unwrap();

        assert_eq!(
            points_uuid,
            PointsUuid::from_str("e858c829-d2f3-4994-b9fb-95bcc8003fa3").unwrap()
        );
    }

    #[test]
    fn rejects_invalid_points_filename() {
        let file_name = "ba338e220eda49268c7126a02970a160#e858c829-d2f3-4994-b9fb-95bcc8003fa3";
        assert!(parse_points_file_name(file_name).is_err());
    }
}
