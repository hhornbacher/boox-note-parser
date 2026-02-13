# boox-note-parser

**WARNING:** This crate is still work in progress.

> A Rust library for parsing `.note` files from Onyx Boox e-ink devices.

[![Crates.io](https://img.shields.io/crates/v/boox-note-parser.svg)](https://crates.io/crates/boox-note-parser)
[![Docs.rs](https://docs.rs/boox-note-parser/badge.svg)](https://docs.rs/boox-note-parser)
[![License](https://img.shields.io/crates/l/boox-note-parser.svg)](https://github.com/hhornbacher/boox-note-parser/blob/main/LICENSE)

`boox-note-parser` provides a pure Rust implementation for reading and interpreting handwritten note data stored in `.note` files on Boox devices.
The format is undocumented and [reverse-engineered](docs/format.md) from real note exports.

## What Works Today

- Open single-note and multi-note `.note` archives.
- Read note metadata (`note_tree` / `note_info` protobuf payloads).
- Parse page models and virtual page/document metadata.
- Parse shape groups and custom points stroke files.
- Parse `extra/pb/extra` metadata.
- Parse templates from `template/json/*.template_json` and `.template_json` (including mirrored `document/<note_id>/...` paths).
- Surface `resource/pb/*` records, including explicit handling for empty payload files.
- Surface TOC and preview PNG asset metadata.
- Render page strokes to PNG using style-aware color/width/line-style mapping.

## Quick Start

```rust
use std::{fs::File, path::Path};
use boox_note_parser::NoteFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("MyBackup_1753137342560.note");
    let file = File::open(path)?;
    let note_file = NoteFile::read(file)?;

    for (note_id, name) in note_file.list_notes() {
        println!("{} -> {}", note_id.to_hyphenated_string(), name);
    }

    Ok(())
}
```

You can also run the example inspector:

```bash
cargo run --example inspector -- <path-to-note-file>
```

## Try It Out

### CLI Track (Inspector Example)

Run the built-in inspector:

```bash
cargo run --example inspector -- <path-to-note-file.note>
```

Expected behavior:

- Prints note metadata (IDs, names, timestamps, pen settings).
- Prints template/resource/extra/asset metadata (`Templates:`, `Resources:`, `TOC exists:`, `Preview:`).
- Renders pages and writes PNG files to the current directory.

Quick verification checks:

- Confirm output includes lines for template/resource/assets.
- Confirm generated PNGs are present in your working directory.

### Library Track (Use the Crate in Your Own Binary)

Minimal example:

```rust
use std::{env, fs::File, path::PathBuf};

use boox_note_parser::NoteFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let note_path = env::args().nth(1).expect("Usage: app <file.note>");
    let file = File::open(PathBuf::from(note_path))?;
    let note_file = NoteFile::read(file)?;

    for (note_id, name) in note_file.list_notes() {
        println!("{} -> {}", note_id.to_hyphenated_string(), name);
        let mut note = note_file.get_note(&note_id).expect("note should exist");

        if let Some(page_id) = note.active_pages().first().copied() {
            let mut page = note.get_page(&page_id).expect("page should exist");
            let rendered = page.render()?;
            rendered.write_png(format!("{}_{}.png", name, page_id.to_simple_string()))?;
            println!("Rendered first active page for {}", name);
        }
    }

    Ok(())
}
```

Compile/run guidance:

```bash
cargo new boox-note-playground --bin
cd boox-note-playground
```

Add this dependency in `Cargo.toml`:

```toml
[dependencies]
boox-note-parser = { path = "../boox-note-parser" }
```

Then run:

```bash
cargo run -- <path-to-note-file.note>
```

### Troubleshooting

- If you forget the note-file path, inspector exits with a usage line.
- If a note is unsupported/corrupt, parsing returns an error instead of panicking.
- Corpus integration tests are skipped unless enabled with:
  - `BOOX_NOTE_PARSER_RUN_CORPUS_TESTS=1`

## Fixture Contribution Guide

This project prefers extracted-minimal fixtures over full `.note` files.

Guidelines:

- Do not commit personal/raw full exports by default.
- Contribute only the smallest extracted blobs needed to prove parser behavior under `tests/fixtures/<topic>/`.
- Remove or anonymize sensitive names/text/identifiers in fixture content before submission.
- Add a short note in the test describing which behavior the fixture validates.

Expected wiring:

- Fixture files live under `tests/fixtures/<topic>/`.
- In-memory archive assembly and reuse helpers live in `tests/common/mod.rs`.
- Behavior-focused integration tests follow the pattern in `tests/p2_fixture_coverage.rs`.

## Quality Checks

Baseline required checks:

```bash
cargo fmt -- --check
cargo check --quiet
cargo test --quiet
```

Optional local corpus validation (requires local sample `.note` files):

```bash
BOOX_NOTE_PARSER_RUN_CORPUS_TESTS=1 cargo test --test p1_format_coverage
```

Optional stricter static analysis (non-gating for now):

```bash
cargo clippy --all-targets --all-features
```

## Corpus Notes (Current Reverse Engineering Basis)

The code and format notes are currently validated against Boox Notes App exports from a Boox Note Air 4 C (Notes app build `42842 - 0760e1b1dad`).

Observed in that corpus:

- Both single-note and multi-note layouts are present.
- UUIDs appear in both simple (32 hex chars) and hyphenated forms.
- `template/json/.template_json` appears as a per-note default template/background descriptor.
- `resource/pb` files may exist but can be empty (0 bytes).
- `toc/` directories may exist and be empty.
- Multi-note exports include note preview PNG thumbnails (`499x666`) at note-root level.

## Current Gaps

- Unknown protobuf fields and some metadata semantics still need broader cross-device validation.
- `resource/pb/*` and `toc/pb/*` payload internals are still treated as opaque/partially modeled.
- More cross-device corpus coverage is needed to confirm behavior outside current observed exports.
