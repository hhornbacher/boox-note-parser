use std::{
    collections::HashSet,
    env, fs,
    fs::File,
    io::{Error as IoError, ErrorKind},
    path::{Path, PathBuf},
};

use boox_note_parser::{Note, NoteFile, id::PageUuid};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let bin_name = args.next().unwrap_or_else(|| "render".to_string());

    let input_path = args.next().map(PathBuf::from).ok_or_else(|| {
        print_usage(&bin_name);
        IoError::new(ErrorKind::InvalidInput, "missing input .note file path")
    })?;

    let output_dir = args.next().map(PathBuf::from).unwrap_or_else(|| ".".into());

    if args.next().is_some() {
        print_usage(&bin_name);
        return Err(IoError::new(
            ErrorKind::InvalidInput,
            "too many arguments; expected: <file.note> [output_dir]",
        )
        .into());
    }

    if !input_path.exists() {
        return Err(IoError::new(
            ErrorKind::NotFound,
            format!("input file not found: {}", input_path.display()),
        )
        .into());
    }

    fs::create_dir_all(&output_dir)?;

    let file = File::open(&input_path)?;
    let note_file = NoteFile::read(file)?;

    let mut notes: Vec<_> = note_file.list_notes().into_iter().collect();
    notes.sort_by_key(|(note_id, name)| (name.clone(), note_id.to_simple_string()));

    for (note_id, _name) in notes {
        let mut note = note_file.get_note(&note_id).ok_or_else(|| {
            IoError::new(
                ErrorKind::NotFound,
                format!(
                    "note metadata exists but note payload is missing for {}",
                    note_id.to_hyphenated_string()
                ),
            )
        })?;

        let note_file_prefix = sanitize_file_component(note.name());
        let page_ids = collect_all_page_ids(&note);

        if page_ids.is_empty() {
            println!(
                "Skipping note '{}' ({}): no pages",
                note.name(),
                note_id.to_hyphenated_string()
            );
            continue;
        }

        println!(
            "Rendering note '{}' ({}) with {} page(s)",
            note.name(),
            note_id.to_hyphenated_string(),
            page_ids.len()
        );

        for page_id in page_ids {
            let mut page = note.get_page(&page_id).ok_or_else(|| {
                IoError::new(
                    ErrorKind::NotFound,
                    format!(
                        "failed to resolve page {} for note {}",
                        page_id.to_hyphenated_string(),
                        note_id.to_hyphenated_string()
                    ),
                )
            })?;

            let draw_target = page.render()?;
            let output_name = format!(
                "{}_{}_{}.png",
                note_file_prefix,
                note_id.to_simple_string(),
                page_id.to_simple_string()
            );
            let output_path = output_dir.join(output_name);
            draw_target.write_png(&output_path)?;
            println!("  wrote {}", output_path.display());
        }
    }

    Ok(())
}

fn collect_all_page_ids<R: std::io::Read + std::io::Seek>(note: &Note<R>) -> Vec<PageUuid> {
    let mut page_ids = Vec::new();
    let mut seen = HashSet::new();

    for page_id in note
        .active_pages()
        .iter()
        .copied()
        .chain(note.reserved_pages().iter().copied())
        .chain(note.detached_pages().iter().copied())
    {
        if seen.insert(page_id) {
            page_ids.push(page_id);
        }
    }

    page_ids
}

fn sanitize_file_component(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut last_was_separator = false;

    for ch in name.chars() {
        let normalized = if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
            Some(ch)
        } else if ch.is_ascii_whitespace() {
            Some('_')
        } else {
            Some('_')
        };

        if let Some(ch) = normalized {
            if ch == '_' {
                if !last_was_separator {
                    out.push(ch);
                }
                last_was_separator = true;
            } else {
                out.push(ch);
                last_was_separator = false;
            }
        }
    }

    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "note".to_string()
    } else {
        trimmed.to_string()
    }
}

fn print_usage(bin_name: &str) {
    let name = Path::new(bin_name)
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or(bin_name);
    eprintln!("Usage: {name} <file.note> [output_dir]");
}
