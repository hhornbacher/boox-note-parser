# boox-note-parser

> A Rust library for parsing `.note` files from Onyx Boox e-ink devices.

[![Crates.io](https://img.shields.io/crates/v/boox-note-parser.svg)](https://crates.io/crates/boox-note-parser)
[![Docs.rs](https://docs.rs/boox-note-parser/badge.svg)](https://docs.rs/boox-note-parser)
[![License](https://img.shields.io/crates/l/boox-note-parser.svg)](https://github.com/hhornbacher/boox-note-parser/blob/main/LICENSE)

---

`boox-note-parser` provides a pure Rust implementation for reading and interpreting handwritten note data stored in `.note` files on Boox devices.  
This format is undocumented and [was reverse-engineered](docs/format.md) to allow open access to user-created content.

