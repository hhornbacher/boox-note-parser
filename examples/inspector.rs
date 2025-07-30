use std::{fs::File, path::Path};

use boox_note_parser::{Note, NoteFile, id::PageUuid};
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
        println!("    Flag: {:08x}", note.flag());
        println!("    Pen Width: {}", note.pen_width());
        println!("    Pen Type: {}", note.pen_type());
        println!("    Scale factor: {}", note.scale_factor());
        println!("    Fill Color: {:08x}", note.fill_color());
        println!("    Pen Settings Fill Color: {:08x}", note.pen_settings_fill_color());
        println!("    Pen Settings Graphics Shape Color: {:08x}", note.pen_settings_graphics_shape_color());

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

        let mut page = note.get_page(page_id).expect("Failed to get page");

        let draw_target = page.render().expect("Failed to render page");

        draw_target
            .write_png(format!(
                "{}_{}.png",
                note.name(),
                page_id.to_simple_string()
            ))
            .expect("Failed to write PNG");

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
