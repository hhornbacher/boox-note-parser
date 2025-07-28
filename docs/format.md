# Boox Note File Format

This document describes the `.note` file format used by Onyx Boox e-ink devices, based solely on reverse engineering and the implementation in this repository.

---

## Archive Structure

A `.note` file is a ZIP archive. It can contain either a single note or multiple notes.

- **Multi-note archive**:  
  Contains a file named `note_tree` in the root directory. This file is encoded using Protocol Buffers and contains metadata for all notes in the archive. See the implementation in [`src/note_tree.rs`](../src/note_tree.rs).

- **Single-note archive**:  
  Contains a file in `note/pb/note_info` with the metadata of the note. See the implementation in [`src/note_tree.rs`](../src/note_tree.rs).

---

## File Hierarchy

The file structure is hierarchical and differs slightly between single-note and multi-note archives.

### Multi-note Archive

```
<archive>.note
├── note_tree                  # Protobuf: metadata for all notes
├── <note_id>/                 # One directory per note
│   ├── pageModel/pb/          # Protobuf: page model files
│   ├── virtual/doc/pb/        # Protobuf: virtual document files
│   ├── virtual/page/pb/       # Protobuf: virtual page files
│   ├── shape/                 # Shape group files (protobuf)
│   ├── point/                 # Handwriting stroke data (custom binary)
│   ├── resource/              # Resources
│   └── template/              # Templates
```

### Single-note Archive

```
<archive>.note
├── note/pb/note_info          # Protobuf: metadata for the note
├── pageModel/pb/              # Protobuf: page model files
├── virtual/doc/pb/            # Protobuf: virtual document files
├── virtual/page/pb/           # Protobuf: virtual page files
├── shape/                     # Shape group files (protobuf)
├── point/                     # Handwriting stroke data (custom binary)
├── document/                  # Additional document data
├── extra/                     # Additional data
├── resource/                  # Resources
└── template/                  # Templates
```

---

## Serialization Types

**All binary data is stored in big-endian format.**

The format uses several serialization methods:

- **Protocol Buffers (protobuf):**  
  Used for most metadata files, such as `note_tree`, `note_info`, page models, virtual documents/pages, and shape groups.

- **JSON:**  
  Some fields within protobuf structures (e.g., pen settings, canvas state) are stored as JSON strings and must be parsed separately.

- **Custom Binary:**  
  Points data (handwriting strokes) is stored in a custom binary format, described below. See the implementation in [`src/points.rs`](../src/points.rs).

---

## Conceptual Layers

The format is organized into several conceptual layers, each represented by specific files and structures:

- [**NoteTree**](../src/note_tree.rs)  
  Top-level metadata for all notes in a multi-note archive, mapping note IDs to their corresponding `NoteMetadata`.  
  In single-note archives, this structure contains only one note but retains the same format.

- [**NoteMetadata**](../src/note_tree.rs)
  Metadata for a single note, including creation/modification times, name, pen settings, canvas state, background configuration, device info, and lists of page UUIDs (active, reserved, detached).

- [**PageModel**](../src/page_model.rs)
  Describes the structure and properties of a page, including dimensions and layer arrangement.

- **[VirtualDoc](../src/virtual_doc.rs) / [VirtualPage](../src/virtual_page.rs)**  
  Represent virtualized versions of documents and pages, with their own metadata and content.

- [**ShapeGroup**](../src/shape.rs)
  Contains groups of shapes (e.g. strokes) for a page, stored as protobuf.

- [**PointsFile**](../src/points.rs)
  Contains the actual handwritten stroke data, organized by groups and stored in a custom binary format.

---

## Boox Note Points File Format

### File Header

| Field   | Type    | Note                                                                          |
| ------- | ------- | ----------------------------------------------------------------------------- |
| version | u32     |                                                                               |
| uuid1   | [u8;36] | UTF8, sometimes hyphenated, sometimes condensed and padded with spaces (0x20) |
| uuid2   | [u8;36] | UTF8, always hyphenated                                                       |

### Stroke Table

| Field                           | Type    | Note                                                                                                                      |
| ------------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------- |
| Stroke UUID                     | [u8;36] | UTF8, always hyphenated                                                                                                   |
| Start Address                   | u32     | Address of the first point                                                                                                |
| Point count (31:4) / Flag (3:0) | u32     | Point count (upper 28 bits), flag (lower 4 bits). Point count is calculated by masking out the lower 4 bits and shifting. |

### Point

| Field              | Type | Note                                    |
| ------------------ | ---- | --------------------------------------- |
| relative timestamp | u32  |                                         |
| X coordinate       | f32  |                                         |
| Y coordinate       | f32  |                                         |
| X pen tilt         | i8   | Assumption: could be pen tilt raw value |
| Y pen tilt         | i8   | Assumption: could be pen tilt raw value |
| pressure           | u16  | Stylus pressure: 0-4095                 |

---

## Parsing Process

1. **Read the file header** to get version and UUIDs.
2. **Read a u32 from the end of the file** to get the stroke table address.
3. **Parse the stroke table** to get stroke UUIDs, start addresses, and point counts/flags.
4. **Parse points for each stroke** using the addresses and counts from the stroke table.

---

## Summary

- The `.note` format is a ZIP archive containing protobuf, JSON, and custom binary data.
- Metadata is organized hierarchically: archive → note tree → note metadata → pages → shapes → strokes/points.
- Handwriting data is stored in a custom binary format with a header, stroke table, and point data.
- The format supports both single-note and multi-note archives, with different directory structures.

**All details above are based solely on reverse engineering efforts, and may not represent the official format.**
