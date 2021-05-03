use binread::{BinReaderExt, derive_binread, NullString, io::{Read, Seek, SeekFrom}};
use std::io::BufReader;
use std::path::Path;

pub use binread::Error;
pub use binread::BinResult;

mod writer;

#[derive_binread]
#[derive(Debug, PartialEq)]
pub struct LzarcFile {
    pub file_size: u32,

    /// size when being loaded (decompressed) into a buffer where each file
    /// is 8KiB-aligned with 8KiB padding before each. Minimum 8KiB for empty archive.
    pub aligned_size: u32,

    #[br(temp)]
    file_count: u32,

    #[br(count = file_count)]
    pub files: Vec<FileEntry>,
}

#[derive_binread]
#[derive(Debug, PartialEq)]
pub struct FileEntry {
    #[br(pad_size_to = 0x80, map = NullString::into_string)]
    pub name: String,

    #[br(temp)]
    data_pos: u32,

    #[br(temp)]
    compressed_size: u32,

    /// position when being loaded (decompressed) into a buffer where each file
    /// is 8KiB-aligned with 8KiB padding before each. Minimum 8KiB for empty archive.
    #[br(temp)]
    aligned_pos: u32,

    #[br(temp)]
    uncompressed_size_plus_something: u32,

    #[br(temp)]
    uncompressed_size: u32,

    #[br(
        restore_position,
        seek_before = SeekFrom::Start(data_pos as u64),
        parse_with = binread::helpers::read_bytes,
        try_map = decompress,
        count = compressed_size
    )]
    pub data: Vec<u8>,
}

fn decompress(compressed: Vec<u8>) -> binread::io::Result<Vec<u8>> {
    Ok(rust_lzss::decompress(
        &mut binread::io::Cursor::new(&compressed[4..]),
    ).unwrap())
}

impl LzarcFile {
    pub fn open<P: AsRef<Path>>(path: P) -> BinResult<Self> {
        BufReader::new(std::fs::File::open(path)?).read_be()
    }

    pub fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_be()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let mut x = std::io::Cursor::new(std::fs::read("/home/jam/dev/sarc/Fld_TN_PostOffice_map.lzarc").unwrap());
        let lzarc: LzarcFile = x.read_be().unwrap();
    }

    #[test]
    fn double_round_trip() {
        let mut x = std::io::Cursor::new(std::fs::read("/home/jam/dev/sarc/Fld_TN_PostOffice_map.lzarc").unwrap());
        let mut lzarc: LzarcFile = x.read_be().unwrap();
        
        let mut data = Vec::new();
        lzarc.write(&mut data).unwrap();

        let lzarc2: LzarcFile = std::io::Cursor::new(data).read_be().unwrap();

        assert_eq!(lzarc.files, lzarc2.files);
    }
}
