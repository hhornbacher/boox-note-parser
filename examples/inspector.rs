use std::{fs::File, path::Path};

use boox_note_parser::{
    Note, NoteFile, error::Result, id::PageUuid, points::PointsFile, shape::Shape,
    virtual_page::VirtualPage,
};
use raqote::{DrawOptions, DrawTarget, Source};
use tracing_subscriber::filter::LevelFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <your_notes.note>", args[0]);
        std::process::exit(1);
    }
    let path = Path::new(&args[1]);

    let file = File::open(path).expect("Failed to open file");

    let note_file = NoteFile::read(file).expect("Failed to read note file");

    println!("Notes:");
    for (note_id, name) in note_file.list_notes() {
        println!("  Note ID: {}", note_id.to_hyphenated_string());
        println!("    Name: {}", name);

        let mut note = note_file.get_note(&note_id).unwrap();
        println!("    Created: {}", note.created());
        println!("    Modified: {}", note.modified());

        let virtual_doc = note.virtual_doc().expect("No virtual doc found for note");
        println!(
            "    Virtual Doc ID: {}",
            virtual_doc.virtual_doc_id.to_hyphenated_string()
        );
        println!("      Created: {}", virtual_doc.created);
        println!("      Modified: {}", virtual_doc.modified);
        println!("      Content: {:?}", virtual_doc.content);

        println!("    Active Pages:");
        let active_pages = note.active_pages().to_vec();
        list_pages(&mut note, active_pages);

        println!("    Reserved Pages:");
        let reserved_pages = note.reserved_pages().to_vec();
        list_pages(&mut note, reserved_pages);

        println!("    Detached Pages:");
        let detached_pages = note.detached_pages().to_vec();
        list_pages(&mut note, detached_pages);
    }
}

fn list_pages<R: std::io::Read + std::io::Seek>(note: &mut Note<R>, pages: Vec<PageUuid>) {
    for page_id in &pages {
        println!("      Page ID: {}", page_id.to_hyphenated_string());

        let page = note.get_page(page_id).expect("Failed to get page");

        let page_model = page.page_model();
        println!("        Page Model:",);
        println!("          Created: {}", page_model.created);
        println!("          Modified: {}", page_model.modified);
        println!("          Dimensions: {:?}", page_model.dimensions);
        println!("          Layers: {:?}", page_model.layers);

        if let Some(virtual_page) = page.virtual_page() {
            println!("        Virtual Page:");
            println!("          Created: {}", virtual_page.created);
            println!("          Modified: {}", virtual_page.modified);
            println!("          Zoom Scale: {}", virtual_page.zoom_scale);
            println!("          Dimensions: {:?}", virtual_page.dimensions);
            println!("          Layout: {:?}", virtual_page.layout);
            println!("          Geo: {:?}", virtual_page.geo);
            println!("          Geo Layout: {}", virtual_page.geo_layout);
            println!("          Template Path: {}", virtual_page.template_path);
            println!("          Page Number: {}", virtual_page.page_number);
        } else {
            println!("        No virtual page found for this page.");
        }
    }
}

fn draw_page(virtual_page: &VirtualPage, shapes: &[Shape], points_file: &PointsFile) -> Result<()> {
    let width = (virtual_page.dimensions.right - virtual_page.dimensions.left) as i32;
    let height = (virtual_page.dimensions.bottom - virtual_page.dimensions.top) as i32;
    log::info!("Drawing page with dimensions: {}x{}", width, height);
    let mut draw_target = DrawTarget::new(width, height);

    draw_target.fill_rect(
        0.0,
        0.0,
        width as f32,
        height as f32,
        &Source::Solid(raqote::Color::new(255, 255, 255, 255).into()),
        &DrawOptions::new(),
    );

    let mut shapes = shapes.to_vec();
    shapes.sort_by_key(|shape| shape.z_order);

    for shape in shapes {
        let stroke_uuid = shape.stroke_id;

        if let Some(stroke) = points_file.get_stroke(&stroke_uuid) {
            stroke.draw(&mut draw_target)?;
        } else {
            log::warn!("No points found for stroke UUID: {}", stroke_uuid);
        }
    }

    draw_target
        .write_png(format!("{}.png", virtual_page.page_id.to_simple_string()))
        .expect("Failed to write PNG");
    Ok(())
}
