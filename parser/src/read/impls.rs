// use std::{
//     io::{Read, Seek},
//     num::NonZeroI32,
// };
// 
// use super::BinRead;
// use crate::utils::{endian::Endian, error::BinResult};
// 
// impl<T, GlobalR> BinRead for T
// where
//     T: Fn(),
//     GlobalR: Read + Seek
// {
//     type Args<'a>;
// 
//     type Out;
// 
//     fn bin_read_non_backtracking<R: Read + Seek>(
//         reader: &mut R,
//         endian: Endian,
//         args: Self::Args<'_>,
//     ) -> BinResult<Self::Out> {
// 
//     }
// }
// 
// macro_rules! impl_binread_primitive {
//     ($($type:ty),* $(,)*) => {
//         $(
//             impl BinRead for $type {
//                 type Args<'a> = ();
// 
//                 type Out = Self;
// 
//                 fn bin_read_non_backtracking<R: Read + Seek>(
//                     reader: &mut R,
//                     endian: Endian,
//                     _: Self::Args<'_>,
//                 ) -> BinResult<Self::Out> {
//                     let mut buf = [0u8; size_of::<$type>()];
//                     reader.read_exact(&mut buf)?;
//                     Ok(match endian {
//                         Endian::Big => <$type>::from_be_bytes(buf),
//                         Endian::Little => <$type>::from_le_bytes(buf),
//                     })
//                 }
//             }
//         )*
//     };
// }
// 
// #[rustfmt::skip]
// impl_binread_primitive!(
//     u8, u16, u32, u64, u128,
//     i8, i16, i32, i64, i128,
//     f32, f64,
// );
// 
// impl BinRead for NonZeroI32 {
//     type Args<'a> = ();
// 
//     type Out = Self;
// 
//     fn bin_read_non_backtracking<R: Read + Seek>(
//         reader: &mut R,
//         endian: Endian,
//         _: Self::Args<'_>,
//     ) -> BinResult<Self::Out> {
//         i32::bin_read(reader, endian, ());
//         todo!()
//     }
// }
