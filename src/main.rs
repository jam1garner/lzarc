use structopt::StructOpt;
use std::path::PathBuf;
use std::fs;

use lzarc::LzarcFile;

use prettytable::{Table, Row, Cell, row, cell, format::{FormatBuilder, LinePosition, LineSeparator}};
use humansize::{FileSize, file_size_opts::CONVENTIONAL};

#[derive(StructOpt)]
enum Args {
    #[structopt(about = "Extract an lzarc archive to a given directory")]
    Extract {
        file: PathBuf,
        out_dir: PathBuf,
    },
    #[structopt(about = "Pack a given directory into an lzarc file")]
    Pack {
        dir: PathBuf,
        out_file: PathBuf,
    },
    #[structopt(about = "List files in a given lzarc file")]
    List {
        file: PathBuf,

        #[structopt(short, long)]
        size_bytes: bool,
    }
}

fn main() {
    match Args::from_args() {
        Args::Extract { file, out_dir } => extract(file, out_dir),
        Args::List { file, size_bytes } => list(file, size_bytes),
        Args::Pack { dir, out_file } => todo!(),
    }
}

fn extract(in_file: PathBuf, out_dir: PathBuf) {
    let lzarc = LzarcFile::open(in_file).unwrap();
    for file in lzarc.files {
        let path = out_dir.join(file.name);
        let _ = fs::create_dir_all(path.parent().unwrap());
        fs::write(path, file.data).unwrap();
    }
}

fn list(in_file: PathBuf, byte_count: bool) {
    let lzarc = LzarcFile::open(in_file).unwrap();
    let mut table = Table::new();
    let mut total_size = 0;
    table.set_titles(row![
        c->"Size", c->"Name", c->"First bytes"
    ]);
    table.set_format(
        FormatBuilder::new()
            .column_separator(' ')
            .borders(' ')

            .separators(&[
                LinePosition::Title
            ], LineSeparator::new('-', ' ', ' ', ' '))
            .build()
    );
    for file in &lzarc.files {
        let name = &file.name;
        let bytes: String = file.data[..4].iter().map(hex).collect();
        let str_bytes: String = file.data[..4].iter().map(byte_char).collect();
        let bytes = bytes + " | " + &str_bytes;
        table.add_row(row![
            size(file.data.len(), byte_count), name, bytes
        ]);
        total_size += file.data.len();
    }
    table.add_row(row![
        "--------", "", "---------------"
    ]);
    table.add_row(row![
        size(total_size, byte_count), "", format!("{} file(s)", lzarc.files.len())
    ]);
    table.printstd();
}

fn size(size: usize, byte_count: bool) -> String {
    if byte_count {
        size.to_string()
    } else {
        size.file_size(CONVENTIONAL).unwrap()
    }
}

fn hex(byte: &u8) -> String {
    format!("{:02X}", byte)
}

fn byte_char(byte: &u8) -> char {
    match *byte as char {
        c @ ' '..='~' => c,
        _ => '.'
    }
}
