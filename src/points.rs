use byteorder::{BE, ReadBytesExt};

use crate::{
    error::{Error, Result},
    id::{StrokeUuid, Unknown1Uuid, Unknown2Uuid},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub version: u32,
    pub uuid1: Unknown1Uuid,
    pub uuid2: Unknown2Uuid,
}

impl Header {
    pub fn read(reader: impl std::io::Read + std::io::Seek) -> Result<Self> {
        let mut reader = reader;
        reader.seek(std::io::SeekFrom::Start(0))?;

        let version = reader.read_u32::<BE>()?;

        let mut buffer = [0; 36];

        reader.read_exact(&mut buffer)?;
        let uuid1_str = str::from_utf8(&buffer).map_err(|e| Error::UuidInvalidUtf8(e))?;
        let uuid1 = Unknown1Uuid::from_str(uuid1_str)?;

        // Clear buffer for the next read
        buffer.fill(0);
        reader.read_exact(&mut buffer)?;
        let uuid2_str = str::from_utf8(&buffer).map_err(|e| Error::UuidInvalidUtf8(e))?;
        let uuid2 = Unknown2Uuid::from_str(uuid2_str)?;

        Ok(Self {
            version,
            uuid1,
            uuid2,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointsTableEntry {
    pub stroke_id: StrokeUuid,
    /// Byte offset of the first point in the file
    pub start_addr: u32,
    /// Number of points for this stroke (extracted from bits 31:4 of the packed field)
    pub point_count: u32,
    /// Lowest nibble (bits 3:0) of the packed field
    pub flag: u8,
}

impl PointsTableEntry {
    pub fn read(reader: impl std::io::Read + std::io::Seek) -> Result<Self> {
        let mut reader = reader;

        let mut buffer = [0; 36];
        reader.read_exact(&mut buffer)?;
        let stroke_uuid_str = str::from_utf8(&buffer).map_err(|e| Error::UuidInvalidUtf8(e))?;
        let stroke_uuid = StrokeUuid::from_str(stroke_uuid_str)?;

        let start_addr = reader.read_u32::<BE>()?;
        let packed = reader.read_u32::<BE>()?;

        let point_count = (packed >> 4) & 0x0FFFFFFF; // Bits 31:4
        let flag = (packed & 0xF) as u8; // Bits 3:0

        Ok(Self {
            stroke_id: stroke_uuid,
            start_addr,
            point_count,
            flag,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub timestamp_rel: u32,
    pub x: f32,
    pub y: f32,
    pub tilt_x: i8,
    pub tilt_y: i8,
    pub pressure: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub meta: PointsTableEntry,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointsFile {
    header: Header,
    stroke_table: Vec<PointsTableEntry>,
}

impl PointsFile {
    pub fn read(mut reader: impl std::io::Read + std::io::Seek) -> Result<Self> {
        let header = Header::read(&mut reader)?;

        let stroke_table_end = reader.seek(std::io::SeekFrom::End(-4))?;
        let stroke_table_start = reader.read_u32::<BE>()?;

        let mut stroke_table = Vec::new();

        reader.seek(std::io::SeekFrom::Start(stroke_table_start as u64))?;
        while reader.stream_position()? < stroke_table_end {
            let entry = PointsTableEntry::read(&mut reader)?;
            stroke_table.push(entry);
        }

        Ok(Self {
            header,
            stroke_table,
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn stroke_table(&self) -> &[PointsTableEntry] {
        &self.stroke_table
    }

    pub fn get_stroke(&self, stroke_id: &StrokeUuid) -> Option<&PointsTableEntry> {
        self.stroke_table
            .iter()
            .find(|s| s.stroke_id == *stroke_id)
    }
}
