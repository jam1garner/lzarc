use crate::LzarcFile;
use binwrite::BinWrite;

use std::io::{Write, BufWriter, self};
use std::convert::TryInto;
use std::path::Path;
use std::fs::File;

#[derive(BinWrite, Debug)]
#[binwrite(big)]
pub struct Header {
    file_size: u32,
    padded_size: u32,
    file_count: u32,
}

impl Header {
    const SIZE: usize = 0xc;
}

#[derive(BinWrite, Debug)]
#[binwrite(big)]
pub struct EntryWriter {
    #[binwrite(preprocessor(to_char128))]
    name: String,

    pos: u32,
    compressed_size: u32,
    aligned_pos: u32,
    uncompressed_size_plus_something: u32,
    uncompressed_size: u32,
}

impl EntryWriter {
    const SIZE: usize = 0x94;
}

fn to_char128(string: &String) -> Vec<u8> {
    let mut bytes = string.as_bytes().to_owned();
    bytes.extend(vec![0u8; 0x80 - bytes.len()].into_iter());
    bytes
}

// NOTE: `to` must be a power of 2
fn align(x: usize, to: usize) -> usize {
    (x + (to - 1)) & (!(to - 1))
}

fn as_u24_le(x: usize) -> [u8; 3] {
    (x as u32).to_le_bytes()[..3].try_into().unwrap()
}

const ALIGNMENT: usize = 0x2000;

impl LzarcFile {
    pub fn save<P: AsRef<Path>>(&mut self, filename: P) -> io::Result<()> {
        self.write(&mut BufWriter::new(File::create(filename)?))
    }

    pub fn write<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.files.sort_by_key(|file| file.data.len());

        let end_of_table = Header::SIZE + (EntryWriter::SIZE * self.files.len());
        let start_of_data = align(end_of_table, 0x40);

        let mut current_pos = start_of_data as u32;
        let mut current_aligned_pos = ALIGNMENT;
        let mut datas = Vec::new();

        let files: Vec<_> = self.files.iter()
            .map(|file| {
                let mut compressed = vec![0x13];
                compressed.write_all(&as_u24_le(file.data.len())).unwrap();
                rust_lzss::lazy_encode_lzss11(&file.data, &mut compressed).unwrap();

                let compressed_size = compressed.len() as u32;
                let padding = align(compressed.len(), 0x40) - compressed.len();
                compressed.extend(vec![0; padding]);
                datas.push(compressed);

                let pos = current_pos;
                let aligned_pos = current_aligned_pos as u32;

                current_pos += compressed_size + (padding as u32);
                current_aligned_pos += align(file.data.len() + ALIGNMENT, ALIGNMENT);

                EntryWriter {
                    name: file.name.clone(),
                    pos,
                    compressed_size,
                    aligned_pos,
                    uncompressed_size_plus_something: file.data.len() as u32,
                    uncompressed_size: file.data.len() as u32,
                }
            })
            .collect();

        (
            Header {
                file_size: current_pos,
                padded_size: current_aligned_pos as u32,
                file_count: self.files.len() as u32,
            },
            files,
            vec![0u8; start_of_data - end_of_table],
            datas
        ).write(writer)
    }
}
