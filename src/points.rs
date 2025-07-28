use byteorder::{BE, ReadBytesExt};

use crate::{
    error::{Error, Result},
    id::{PageUuid, PointsUuid, StrokeUuid},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub version: u32,
    pub page_id: PageUuid,
    pub points_id: PointsUuid,
}

impl Header {
    pub fn read(reader: impl std::io::Read + std::io::Seek) -> Result<Self> {
        let mut reader = reader;
        reader.seek(std::io::SeekFrom::Start(0))?;

        let version = reader.read_u32::<BE>()?;

        let mut buffer = [0; 36];

        reader.read_exact(&mut buffer)?;
        let page_id_str = str::from_utf8(&buffer).map_err(|e| Error::UuidInvalidUtf8(e))?;
        let page_id = PageUuid::from_str(page_id_str.trim())?;

        // Clear buffer for the next read
        buffer.fill(0);
        reader.read_exact(&mut buffer)?;
        let points_id_str = str::from_utf8(&buffer).map_err(|e| Error::UuidInvalidUtf8(e))?;
        let points_id = PointsUuid::from_str(points_id_str)?;

        Ok(Self {
            version,
            page_id,
            points_id,
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
    points_table: Vec<PointsTableEntry>,
}

impl PointsFile {
    pub fn read(mut reader: impl std::io::Read + std::io::Seek) -> Result<Self> {
        let header = Header::read(&mut reader)?;

        let points_table_end = reader.seek(std::io::SeekFrom::End(-4))?;
        let points_table_start = reader.read_u32::<BE>()?;

        let mut points_table = Vec::new();

        reader.seek(std::io::SeekFrom::Start(points_table_start as u64))?;
        while reader.stream_position()? < points_table_end {
            let entry = PointsTableEntry::read(&mut reader)?;
            points_table.push(entry);
        }

        Ok(Self {
            header,
            points_table,
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn points_table(&self) -> &[PointsTableEntry] {
        &self.points_table
    }

    pub fn get_points(&self, stroke_id: &StrokeUuid) -> Option<&PointsTableEntry> {
        self.points_table.iter().find(|s| s.stroke_id == *stroke_id)
    }
}
