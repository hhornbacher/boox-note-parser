# Boox Note container file format

## Archive

The file is actually a ZIP. It can contain either one Note or multiple notes.
If it contains multiple notes, there will be a protobuf encoded file called note_tree in the root directory, which contains metadata about all notes.
If not, the metadata will be split into multiple directories containing single protobuf encoded files with this information.

All Binary Data is Big endian

# Boox Note points file format

## File Header:

| Field   | Type    | Note                                                                          |
| ------- | ------- | ----------------------------------------------------------------------------- |
| version | u32     |                                                                               |
| uuid1   | [u8;36] | UTF8, Sometimes hyphenated, sometimes condensed and padded with spaces (0x20) |
| uuid2   | [u8;36] | UTF8, Always hypehenated                                                      |

## Stroke Table:

| Field                           | Type    | Note                                                                                                                                               |
| ------------------------------- | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| Stroke UUID                     | [u8;36] | UTF8, Always hypehenated                                                                                                                           |
| Start Address                   | u32     | Address of the first point                                                                                                                         |
| Point count (31:4) / Flag (3:0) | u32     | Point count and flag at the lowest nyble. Needs more investigation, the count is calculated by masking the flag and sifting the whole 4 bits right |

## Point

| Field              | Type | Note                                    |
| ------------------ | ---- | --------------------------------------- |
| relative timestamp | u32  |                                         |
| X coordinate       | f32  |                                         |
| Y coordinate       | f32  |                                         |
| X pen tilt         | i8   | Assumption: could be pen tilt raw value |
| Y pen tilt         | i8   | Assumption: could be pen tilt raw value |
| pressure           | u16  | Stylus pressure: 0-4095                 |

## Parsing:

1. Read header
2. Read u32 from end of file (stroke table address)
3. Parse stroke table
4. Parse points for a stroke
