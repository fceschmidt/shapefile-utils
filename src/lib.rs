#![feature(associated_consts)]

extern crate byteorder;
extern crate dbf;

pub mod filetypes;

pub use filetypes::Shapefile;
pub use filetypes::ShapefileRecordIterator;

#[cfg(test)]
mod tests {
    use super::Shapefile;
    use std::path::Path;
    use dbf;

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
