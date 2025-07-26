# boox-note

> A Rust library for parsing `.note` files from Onyx Boox e-ink devices.

[![Crates.io](https://img.shields.io/crates/v/boox-note.svg)](https://crates.io/crates/boox-note)
[![Docs.rs](https://docs.rs/boox-note/badge.svg)](https://docs.rs/boox-note)
[![CI](https://github.com/yourusername/boox-note/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/boox-note/actions)
[![License](https://img.shields.io/crates/l/boox-note.svg)](https://github.com/yourusername/boox-note/blob/main/LICENSE)

---

`boox-note` provides a pure Rust implementation for reading and interpreting handwritten note data stored in `.note` files on Boox devices.  
This format is undocumented and [was reverse-engineered](docs/format.md) to allow open access to user-created content.

