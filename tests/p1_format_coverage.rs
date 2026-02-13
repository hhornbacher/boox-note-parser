use std::{fs::File, path::PathBuf};

use boox_note_parser::{
    NoteFile,
    id::{NoteUuid, PageUuid},
    resource::ResourcePayload,
};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name)
}

fn corpus_tests_enabled() -> bool {
    std::env::var("BOOX_NOTE_PARSER_RUN_CORPUS_TESTS")
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            normalized == "1" || normalized == "true" || normalized == "yes"
        })
        .unwrap_or(false)
}

fn open_corpus_file(name: &str) -> Option<File> {
    if !corpus_tests_enabled() {
        eprintln!("Skipping corpus test '{name}' (set BOOX_NOTE_PARSER_RUN_CORPUS_TESTS=1)");
        return None;
    }

    let path = fixture_path(name);
    if !path.exists() {
        eprintln!(
            "Skipping corpus test '{name}' (fixture '{}' missing)",
            path.display()
        );
        return None;
    }

    Some(File::open(path).unwrap())
}

#[test]
fn stroke_tests_note_exposes_p1_metadata() {
    let Some(file) = open_corpus_file("Stroke Tests.note") else {
        return;
    };
    let note_file = NoteFile::read(file).unwrap();

    let note_id = NoteUuid::from_str("7a960ca753b0420ea2d5b88d57f7bf62").unwrap();
    let page_id = PageUuid::from_str("ba338e220eda49268c7126a02970a160").unwrap();

    let mut note = note_file.get_note(&note_id).unwrap();

    let extra = note.extra_metadata().unwrap().unwrap();
    assert_eq!(extra.flag, 1);
    assert_eq!(extra.app_build, 42842);
    assert_eq!(extra.key, "share_user");

    let resources = note.resources().unwrap();
    assert_eq!(resources.len(), 1);
    assert!(matches!(resources[0].payload, ResourcePayload::Empty));

    assert!(note.default_template().unwrap().is_none());
    assert!(note.template_for_page(&page_id).unwrap().is_some());

    let assets = note.assets_metadata().unwrap();
    assert!(!assets.toc.exists);
    assert!(assets.preview.is_none());
}

#[test]
fn mybackup_note_exposes_templates_toc_and_preview() {
    let Some(file) = open_corpus_file("MyBackup_1753137342560.note") else {
        return;
    };
    let note_file = NoteFile::read(file).unwrap();

    let note_id = NoteUuid::from_str("a38df30a1bf343988305352f90b7086c").unwrap();
    let mut note = note_file.get_note(&note_id).unwrap();

    assert!(note.default_template().unwrap().is_some());
    assert!(!note.templates().unwrap().by_page.is_empty());

    let assets = note.assets_metadata().unwrap();
    assert!(assets.toc.exists);
    let preview = assets.preview.as_ref().unwrap();
    assert!(preview.size_bytes > 0);
}
