mod common;

use boox_note_parser::{
    NoteFile,
    id::{NoteUuid, PageUuid},
    resource::ResourcePayload,
};

fn assert_render_has_colored_strokes(draw_target: &raqote::DrawTarget) {
    let pixels = draw_target.get_data();
    let non_background_count = pixels.iter().filter(|pixel| **pixel != 0xFFFF_FFFF).count();
    assert!(
        non_background_count > 0,
        "rendered output should contain non-background pixels"
    );

    let has_non_grayscale = pixels.iter().any(|pixel| {
        let r = ((pixel >> 16) & 0xff) as u8;
        let g = ((pixel >> 8) & 0xff) as u8;
        let b = (pixel & 0xff) as u8;
        r != g || g != b
    });
    let mut unique_colors = pixels.iter().copied().collect::<Vec<_>>();
    unique_colors.sort_unstable();
    unique_colors.dedup();
    let sample = unique_colors.into_iter().take(12).collect::<Vec<_>>();
    assert!(
        has_non_grayscale,
        "rendered output should contain at least one non-grayscale pixel (non-background={non_background_count}, sample={sample:#x?})"
    );
}

#[test]
fn rooted_single_note_fixture_exercises_p2_paths() {
    let note_file = NoteFile::read(common::rooted_single_note_archive()).unwrap();

    assert_eq!(note_file.list_notes().len(), 1);

    let note_id = NoteUuid::from_str(common::NOTE_ID_SIMPLE).unwrap();
    let page_id = PageUuid::from_str(common::PAGE_ID_SIMPLE).unwrap();
    let mut note = note_file.get_note(&note_id).unwrap();

    let extra = note.extra_metadata().unwrap().unwrap();
    assert_eq!(extra.key, "share_user");

    let resources = note.resources().unwrap();
    assert_eq!(resources.len(), 1);
    assert!(matches!(resources[0].payload, ResourcePayload::Empty));

    assert!(note.default_template().unwrap().is_some());
    assert!(note.template_for_page(&page_id).unwrap().is_some());

    let mut page = note.get_page(&page_id).unwrap();
    assert!(page.template().unwrap().is_some());
    let rendered = page.render().unwrap();
    assert_render_has_colored_strokes(&rendered);
}

#[test]
fn multi_note_fixture_rootless_exposes_assets_and_templates() {
    let note_file = NoteFile::read(common::multi_note_archive_rootless()).unwrap();

    let note_id = NoteUuid::from_str(common::NOTE_ID_SIMPLE).unwrap();
    let page_id = PageUuid::from_str(common::PAGE_ID_SIMPLE).unwrap();
    let mut note = note_file.get_note(&note_id).unwrap();

    assert!(note.default_template().unwrap().is_some());
    assert!(note.template_for_page(&page_id).unwrap().is_some());

    let assets = note.assets_metadata().unwrap();
    assert!(assets.toc.exists);
    let preview = assets.preview.as_ref().unwrap();
    assert!(preview.size_bytes > 0);
}

#[test]
fn multi_note_fixture_rooted_detects_note_tree_under_prefix() {
    let note_file = NoteFile::read(common::multi_note_archive_rooted()).unwrap();

    let note_id = NoteUuid::from_str(common::NOTE_ID_SIMPLE).unwrap();
    let page_id = PageUuid::from_str(common::PAGE_ID_SIMPLE).unwrap();
    let mut note = note_file.get_note(&note_id).unwrap();

    let mut page = note.get_page(&page_id).unwrap();
    let rendered = page.render().unwrap();
    assert_render_has_colored_strokes(&rendered);
}
