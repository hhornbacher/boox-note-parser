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
        println!(
            "    Pen Settings Fill Color: {:08x}",
            note.pen_settings_fill_color()
        );
        println!(
            "    Pen Settings Graphics Shape Color: {:08x}",
            note.pen_settings_graphics_shape_color()
        );

        match note
            .extra_metadata()
            .expect("Failed to parse extra metadata")
        {
            Some(extra) => {
                println!("    Extra Metadata:");
                println!("      Flag: {}", extra.flag);
                println!("      App Build: {}", extra.app_build);
                println!("      Key: {}", extra.key);
                println!("      Value: {}", extra.value);
            }
            None => println!("    Extra Metadata: none"),
        }

        let templates = note.templates().expect("Failed to parse templates");
        println!("    Templates:");
        println!("      Default: {}", templates.default.is_some());
        println!("      Per-page count: {}", templates.by_page.len());
        println!("      Other IDs count: {}", templates.by_other_id.len());

        let resources = note.resources().expect("Failed to parse resources");
        println!("    Resources: {}", resources.len());
        for resource in resources {
            let payload_kind = match &resource.payload {
                boox_note_parser::resource::ResourcePayload::Empty => "Empty",
                boox_note_parser::resource::ResourcePayload::Raw(_) => "Raw",
            };
            println!(
                "      {} ({} bytes, payload: {})",
                resource.path, resource.size_bytes, payload_kind
            );
        }

        let assets = note
            .assets_metadata()
            .expect("Failed to discover note assets metadata");
        println!("    TOC exists: {}", assets.toc.exists);
        println!("    TOC file count: {}", assets.toc.files.len());
        if let Some(preview) = &assets.preview {
            println!(
                "    Preview: {} ({} bytes)",
                preview.path, preview.size_bytes
            );
        } else {
            println!("    Preview: none");
        }

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

        let Some(mut page) = note.get_page(page_id) else {
            println!("        (no page model — detached/orphan)");
            continue;
        };

        if page.is_empty().expect("Failed to inspect page contents") {
            log::warn!(
                "Skipping render for empty page {} in note {:?}",
                page_id.to_hyphenated_string(),
                note.name()
            );
            println!("        (empty page — skipped render)");
        } else {
            let draw_target = page.render().expect("Failed to render page");

            draw_target
                .write_png(format!(
                    "{}_{}.png",
                    note.name(),
                    page_id.to_simple_string()
                ))
                .expect("Failed to write PNG");
        }

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

        if let Some(template) = page.template().expect("Failed to load page template") {
            println!(
                "        Template: type={}, subtype={}, resource={}",
                template.type_,
                template.properties.sub_type,
                template.properties.resource_attr.res_name
            );
        } else {
            println!("        Template: none");
        }
    }
}
