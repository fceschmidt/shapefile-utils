//! Module for SHX files
//!
//! These files are basically index files for the SHP files: They contain, in ascending order, all
//! the entries that can be found in the SHP file. Just a simple index.

use std::io::{Error, ErrorKind, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::fs::File;
use byteorder::{BigEndian, ReadBytesExt};
use super::{FileHeader, ShxFile};

/// An index record.
#[derive(Debug)]
pub struct Record {
    /// Offset of the SHP file record measured in 16-bit words
    pub offset: i32,
    /// Length of the SHP file record measured in 16-bit words
    pub length: i32,
}

impl Record {
    /// Constructs a zero-initialized Record
    pub fn new() -> Record {
        Record {
            offset: 0,
            length: 0,
        }
    }

    /// Reads a record from the binary input stream
    /// Consumes 8 bytes from the stream.
    pub fn parse<T: Read>(file: &mut T) -> Result<Record, Error> {
        let mut result = Record::new();

        // Read the header fields -- First: offset, Big Endian
        result.offset = try!(file.read_i32::<BigEndian>());

        // Second: Content Length, Big Endian
        result.length = try!(file.read_i32::<BigEndian>());

        Ok(result)
    }
}

impl ShxFile {
    /// Parses the SHX file from the supplied input stream
    fn parse_header(mut self) -> Result<Self, Error> {
        try!(self.file.seek(SeekFrom::Start(0)));

        // Try parsing the header
        self.header = try!(FileHeader::parse(&mut self.file));

        Ok(self)
    }

    /// Given a file name, parses the SHX file and returns the result.
    pub fn parse_file(path: &Path) -> Result<Self, Error> {
        let result = ShxFile {file: BufReader::new(try!(File::open(path))), header: FileHeader::new()};

        // Check file header is actually there before attempting any reads
        match result.file.get_ref().metadata() {
            Ok(m) => {
                if m.len() < 100 {
                    return Err(Error::new(ErrorKind::Other, "SHX file has invalid size!"));
                }
            },
            Err(e) => {
                return Err(e);
            }
        }

        // Parse the data
        return result.parse_header();
    }

    /// Returns a record with the given ID.
    ///
    /// This record contains the offset and the length of the SHP file entry in 16-bit words.
    pub fn record(&mut self, id: u64) -> Option<Record> {
        let header_size = 100u64;
        let record_size = 8u64;
        let record_count = self.num_records();

        // Check overflow
        if id > record_count || id < 1 {
            return None;
        }

        let record_pos = header_size + (id - 1u64) * record_size;

        match self.file.seek(SeekFrom::Start(record_pos)) {
            Ok(p) => {
                if p != record_pos {
                    return None;
                }
            },
            Err(_) => return None,
        }

        match Record::parse(&mut self.file) {
            Ok(v) => return Some(v),
            Err(_) => return None,
        }
    }

    /// Gets the amount of records listed in the index file.
    pub fn num_records(&self) -> u64 {
        let file_size = self.header.file_length as u64 * 2u64;
        let header_size = 100u64;
        let record_size = 8u64;

        (file_size - header_size) as u64 / record_size
    }
}
