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
- Render page strokes to PNG using `raqote`.

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

- Some parser paths still use `expect(...)` and can panic on malformed data.
- `extra/pb`, `resource/pb`, `template/json`, `document/`, `toc/`, and preview PNG metadata are not modeled yet.
- There is no fixture-driven integration test suite yet.

See `TODO.md` for a prioritized task list.
