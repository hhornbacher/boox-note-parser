use std::io::{Cursor, Write};

use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

pub const NOTE_ID_SIMPLE: &str = "7a960ca753b0420ea2d5b88d57f7bf62";
pub const PAGE_ID_SIMPLE: &str = "ba338e220eda49268c7126a02970a160";
pub const POINTS_ID_HYPHENATED: &str = "e858c829-d2f3-4994-b9fb-95bcc8003fa3";
pub const SHAPE_GROUP_ID_HYPHENATED: &str = "537164a1-9052-496a-80d3-a3aadcff339b";
pub const VIRTUAL_PAGE_ID_HYPHENATED: &str = "01984222-a0c1-7994-a697-54490ff82e60";
pub const PAGE_MODEL_ID_HYPHENATED: &str = "01984222-a0e1-7fbd-89cd-15c36de327a9";
pub const RESOURCE_FILE_NAME: &str = "b199553d-a169-4f7f-ae01-f9312c9cbd3c#1753456222446";

const NOTE_INFO_BYTES: &[u8] = include_bytes!("../fixtures/p2/note_info.pb");
const NOTE_TREE_BYTES: &[u8] = include_bytes!("../fixtures/p2/note_tree.pb");
const VIRTUAL_DOC_BYTES: &[u8] = include_bytes!("../fixtures/p2/virtual_doc.pb");
const VIRTUAL_PAGE_BYTES: &[u8] = include_bytes!("../fixtures/p2/virtual_page.pb");
const PAGE_MODEL_BYTES: &[u8] = include_bytes!("../fixtures/p2/page_model.pb");
const SHAPE_GROUP_BYTES: &[u8] = include_bytes!("../fixtures/p2/shape_group.zip");
const POINTS_BYTES: &[u8] = include_bytes!("../fixtures/p2/points.bin");
const TEMPLATE_PAGE_BYTES: &[u8] = include_bytes!("../fixtures/p2/template_page.template_json");
const TEMPLATE_DEFAULT_BYTES: &[u8] =
    include_bytes!("../fixtures/p2/template_default.template_json");
const EXTRA_BYTES: &[u8] = include_bytes!("../fixtures/p2/extra.pb");
const RESOURCE_EMPTY_BYTES: &[u8] = include_bytes!("../fixtures/p2/resource_empty.bin");
const PREVIEW_BYTES: &[u8] = include_bytes!("../fixtures/p2/preview.png");
const TOC_INDEX_BYTES: &[u8] = include_bytes!("../fixtures/p2/toc_index.bin");

fn write_archive(entries: Vec<(String, &'static [u8])>) -> Cursor<Vec<u8>> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    for (path, content) in entries {
        writer.start_file(path, options).unwrap();
        writer.write_all(content).unwrap();
    }

    writer.finish().unwrap()
}

fn common_note_entries(note_prefix: &str) -> Vec<(String, &'static [u8])> {
    vec![
        (
            format!("{note_prefix}/virtual/doc/pb/{NOTE_ID_SIMPLE}"),
            VIRTUAL_DOC_BYTES,
        ),
        (
            format!("{note_prefix}/virtual/page/pb/{VIRTUAL_PAGE_ID_HYPHENATED}"),
            VIRTUAL_PAGE_BYTES,
        ),
        (
            format!("{note_prefix}/pageModel/pb/{PAGE_MODEL_ID_HYPHENATED}"),
            PAGE_MODEL_BYTES,
        ),
        (
            format!(
                "{note_prefix}/shape/{PAGE_ID_SIMPLE}#{SHAPE_GROUP_ID_HYPHENATED}#1753456222637.zip"
            ),
            SHAPE_GROUP_BYTES,
        ),
        (
            format!(
                "{note_prefix}/point/{PAGE_ID_SIMPLE}/{PAGE_ID_SIMPLE}#{POINTS_ID_HYPHENATED}#points"
            ),
            POINTS_BYTES,
        ),
        (
            format!("{note_prefix}/template/json/{PAGE_ID_SIMPLE}.template_json"),
            TEMPLATE_PAGE_BYTES,
        ),
        (
            format!("{note_prefix}/template/json/.template_json"),
            TEMPLATE_DEFAULT_BYTES,
        ),
        (
            format!(
                "{note_prefix}/document/{NOTE_ID_SIMPLE}/template/json/{PAGE_ID_SIMPLE}.template_json"
            ),
            TEMPLATE_PAGE_BYTES,
        ),
        (format!("{note_prefix}/extra/pb/extra"), EXTRA_BYTES),
        (
            format!("{note_prefix}/resource/pb/{RESOURCE_FILE_NAME}"),
            RESOURCE_EMPTY_BYTES,
        ),
    ]
}

pub fn rooted_single_note_archive() -> Cursor<Vec<u8>> {
    let note_prefix = format!("export/{NOTE_ID_SIMPLE}");
    let mut entries = common_note_entries(&note_prefix);
    entries.push((format!("{note_prefix}/note/pb/note_info"), NOTE_INFO_BYTES));
    write_archive(entries)
}

pub fn multi_note_archive_rootless() -> Cursor<Vec<u8>> {
    let note_prefix = NOTE_ID_SIMPLE.to_string();
    let mut entries = common_note_entries(&note_prefix);
    entries.push(("note_tree".to_string(), NOTE_TREE_BYTES));
    entries.push((format!("{note_prefix}/{NOTE_ID_SIMPLE}.png"), PREVIEW_BYTES));
    entries.push((format!("{note_prefix}/toc/pb/index"), TOC_INDEX_BYTES));
    write_archive(entries)
}

pub fn multi_note_archive_rooted() -> Cursor<Vec<u8>> {
    let note_prefix = format!("backup/{NOTE_ID_SIMPLE}");
    let mut entries = common_note_entries(&note_prefix);
    entries.push(("backup/note_tree".to_string(), NOTE_TREE_BYTES));
    entries.push((format!("{note_prefix}/{NOTE_ID_SIMPLE}.png"), PREVIEW_BYTES));
    entries.push((format!("{note_prefix}/toc/pb/index"), TOC_INDEX_BYTES));
    write_archive(entries)
}
