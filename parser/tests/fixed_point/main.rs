use std::io::{Cursor, Read, Seek};

use fixed::types::I16F16;
use glam::Vec3;
use nomenclature::prelude::*;

const DATA: &[u8; 0x40] = include_bytes!("fixed_point.bin");
const UNITS_PER_METER: f32 = 32.0;

struct I16F16Vec3;

impl BinReader for I16F16Vec3 {
    type Args<'a> = ();
    type Out = Vec3;

    fn reader<'a, Reader>() -> impl BinRead<Reader, Self::Args<'a>, Self::Out>
    where
        Reader: Read + Seek,
    {
        |reader: &mut Reader, endian, _| {
            i32::reader()
                .map(|v| I16F16::from_bits(v).to_num::<f32>())
                .repeat_array::<3>()
                .map(Vec3::from_array)
                .read(reader, endian, ())
        }
    }
}

#[derive(Debug, PartialEq)]
struct Cloud {
    points: Vec<Vec3>,
}

impl BinReader for Cloud {
    type Args<'a> = ();
    type Out = Self;

    fn reader<'a, Reader>() -> impl BinRead<Reader, Self::Args<'a>, Self::Out>
    where
        Reader: Read + Seek,
    {
        |reader: &mut Reader, endian, _| {
            let num_points = u32::reader().read(reader, endian, ())?;

            let scale = I16F16Vec3::reader().read(reader, endian, ())?;
            let offset = I16F16Vec3::reader().pad_after(4).read(reader, endian, ())?;

            let points = u8::reader()
                .map(|v| v as f32 / u8::MAX as f32)
                .repeat_array::<3>()
                .map(Vec3::from_array)
                .map(|v| (v * scale + offset) / UNITS_PER_METER)
                .repeat_vec(num_points as usize)
                .read(reader, endian, ())?;

            Ok(Self { points })
        }
    }
}

#[test]
fn main() {
    let mut data = Cursor::new(DATA);
    let result = Cloud::reader().read(&mut data, Endian::Big, ());

    assert_eq!(
        result,
        Ok(Cloud {
            points: vec![
                Vec3::new(-0.127_573_52_, 0.112_573_616, -0.013_125_027),
                Vec3::new(-0.118_933_83_, 0.051_495_12_, -0.013_166_693),
                Vec3::new(-0.109_375____, 0.117_720_686, -0.014_187_516),
                Vec3::new(-0.122_426_465, 0.077_573_58_, -0.012_708_364),
                Vec3::new(-0.115_625____, 0.073_455_93_, -0.012_416_701),
                Vec3::new(-0.112_316_18_, 0.117_034_405, -0.015_062_506),
                Vec3::new(-0.133_823_53_, 0.081_004_96_, -0.011_000_05_),
                Vec3::new(-0.122_058_82_, 0.042_230_405, -0.014_937_507),
                Vec3::new(-0.114_705_88_, 0.039_828_442, -0.014_416_68_),
                Vec3::new(-0.151_102_95_, 0.073_112_786, -0.011_062_549)
            ]
        })
    )
}
