//! The crate containing all the parsing needed to load a shapefile.
//!
//! Acoording to the ESRI technical description, located at http://www.esri.com/library/whitepapers/pdfs/shapefile.pdf,
//! this includes at least three files:
//!
//! * Main File, .shp
//!
//!     Contains the geographic data in shapes (points, lines, ...).
//!
//! * Index File, .shx
//!
//!     An "overview" of the records stored in the main file.
//!
//! * dBASE Table, .dbf
//!
//!     The metadata associated with the geographic shapes from the main file.
//!
//!
//! There are a couple of other formats on my to-do list, which are mostly sidecar files like CPG for
//! the dBASE table encoding, and PRJ for the projection used in the main file.
//!
//! This file mostly defines the data structures for interchange. The function implementations reside
//! inside the respective submodules.
//!
//! This crate needs a nightly version of Rust, because of the `associated_consts` feature.
//!
//! # Example
//!
//! Of course, don't forget `extern crate shapefile_utils;` and the `use` clause!
//!
//! ```
//! use std::path::Path;
//!
//! let mut my_shapefile = shapefile_utils::Shapefile::new(
//!     &Path::new("assets/test.shp"),
//!     &Path::new("assets/test.shx"),
//!     &Path::new("assets/test.dbf")).unwrap();
//!
//! for record in my_shapefile.iter() {
//!     println!("Something called {:?}", record.metadata.get(&String::from("name")).unwrap());
//!     break;
//! }
//! ```

#![feature(associated_consts)]

extern crate byteorder;
extern crate dbf;

pub mod shape;
mod shapefile;
mod shpfile;
mod shxfile;
mod dbffile;

use std::collections::HashMap;
use std::io::BufReader;
use std::fs::File;

/// A bounding box specifying minimum and maximum values on X, Y, Z and M axes.
/// The x is for latitude, y is for longitude.
/// The z is for altitude and optional.
/// The m is a "measure" axis for scalar maps, and optional.
#[derive(Debug, PartialEq)]
struct BoundingBoxZ {
    /// The minimum latitude.
    x_min: f64,
    /// The minimum longitude.
    y_min: f64,

    /// The maximum latitude.
    x_max: f64,
    /// The maximum longitude.
    y_max: f64,

    /// The minimum altitude.
    z_min: f64,
    /// The maximum altitude.
    z_max: f64,

    /// The minimum measure.
    m_min: f64,
    /// The maximum measure.
    m_max: f64,
}

/// The header of a SHP file, as defined in the spec.
#[derive(Debug, PartialEq)]
struct FileHeader {
    /// The length of the file in 16-bit words.
    file_length: i32,
    /// One of the shape type constants with STY_*. All shapes in the file must be of the specified
    /// type, or of type 0.
    shape_type: i32,
    /// The bounding box of the data contained in the shape file.
    bounding_box: BoundingBoxZ,
}

/// A SHP file.
#[derive(Debug)]
struct ShpFile {
    /// The file header.
    header: FileHeader,
    /// The file handle
    file: BufReader<File>,
}

/// An index record.
#[derive(Debug)]
struct ShxRecord {
    /// Offset of the SHP file record measured in 16-bit words
    pub offset: i32,
    /// Length of the SHP file record measured in 16-bit words
    pub length: i32,
}

/// An SHX file
struct ShxFile {
    /// The SHX file header
    header: FileHeader,
    /// The file handle
    file: BufReader<File>,
}

/// A DBF file, implemented by the `dbf` crate
struct DbfFile {
    /// The `dbf` file handle
    file: dbf::DbfFile<File>,
}

/// Represents a record in the shapefile - has shape and metadata.
#[derive(Debug)]
pub struct ShapefileRecord {
    /// The shape as defined in the SHP file.
    pub shape: shape::Shape,
    /// The metadata as it comes from the DBF file.
    pub metadata: HashMap<String, dbf::Field>
}

/// The joint struct which makes the API of all of this.
pub struct Shapefile {
    /// SHP file handle
    shp_file: ShpFile,
    /// SHX file handle
    shx_file: ShxFile,
    /// DBF file handle
    dbf_file: DbfFile,
}

/// An iterator over record-organized structures.
pub struct ShapefileRecordIterator<'a> {
    /// The reference to the instance
    instance: &'a mut Shapefile,
    /// Current ID for the iterator
    id: u64,
}


#[cfg(test)]
mod tests {
    use super::{Shapefile, ShpFile, ShxFile, DbfFile};
    use std::path::Path;
    use dbf;

    #[test]
    fn test_shp_file_parse() {
        match ShpFile::parse_file(&Path::new("assets/test.shp")) {
            Ok(_) => (),
            Err(e) => {println!("Error: {:?}", e); panic!()},
        }
    }

    #[test]
    fn test_shx_file_parse() {
        match ShxFile::parse_file(&Path::new("assets/test.shx")) {
            Ok(_) => (),
            Err(e) => {println!("Error: {:?}", e); panic!()},
        }
    }

    #[test]
    fn test_dbf_file_parse() {
        match DbfFile::parse_file(&Path::new("assets/test.dbf")) {
            Ok(_) => (),
            Err(e) => {println!("Error: {:?}", e); panic!()},
        }
    }

    #[test]
    fn test_shapefile_direct_access() {
        let mut sf = Shapefile::new(&Path::new("assets/test.shp"), &Path::new("assets/test.shx"), &Path::new("assets/test.dbf")).unwrap();

        // Test some known value
        if let &dbf::Field::Character(ref s) = sf.record(1).unwrap().metadata.get(&String::from("name")).unwrap() {
            if s != &String::from("Dock 10") {
                panic!()
            }
        } else {
            panic!()
        }

        // Also take the last one, because you know, bounds checking and stuff
        if let &dbf::Field::Character(ref s) = sf.record(298773).unwrap().metadata.get(&String::from("osm_id")).unwrap() {
            if s != &String::from("464787242") {
                panic!()
            }
        } else {
            panic!()
        }

        // And some robustness testing with max+1 and min-1
        if let Some(_) = sf.record(298774) {
            panic!()
        }

        if let Some(_) = sf.record(0) {
            panic!()
        }
    }

    #[test]
    fn test_shapefile_record_iterator() {
        let mut sf = Shapefile::new(&Path::new("assets/test.shp"), &Path::new("assets/test.shx"), &Path::new("assets/test.dbf")).unwrap();

        let shape = sf.record(1u64).unwrap().shape;

        for record in sf.iter() {
            if record.shape != shape {
                panic!()
            } else {
                break
            }
        }

        // Play the same song again!
        for record in sf.iter() {
            if record.shape != shape {
                panic!()
            } else {
                break
            }
        }
    }
}
