use std::{fs::File, path::Path};

use boox_note_parser::NoteFile;

fn main() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <your_notes.note>", args[0]);
        std::process::exit(1);
    }
    let path = Path::new(&args[1]);

    let file = File::open(path).expect("Failed to open file");

    let note_file = NoteFile::read(file).expect("Failed to read note file");

    println!("Container type: {:?}", note_file.container_type());

    println!("Notes:");
    for (id, name) in note_file.list_notes() {
        println!("  {}: {}", id, name);
    }
}
