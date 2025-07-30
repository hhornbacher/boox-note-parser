# boox-note-parser

**WARNING:** This crate is still WiP!

> A Rust library for parsing `.note` files from Onyx Boox e-ink devices.

[![Crates.io](https://img.shields.io/crates/v/boox-note-parser.svg)](https://crates.io/crates/boox-note-parser)
[![Docs.rs](https://docs.rs/boox-note-parser/badge.svg)](https://docs.rs/boox-note-parser)
[![License](https://img.shields.io/crates/l/boox-note-parser.svg)](https://github.com/hhornbacher/boox-note-parser/blob/main/LICENSE)

---

`boox-note-parser` provides a pure Rust implementation for reading and interpreting handwritten note data stored in `.note` files on Boox E-Ink devices.
This format is undocumented and [was reverse-engineered](docs/format.md) to allow open access to read, modify and export it's data.

The reverse engineering efforts are currently based solely on data exported from the Notes App (version 42842 - 0760e1b1dad) running on a Boox Note Air 4 C. The file format may differ on other devices or app versions. Sample file contributions are welcome.

