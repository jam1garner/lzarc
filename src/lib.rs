use binread::{NullString, derive_binread, PosValue, io::SeekFrom};

#[derive_binread]
#[derive(Debug)]
struct LzarcFile {
    file_size: u32,
    unk: u32,

    #[br(temp)]
    file_count: u32,

    #[br(count = file_count)]
    files: Vec<FileEntry>,
}

#[derive_binread]
#[derive(Debug)]
struct FileEntry {
    #[br(pad_size_to = 0x80, map = NullString::into_string)]
    name: String,

    #[br(temp)]
    data_pos: u32,

    #[br(temp)]
    compressed_size: u32,

    #[br(temp)]
    unk: u32,

    #[br(temp)]
    uncompressed_size: u32,

    #[br(temp, assert(uncompressed_size == uncompressed_size2))]
    uncompressed_size2: u32,

    #[br(
        restore_position,
        seek_before = SeekFrom::Start(data_pos as u64),
        try_map = decompress,
        count = compressed_size
    )]
    data: Vec<u8>,
}

fn decompress(compressed: Vec<u8>) -> binread::io::Result<Vec<u8>> {
    rust_lzss::decompress(
        &mut binread::io::Cursor::new(&compressed[4..]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use binread::BinReaderExt;

    #[test]
    fn parse() {
        let mut x = std::io::Cursor::new(std::fs::read("/home/jam/dev/sarc/Fld_TN_PostOffice_map.lzarc").unwrap());
        let lzarc: LzarcFile = x.read_be().unwrap();

        for file in lzarc.files {
            if file.data.is_empty() {
                println!("{}", file.name);
            }
        }
    }
}
