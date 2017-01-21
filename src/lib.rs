#![feature(associated_consts)]

extern crate byteorder;

pub mod shpfile;

#[cfg(test)]
mod tests {
    use shpfile::ShpFile;

    #[test]
    fn test_shp_file_parse() {
        match ShpFile::parse("assets/test.shp") {
            Ok(_) => (),
            Err(e) => {println!("Error: {:?}", e);panic!()},
        }
    }
}
