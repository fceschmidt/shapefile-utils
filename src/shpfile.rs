//! Module for SHP files
//!
//! The structure of SHP files is described in http://www.esri.com/library/whitepapers/pdfs/shapefile.pdf,
//! page 2. The struct and field names try to follow the specification, where possible.
//!
//! SHP files contain an arbitrary number of geometric data records. They are all of the same type.
//!

use std::fs::File;
use std::io::{Error, ErrorKind, BufReader, Read, SeekFrom, Seek};
use std::path::Path;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

use super::{ShpFile, ShxFile, ShxRecord, FileHeader, BoundingBoxZ};
use super::shape::*;

/// One of multiple geometric data records in a SHP file.
#[derive(Debug, PartialEq)]
pub struct Record {
    /// The ID of the record (starting at 1)
    pub record_number: i32,
    /// Length of the record contents section in 16-bit words
    content_length: i32,
    /// The shape
    pub shape: Shape,
}

impl Record {
    /// Constructs a zero-initialized Record
    pub fn new() -> Record {
        Record {
            record_number: 0,
            content_length: 0,
            shape: Shape::NullShape,
        }
    }

    /// Reads a record from the binary input stream
    pub fn parse<T: Read>(file: &mut T) -> Result<(Record, usize), Error> {
        let mut result = Record::new();
        let mut read = 0usize;

        // Read the header fields -- First: Record number, Big Endian
        result.record_number = try!(file.read_i32::<BigEndian>());
        read += 4usize;

        // Second: Content Length, Big Endian
        result.content_length = try!(file.read_i32::<BigEndian>());
        read += 4usize;

        // Third: Actual shape
        let (shape, shape_length) = try!(Shape::parse(file));
        result.shape = shape;

        Ok((result, read + shape_length))
    }
}

impl FileHeader {
    /// The magic number at the beginning of SHP files
    const SHP_MAGIC_NUMBER: i32 = 9994;
    /// The supported version
    const SHP_VERSION: i32 = 1000;

    /// Creates a new empty file header
    pub fn new() -> Self {
        FileHeader {file_length: 0, shape_type: 0, bounding_box: BoundingBoxZ::new()}
    }

    /// Reads a file header from the given input stream
    pub fn parse<T: Read + Seek>(file: &mut T) -> Result<Self, Error> {
        // Confirm magic number - Big Endian
        if try!(file.read_i32::<BigEndian>()) != Self::SHP_MAGIC_NUMBER {
            return Err(Error::new(ErrorKind::Other, "SHP header magic number mismatch!"));
        }

        let mut result = Self::new();

        // Take 20 bytes away, since they are unused according to the spec.
        match file.seek(SeekFrom::Current(20)) {
            Err(e) => {
                return Err(e)
            },
            Ok(n) => {
                if n < 20 {
                    return Err(Error::new(ErrorKind::Other, "SHP header too short!"));
                }
            }
        }

        // Read file length - Big Endian
        result.file_length = try!(file.read_i32::<BigEndian>());

        // Read version - Little Endian
        if try!(file.read_i32::<LittleEndian>()) != Self::SHP_VERSION {
            return Err(Error::new(ErrorKind::Other, "SHP header version mismatch!"));
        }

        // Read shape type - Little Endian
        result.shape_type = try!(file.read_i32::<LittleEndian>());

        // Read bounding box
        result.bounding_box = try!(BoundingBoxZ::parse(file));

        // Return our result
        Ok(result)
    }
}

impl ShpFile {
    pub fn parse_header(mut self) -> Result<Self, Error> {
        try!(self.file.seek(SeekFrom::Start(0)));

        // Try parsing the header
        self.header = try!(FileHeader::parse(&mut self.file));

        Ok(self)
    }

    /// Given a file name, parses the SHP file and returns the result.
    pub fn parse_file(path: &Path) -> Result<Self, Error> {
        let result = ShpFile {
            file: BufReader::new(try!(File::open(path))),
            header: FileHeader::new()
        };

        // Check file header is actually there before attempting any reads
        match result.file.get_ref().metadata() {
            Ok(m) => {
                if m.len() < 100 {
                    return Err(Error::new(ErrorKind::Other, "SHP file has invalid size!"));
                }
            },
            Err(e) => {
                return Err(e);
            }
        }

        return result.parse_header();
    }

    pub fn record(&mut self, shx_file: &mut ShxFile, id: u64) -> Option<Record> {
        let rec: ShxRecord;
        match shx_file.record(id) {
            Some(r) => rec = r,
            None => return None,
        }

        match self.file.seek(SeekFrom::Start(rec.offset as u64 * 2u64)) {
            Ok(p) => {
                if p != rec.offset as u64 * 2u64 {
                    return None;
                }
            },
            Err(_) => return None,
        }

        match Record::parse(&mut self.file) {
            Ok((v,_)) => return Some(v),
            Err(_) => return None,
        }
    }
}
