//! Module for Shapefiles
//!
//! The structure of Shapefiles is described in http://www.esri.com/library/whitepapers/pdfs/shapefile.pdf.
//! This takes all the parts and puts them together.
//!

use super::{Shapefile, ShpFile, DbfFile, ShxFile};
use super::shpfile::Shape;
use std::collections::HashMap;
use dbf::Field;
use std::io::Error;
use std::path::Path;

/// Represents a record in the shapefile - has shape and metadata.
#[derive(Debug)]
pub struct Record {
    /// The shape as defined in the SHP file.
    pub shape: Shape,
    /// The metadata as it comes from the DBF file.
    pub metadata: HashMap<String, Field>
}

impl Shapefile {
    /// Creates a new `Shapefile` instance by taking all three files specified in the spec.
    pub fn new(shp_path: &Path, shx_path: &Path, dbf_path: &Path) -> Result<Self, Error> {
        Ok(Shapefile {
            shp_file: try!(ShpFile::parse_file(shp_path)),
            shx_file: try!(ShxFile::parse_file(shx_path)),
            dbf_file: try!(DbfFile::parse_file(dbf_path)),
            id: 1u64,
        })
    }

    /// Gives the data behind the record number
    pub fn record(&mut self, id: u64) -> Option<Record> {
        let mut result = Record {shape: Shape::new(), metadata: HashMap::new()};

        match self.shp_file.record(&mut self.shx_file, id) {
            Some(r) => result.shape = r.shape,
            None => return None,
        }

        match self.dbf_file.record(id as u32 - 1) {
            Some(r) => result.metadata = r,
            None => return None,
        }

        Some(result)
    }

    /// The amount of records in the file.
    pub fn num_records(&self) -> u64 {
        self.shx_file.num_records()
    }
}

impl Iterator for Shapefile {
    type Item = Record;

    fn next(&mut self) -> Option<Record> {
        let id = self.id;
        let result = self.record(id);
        self.id += 1u64;
        result
    }
}
