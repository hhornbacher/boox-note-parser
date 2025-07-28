use std::collections::HashMap;

use byteorder::{BE, ReadBytesExt};
use raqote::{DrawOptions, DrawTarget, PathBuilder, Source, StrokeStyle};

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
    pub points: Vec<Point>,
}

impl Stroke {
    pub fn read(
        reader: impl std::io::Read + std::io::Seek,
        entry: &PointsTableEntry,
    ) -> Result<Self> {
        let mut reader = reader;

        reader.seek(std::io::SeekFrom::Start(entry.start_addr as u64))?;

        let mut points = Vec::with_capacity(entry.point_count as usize);
        for _ in 0..entry.point_count {
            let timestamp_rel = reader.read_u32::<BE>()?;
            let x = reader.read_f32::<BE>()?;
            let y = reader.read_f32::<BE>()?;
            let tilt_x = reader.read_i8()?;
            let tilt_y = reader.read_i8()?;
            let pressure = reader.read_u16::<BE>()?;

            points.push(Point {
                timestamp_rel,
                x,
                y,
                tilt_x,
                tilt_y,
                pressure,
            });
        }

        Ok(Self { points })
    }

    pub fn render(
        &self,
        draw_target: &mut DrawTarget,
        draw_options: &DrawOptions,
        stroke_style: &StrokeStyle,
    ) -> Result<()> {
        if self.points.is_empty() {
            log::warn!("No points to draw for stroke");
            return Ok(());
        }

        let mut path = PathBuilder::new();
        let mut first_point = true;

        for point in &self.points {
            if first_point {
                path.move_to(point.x, point.y);
                first_point = false;
            } else {
                path.line_to(point.x, point.y);
            }
        }

        draw_target.stroke(
            &path.finish(),
            &Source::Solid(raqote::Color::new(255, 0, 0, 0).into()),
            stroke_style,
            draw_options,
        );

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointsFile {
    header: Header,
    points: HashMap<StrokeUuid, Stroke>,
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

        let mut points = HashMap::new();
        for entry in points_table {
            let stroke = Stroke::read(&mut reader, &entry)?;
            points.insert(entry.stroke_id, stroke);
        }

        Ok(Self { header, points })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn get_stroke(&self, stroke_id: &StrokeUuid) -> Option<&Stroke> {
        self.points.get(stroke_id)
    }
}
