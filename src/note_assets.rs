#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMetadata {
    pub path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocMetadata {
    pub exists: bool,
    pub files: Vec<FileMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteAssetsMetadata {
    pub toc: TocMetadata,
    pub preview: Option<FileMetadata>,
}
