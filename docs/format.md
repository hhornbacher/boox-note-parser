# Boox Note File Format

This document describes the `.note` format used by Onyx Boox devices, based on reverse engineering.

All details are best-effort and may vary across device models and Notes app versions.

## Scope and Corpus

Current understanding is based on:

- Boox Note Air 4 C exports (Notes app build `42842 - 0760e1b1dad`)

## Archive Types

A `.note` file is a ZIP archive. Two main layouts are observed:

- **Multi-note archive**
  Contains a top-level `note_tree` protobuf and one directory per note.
- **Single-note archive**
  Contains a note root directory with `note/pb/note_info` protobuf.

## Observed Directory Layouts

### Multi-note archive (observed)

```
<archive>.note
├── note_tree
├── <note_id>/
│   ├── <note_id>.png              # preview thumbnail (observed 499x666)
│   ├── pageModel/pb/
│   ├── point/
│   ├── shape/
│   ├── template/json/
│   ├── toc/                       # may be empty
│   └── virtual/
│       ├── doc/pb/
│       └── page/pb/
```

### Single-note archive (observed)

```
<archive>.note
└── <note_id>/
    ├── note/pb/note_info
    ├── pageModel/pb/
    ├── point/
    ├── shape/
    ├── template/json/
    ├── virtual/
    │   ├── doc/pb/
    │   └── page/pb/
    ├── document/<note_id>/template/json/   # mirrored template path
    ├── extra/pb/extra
    └── resource/pb/
```

## Serialization and Encoding

- **Protocol Buffers:** most metadata files (`note_tree`, `note_info`, page/virtual/shape containers)
- **JSON strings inside protobuf fields:** pen settings, canvas state, background settings, dimensions, etc.
- **Custom binary points files:** stroke point data (documented below)

Important nuance:

- Protobuf data is varint-based wire format (not fixed big-endian structs).
- The custom points binary format uses **big-endian** numeric fields.

## Naming Conventions Seen in Files

- Shape groups:
  `shape/<page_id>#<shape_group_id>#<timestamp>.zip`
- Points data:
  `point/<page_id>/<page_id>#<points_id>#points`
- Template descriptors:
  `template/json/<page_or_template_id>.template_json`
- Per-note default template descriptor:
  `template/json/.template_json`
- Resource protobuf-like artifacts:
  `resource/pb/<resource_id>#<timestamp>`

Notes from samples:

- IDs appear in both simple (`32` hex chars) and hyphenated UUID forms.
- A note can have virtual pages but no shapes/points yet (blank pages).

## Conceptual Layers

- [**NoteTree / NoteMetadata**](../src/note_tree.rs)
  Top-level note metadata, including note name, page lists, pen settings, and background config.
- [**PageModel**](../src/page_model.rs)
  Page geometry/layers, timestamp metadata.
- [**VirtualDoc / VirtualPage**](../src/virtual_doc.rs), [virtual_page.rs](../src/virtual_page.rs)
  Document/page relationship and template/layout-related info.
- [**ShapeGroup**](../src/shape.rs)
  Per-group shape metadata (stroke IDs, bbox, style metadata).
- [**PointsFile**](../src/points.rs)
  Raw stroke points data.

## Parser Coverage in This Crate

Implemented:

- `note_tree` / `note/pb/note_info`
- `pageModel/pb/*`
- `virtual/doc/pb/*`
- `virtual/page/pb/*`
- `shape/*.zip` (embedded protobuf)
- `point/...#points`

Not modeled yet:

- `extra/pb/extra`
- `resource/pb/*`
- `toc/`
- preview PNG metadata
- explicit typed model for `template/json/*.template_json`

## Custom Points File Format

Implementation: [`src/points.rs`](../src/points.rs)

### Header

| Field     | Type     | Notes                                                |
| --------- | -------- | ---------------------------------------------------- |
| version   | u32 (BE) | observed value `1`                                   |
| page_id   | [u8;36]  | UTF-8 UUID, sometimes simple UUID padded with spaces |
| points_id | [u8;36]  | UTF-8 UUID, typically hyphenated                     |

### Stroke Table

| Field             | Type     | Notes                                            |
| ----------------- | -------- | ------------------------------------------------ |
| stroke_id         | [u8;36]  | UTF-8 UUID                                       |
| start_addr        | u32 (BE) | byte offset of first point                       |
| packed_count_flag | u32 (BE) | upper 28 bits = point count, lower 4 bits = flag |

### Point Record

| Field         | Type     | Notes                              |
| ------------- | -------- | ---------------------------------- |
| timestamp_rel | u32 (BE) | relative timestamp                 |
| x             | f32 (BE) | x coordinate                       |
| y             | f32 (BE) | y coordinate                       |
| tilt_x        | i8       | likely stylus tilt x               |
| tilt_y        | i8       | likely stylus tilt y               |
| pressure      | u16 (BE) | stylus pressure (observed 0..4095) |

### Parsing Flow

1. Read header from file start.
2. Read last `u32` in file to get stroke-table start offset.
3. Parse table entries until end marker region.
4. Seek to each `start_addr` and read `point_count` point records.

## Sample-Corpus Observations Worth Tracking

- `resource/pb/*` can exist as empty files.
- `toc/` directories are present in multi-note exports and may be empty.
- `pageModel/pb` file count is lower than virtual-page count in larger notes (containers can hold multiple page models).
- `render_scale_json` inside shapes may include extra keys such as `tiltConfig` not modeled in strongly typed structs.
- Background labels are localized (for example `Blank`, `Leer`), so consumer logic should not hardcode English titles.

## Open Questions

- Exact semantics of several protobuf fields currently labeled `unknown`.
- Full schema of `extra/pb/extra`.
- Full schema and lifecycle of `resource/pb/*`.
- Meaning and lifecycle of `toc/` content.
- Semantics of points-table low nibble `flag`.
