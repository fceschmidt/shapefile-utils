#![feature(associated_consts)]

extern crate byteorder;

pub mod shpfile;

#[cfg(test)]
mod tests {
    use shpfile::{ShpFile, Shape};
    use std::io::Cursor;
    use byteorder::{LittleEndian, WriteBytesExt};

    #[test]
    fn test_shp_file_parse() {
        match ShpFile::parse_file("assets/test.shp") {
            Ok(_) => (),
            Err(e) => {println!("Error: {:?}", e);panic!()},
        }
    }

    #[test]
    fn test_parse_nullshape() {
        let mut input: Vec<u8> = vec![];
        let _ = input.write_i32::<LittleEndian>(0);
        let (shape, _) = Shape::parse(&mut Cursor::new(input)).unwrap();
        match shape {
            Shape::NullShape => {},
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_point() {
        let mut input: Vec<u8> = vec![];
        let _ = input.write_i32::<LittleEndian>(1);
        let _ = input.write_f64::<LittleEndian>(0.25f64);
        let _ = input.write_f64::<LittleEndian>(0.5f64);
        let (shape, _) = Shape::parse(&mut Cursor::new(input)).unwrap();
        match shape {
            Shape::Point {point: p} => {
                if p.x != 0.25f64 || p.y != 0.5f64 {
                    panic!()
                }
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_multipoint() {
        let mut input: Vec<u8> = vec![];
        // Shape type
        let _ = input.write_i32::<LittleEndian>(8);
        // Bounding Box
        let _ = input.write_f64::<LittleEndian>(-0.25f64);
        let _ = input.write_f64::<LittleEndian>(-0.125f64);
        let _ = input.write_f64::<LittleEndian>(0.25f64);
        let _ = input.write_f64::<LittleEndian>(0.125f64);
        // Number of points
        let _ = input.write_i32::<LittleEndian>(3);
        // Three distinct points
        let _ = input.write_f64::<LittleEndian>(1f64);
        let _ = input.write_f64::<LittleEndian>(1f64);
        let _ = input.write_f64::<LittleEndian>(2f64);
        let _ = input.write_f64::<LittleEndian>(2f64);
        let _ = input.write_f64::<LittleEndian>(5f64);
        let _ = input.write_f64::<LittleEndian>(5f64);
        let (shape, _) = Shape::parse(&mut Cursor::new(input)).unwrap();
        match shape {
            Shape::MultiPoint {bounding_box: b, points: p} => {
                if b.x_min != -0.25f64 || b.y_min != -0.125f64 || b.x_max != 0.25f64 || b.y_max != 0.125f64 {
                    panic!()
                }
                if p[0].x != 1f64 || p[0].y != 1f64 || p[1].x != 2f64 || p[1].y != 2f64 || p[2].x != 5f64 || p[2].y != 5f64 {
                    panic!()
                }
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_polygon_polyline() {
        let mut input: Vec<u8> = vec![];
        // Shape type
        let _ = input.write_i32::<LittleEndian>(3);
        // Bounding Box
        let _ = input.write_f64::<LittleEndian>(-0.25f64);
        let _ = input.write_f64::<LittleEndian>(-0.125f64);
        let _ = input.write_f64::<LittleEndian>(0.25f64);
        let _ = input.write_f64::<LittleEndian>(0.125f64);
        // Number of parts
        let _ = input.write_i32::<LittleEndian>(2);
        // Number of points
        let _ = input.write_i32::<LittleEndian>(4);
        // Two distinct parts
        let _ = input.write_i32::<LittleEndian>(0);
        let _ = input.write_i32::<LittleEndian>(2);
        // Four distinct points
        let _ = input.write_f64::<LittleEndian>(1f64);
        let _ = input.write_f64::<LittleEndian>(1f64);
        let _ = input.write_f64::<LittleEndian>(2f64);
        let _ = input.write_f64::<LittleEndian>(2f64);
        let _ = input.write_f64::<LittleEndian>(5f64);
        let _ = input.write_f64::<LittleEndian>(5f64);
        let _ = input.write_f64::<LittleEndian>(6f64);
        let _ = input.write_f64::<LittleEndian>(6f64);

        let (polyline, _) = Shape::parse(&mut Cursor::new(&input)).unwrap();
        match &polyline {
            &Shape::PolyLine {bounding_box: ref b, parts: ref n, points: ref p} => {
                if b.x_min != -0.25f64 || b.y_min != -0.125f64 || b.x_max != 0.25f64 || b.y_max != 0.125f64 {
                    panic!()
                }
                if p[0].x != 1f64 || p[0].y != 1f64 || p[1].x != 2f64 || p[1].y != 2f64 || p[2].x != 5f64 || p[2].y != 5f64 || p[3].x != 6f64 || p[3].y != 6f64 {
                    panic!()
                }
                if n[0] != 0 || n[1] != 2 {
                    panic!()
                }
            },
            _ => panic!(),
        }

        let mut temp: Vec<u8> = vec![];
        let _ = temp.write_i32::<LittleEndian>(5);
        temp.extend_from_slice(&input[4..]);
        let input = temp;

        let (polygon, _) = Shape::parse(&mut Cursor::new(&input)).unwrap();

        match polyline {
            Shape::PolyLine {bounding_box: lb, parts: ln, points: lp} => {
                match polygon {
                    Shape::Polygon {bounding_box: gb, parts: gn, points: gp} => {
                        if gb != lb || gn != ln || gp != lp {
                            panic!()
                        }
                    },
                    _ => panic!()
                }
            },
            _ => panic!(),
        }
    }
}
