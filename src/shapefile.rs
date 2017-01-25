//! Module for Shapefiles
//!
//! The structure of Shapefiles is described in http://www.esri.com/library/whitepapers/pdfs/shapefile.pdf.
//! This takes all the parts and puts them together.
//!

use std::collections::HashMap;
use std::io::Error;
use std::iter::Iterator;
use std::path::Path;

use super::{Shapefile, ShapefileRecord, ShpFile, DbfFile, ShxFile, ShapefileRecordIterator};
use super::shape::Shape;

impl Shapefile {
    /// Creates a new `Shapefile` instance by taking all three files specified in the spec.
    pub fn new(shp_path: &Path, shx_path: &Path, dbf_path: &Path) -> Result<Self, Error> {
        Ok(Shapefile {
            shp_file: try!(ShpFile::parse_file(shp_path)),
            shx_file: try!(ShxFile::parse_file(shx_path)),
            dbf_file: try!(DbfFile::parse_file(dbf_path)),
        })
    }

    /// Constructs a `ShapefileRecordIterator` that can be used to iterate over the records inside
    /// the Shapefile.
    pub fn iter<'a>(&'a mut self) -> ShapefileRecordIterator<'a> {
        ShapefileRecordIterator {instance: self, id: 1u64}
    }

    /// Gives the data behind the record number
    pub fn record(&mut self, id: u64) -> Option<ShapefileRecord> {
        let mut result = ShapefileRecord {shape: Shape::new(), metadata: HashMap::new()};

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

impl<'a> Iterator for ShapefileRecordIterator<'a> {
    type Item = ShapefileRecord;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.id;
        let result = self.instance.record(id);
        self.id += 1u64;
        result
    }
}
