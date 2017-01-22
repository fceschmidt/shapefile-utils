//! Module for DBF files
//!
//! Internally uses the implementation from the `dbf` crate.

use std::path::Path;
use std::io::Error;

use dbf;
use super::DbfFile;

impl DbfFile {
    /// Given a file name, parses the DBF file and returns the result.
    pub fn parse_file(path: &Path) -> Result<DbfFile, Error> {
        let result = DbfFile{ file: dbf::DbfFile::open_file(path) };
        Ok(result)
    }

    /// Get the record with the given ID.
    pub fn record(&mut self, id: u32) -> Option<dbf::Record> {
        self.file.record(id)
    }
}
