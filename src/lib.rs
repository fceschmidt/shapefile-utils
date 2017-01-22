#![feature(associated_consts)]

extern crate byteorder;
extern crate dbf;

pub mod filetypes;

pub use filetypes::Shapefile as Shapefile;

#[cfg(test)]
mod tests {
    use filetypes::Shapefile;
    use std::path::Path;

    #[test]
    fn test_shapefile() {
        let mut sf = Shapefile::new(&Path::new("assets/test.shp"), &Path::new("assets/test.shx"), &Path::new("assets/test.dbf")).unwrap();
        println!("{:?}", sf.record(1u64));
    }
}
