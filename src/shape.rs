//! The file with all definitions related to the Shape struct.

use std::io::{Error, ErrorKind, Read};
use byteorder::{LittleEndian, ReadBytesExt};

use super::BoundingBoxZ;

/// A bounding box limited to X and Y axes. For axis definitions, see the BoundinxBoxZ struct.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BoundingBox {
    /// The minimum latitude.
    pub x_min: f64,
    /// The minimum longitude.
    pub y_min: f64,

    /// The maximum latitude.
    pub x_max: f64,
    /// The maximum longitude.
    pub y_max: f64,
}

/// A point with latitude and longitude on an XY plane.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Point {
    /// The latitude of the point.
    pub x: f64,
    /// The longitude of the point.
    pub y: f64,
}

/// A generic range from a minimum to maximum value, over a type T.
#[derive(Debug, PartialEq)]
pub struct Range<T> {
    pub min: T,
    pub max: T,
}

/// Minimum and maximum on the Measure axis.
pub type MRange = Range<f64>;

/// Minimum and maximum on the altitude axis.
pub type ZRange = Range<f64>;

/// A point with latitude, longitude, and a measure.
#[derive(Debug, PartialEq)]
pub struct PointM {
    /// The latitude
    pub x: f64,
    /// The longitude
    pub y: f64,

    /// The associated scalar measure
    pub m: f64,
}

/// A point with latitude, longitude, altitude and an optional measure
#[derive(Debug, PartialEq)]
pub struct PointZ {
    /// The latitude
    pub x: f64,
    /// The longitude
    pub y: f64,
    /// The altitude
    pub z: f64,

    /// The associated scalar measure
    pub m: f64,
}

/// The type of a single patch (see MultiPatch shape type).
/// Defined on page 20 of the spec.
#[derive(Debug, Eq, PartialEq)]
pub enum PatchType {
    /// Every vertex after the first two spans a triangle with its two predecessors.
    TriangleStrip,
    /// Every vertex after the first two spans a triangle with its predecessor and the first one.
    TriangleFan,
    /// The outer ring of a polygon.
    OuterRing,
    /// A hole of a polygon.
    InnerRing,
    /// The first ring of a polygon of an unspecified type.
    FirstRing,
    /// A ring of a polygon of an unspecified type.
    Ring,
}

/// A shape record defining a geometric feature in the SHP file.
#[derive(Debug, PartialEq)]
pub enum Shape {
    /// The null shape: Empty info.
    NullShape,

    /// A simple point
    Point {
        point: Point
    },

    /// An ordered set of vertices that consists of one or more parts.
    /// A part is a connected sequence of two or more vertices.
    PolyLine {
        bounding_box: BoundingBox,
        parts: Vec<i32>,
        points: Vec<Point>
    },

    /// A polygon with four or more points.
    /// Organized in rings. A ring consists of four or more vertices that form a closed, non-self-
    /// intersecting loop. A polygon may contain multiple outer rings. The order of vertices or
    /// orientation for a ring indicates which side of the ring is the interior of the polygon.
    /// The neighborhood to the right of an observer walking along the ring in vertex order is the
    /// neighborhood inside the polygon. Vertices of rings defining holes in polygons are in a
    /// counterclockwise direction. Vertices for a single, ringed polygon are, therefore, always
    /// in clockwise order. The rings of a polygon are referred to as its parts.
    Polygon {
        bounding_box: BoundingBox,
        parts: Vec<i32>,
        points: Vec<Point>
    },

    /// A set of points.
    MultiPoint {
        bounding_box: BoundingBox,
        points: Vec<Point>
    },

    /// See Point. Has an additional altitude and measure coordinate.
    PointZ {
        point: PointZ
    },

    /// See PolyLine. Has additional altitude and measure coordinates.
    PolyLineZ {
        bounding_box: BoundingBox,
        parts: Vec<i32>,
        points: Vec<Point>,
        z_range: ZRange,
        z: Vec<f64>,
        m_range: MRange,
        m: Vec<f64>
    },

    /// See Polygon. Has additional altitude and measure coordinates.
    PolygonZ {
        bounding_box: BoundingBox,
        parts: Vec<i32>,
        points: Vec<Point>,
        z_range: ZRange,
        z: Vec<f64>,
        m_range: MRange,
        m: Vec<f64>
    },

    /// See MultiPoint. Has additional altitude and measure coordinates.
    MultiPointZ {
        bounding_box: BoundingBox,
        points: Vec<Point>,
        z_range: ZRange,
        z: Vec<f64>,
        m_range: MRange,
        m: Vec<f64>
    },

    /// See Point. Has an additional measure coordinate.
    PointM {
        point: PointM
    },

    /// See PolyLine. Has additional measure coordinates.
    PolyLineM {
        bounding_box: BoundingBox,
        parts: Vec<i32>,
        points: Vec<Point>,
        m_range: MRange,
        m: Vec<f64>
    },

    /// See Polygon. Has additional measure coordinates.
    PolygonM {
        bounding_box: BoundingBox,
        parts: Vec<i32>,
        points: Vec<Point>,
        m_range: MRange,
        m: Vec<f64>
    },

    /// See MultiPoint. Has additional measure coordinates.
    MultiPointM {
        bounding_box: BoundingBox,
        points: Vec<Point>,
        m_range: MRange,
        m: Vec<f64>
    },

    /// A MultiPatch consists of a number of surface patches. Each surface patch describes a surface.
    /// The surface patches of a MultiPatch are referred to as its parts, and the type of part
    /// controls how the order of vertices of an MultiPatch part is interpreted.
    MultiPatch {
        bounding_box: BoundingBox,
        parts: Vec<i32>,
        part_types: Vec<PatchType>,
        points: Vec<Point>,
        z_range: ZRange,
        z: Vec<f64>,
        m_range: MRange,
        m: Vec<f64>
    },
}

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


/// An internal struct for interchanging shape data.
#[derive(Debug, PartialEq)]
struct ShapeBaseData {
    bounding_box: BoundingBox,
    num_parts: i32,
    num_points: i32,
    parts: Vec<i32>,
    part_types: Vec<PatchType>,
    points: Vec<Point>,
    z_range: Range<f64>,
    z: Vec<f64>,
    m_range: Range<f64>,
    m: Vec<f64>,
}

impl ShapeBaseData {
    fn new() -> ShapeBaseData {
        ShapeBaseData {
            bounding_box: BoundingBox::new(),
            num_parts: 0,
            num_points: 0,
            parts: vec![],
            part_types: vec![],
            points: vec![],
            z_range: Range::<f64> {min: 0f64, max: 0f64},
            z: vec![],
            m_range: Range::<f64> {min: 0f64, max: 0f64},
            m: vec![],
        }
    }
}

impl BoundingBox {
    /// Returns a bounding box initialized to all zeroes.
    pub fn new() -> Self {
        BoundingBox {
            x_min: 0f64,
            y_min: 0f64,
            x_max: 0f64,
            y_max: 0f64
        }
    }

    /// Parses a bounding box by consuming four doubles from the input stream.
    pub fn parse<T: Read>(file: &mut T) -> Result<Self, Error> {
        let mut result = Self::new();

        result.x_min = (file.read_f64::<LittleEndian>())?;
        result.y_min = (file.read_f64::<LittleEndian>())?;
        result.x_max = (file.read_f64::<LittleEndian>())?;
        result.y_max = (file.read_f64::<LittleEndian>())?;

        Ok(result)
    }
}

impl Point {
    /// Returns a point initialized to (0,0)
    pub fn new() -> Self {
        Point {x: 0f64, y: 0f64}
    }

    /// Parses a file by reading two f64s in little-endian format from the input stream.
    pub fn parse<T: Read>(file: &mut T) -> Result<Self, Error> {
        let mut result = Self::new();

        result.x = (file.read_f64::<LittleEndian>())?;
        result.y = (file.read_f64::<LittleEndian>())?;

        Ok(result)
    }
}

impl Shape {
    /// Constants for encoding the Shape Type (see enum variants)
    const STY_NULL_SHAPE: i32 = 0;
    const STY_POINT: i32 = 1;
    const STY_POLY_LINE: i32 = 3;
    const STY_POLYGON: i32 = 5;
    const STY_MULTI_POINT: i32 = 8;
    const STY_POINT_Z: i32 = 11;
    const STY_POLY_LINE_Z: i32 = 13;
    const STY_POLYGON_Z: i32 = 15;
    const STY_MULTI_POINT_Z: i32 = 18;
    const STY_POINT_M: i32 = 21;
    const STY_POLY_LINE_M: i32 = 23;
    const STY_POLYGON_M: i32 = 25;
    const STY_MULTI_POINT_M: i32 = 28;
    const STY_MULTI_PATCH: i32 = 31;

    /// Constants for encoding the Patch Type (see MultiPatch variant)
    const PTY_TRIANGLE_STRIP: i32 = 0;
    const PTY_TRIANGLE_FAN: i32 = 1;
    const PTY_OUTER_RING: i32 = 2;
    const PTY_INNER_RING: i32 = 3;
    const PTY_FIRST_RING: i32 = 4;
    const PTY_RING: i32 = 5;

    /// Returns a NullShape variant
    pub fn new() -> Self {
        Shape::NullShape
    }

    /// Consumes an array of num Ts from the input stream and returns them in a Vec.
    fn parse_array<R: Read, V, F>(file: &mut R, n: usize, mut read_function: F) -> Result<Vec<V>, Error>
    where F: FnMut(&mut R) -> Result<V, Error>
    {
        let mut result: Vec<V> = vec![];
        for _ in 0..n {
            result.push((read_function(file))?);
        }
        Ok(result)
    }

    /// Consumes an array of num i32's from the input stream and returns them in a Vec.
    fn parse_i32_array<T: Read>(file: &mut T, n: usize) -> Result<Vec<i32>,Error> {
        Self::parse_array(file, n, ReadBytesExt::read_i32::<LittleEndian>)
    }

    /// Consumes an array of num points from the input stream and returns them in a Vec.
    fn parse_point_array<T: Read>(file: &mut T, n: usize) -> Result<Vec<Point>,Error> {
        Self::parse_array(file, n, Point::parse)
    }

    /// Consumes an array of num f64's from the input stream and returns them in a Vec.
    fn parse_f64_array<T: Read>(file: &mut T, n: usize) -> Result<Vec<f64>,Error> {
        Self::parse_array(file, n, ReadBytesExt::read_f64::<LittleEndian>)
    }

    /// Gets called internally for parsing a point.
    fn parse_point_type<T: Read>(file: &mut T, shape_type: i32) -> Result<(Self, usize), Error> {
        match shape_type {
            // Points come first
            Self::STY_POINT => {
                // X and Y, both double and little endian
                let v = (Self::parse_f64_array(file, 2))?;
                Ok((Shape::Point {point: Point{x: v[0], y: v[1]}}, 16))
            },
            Self::STY_POINT_M => {
                // X, Y and M, both double and little endian
                let v = (Self::parse_f64_array(file, 3))?;
                Ok((Shape::PointM {point: PointM{x: v[0], y: v[1], m: v[2]}}, 24))
            },
            Self::STY_POINT_Z => {
                // X, Y, M and Z, both double and little endian
                let v = (Self::parse_f64_array(file, 4))?;
                Ok((Shape::PointZ {point: PointZ{x: v[0], y: v[1], z: v[2], m: v[3]}}, 32))
            },
            _ => Err(Error::new(ErrorKind::Other, "Supposed point not of any point type!")),
        }
    }

    /// Given the encoded ID of a patch type (see MultiPatch), returns the right enum variant for it.
    fn get_patch_type_from_id(id: &i32) -> Option<PatchType> {
        match *id {
            Self::PTY_TRIANGLE_STRIP => {
                Some(PatchType::TriangleStrip)
            },
            Self::PTY_TRIANGLE_FAN => {
                Some(PatchType::TriangleFan)
            },
            Self::PTY_INNER_RING => {
                Some(PatchType::InnerRing)
            },
            Self::PTY_OUTER_RING => {
                Some(PatchType::OuterRing)
            },
            Self::PTY_FIRST_RING => {
                Some(PatchType::FirstRing)
            },
            Self::PTY_RING => {
                Some(PatchType::Ring)
            },
            _ => {
                // Need to handle the default case somehow...
                None
            },
        }
    }

    /// Consumes two f64 values and an array of f64 values with num entries, and returns a Range
    /// and a Vec object from the data.
    fn parse_f64_range_and_array<T: Read>(file: &mut T, n: usize) -> Result<(Range<f64>, Vec<f64>), Error> {
        let v = try!(Self::parse_f64_array(file, 2));
        let range = Range::<f64> {min: v[0], max: v[1]};
        let arr = try!(Self::parse_f64_array(file, n));
        Ok((range, arr))
    }

    /// Given a Shape type ID and the parsed base data, we can already construct a valid shape
    /// object.
    fn shape_from_base_data(shape_type: i32, base: ShapeBaseData) -> Self {
        match shape_type {
            // The poly lines
            Self::STY_POLY_LINE => {
                Shape::PolyLine {bounding_box: base.bounding_box, parts: base.parts, points: base.points}
            },
            Self::STY_POLY_LINE_M => {
                Shape::PolyLineM {
                    bounding_box: base.bounding_box,
                    parts: base.parts,
                    points: base.points,
                    m_range: base.m_range,
                    m: base.m}
            },
            Self::STY_POLY_LINE_Z => {
                Shape::PolyLineZ {
                    bounding_box: base.bounding_box,
                    parts: base.parts,
                    points: base.points,
                    z: base.z,
                    z_range: base.z_range,
                    m_range: base.m_range,
                    m: base.m}
            },
            // The polygons
            Self::STY_POLYGON => {
                Shape::Polygon {bounding_box: base.bounding_box, parts: base.parts, points: base.points}
            },
            Self::STY_POLYGON_M => {
                Shape::PolygonM {
                    bounding_box: base.bounding_box,
                    parts: base.parts,
                    points: base.points,
                    m_range: base.m_range,
                    m: base.m}
            },
            Self::STY_POLYGON_Z => {
                Shape::PolygonZ {
                    bounding_box: base.bounding_box,
                    parts: base.parts,
                    points: base.points,
                    z_range: base.z_range,
                    z: base.z,
                    m_range: base.m_range,
                    m: base.m}
            },
            // Then the multipoints
            Self::STY_MULTI_POINT => {
                Shape::MultiPoint {bounding_box: base.bounding_box, points: base.points}
            },
            Self::STY_MULTI_POINT_M => {
                Shape::MultiPointM {
                    bounding_box: base.bounding_box,
                    points: base.points,
                    m_range: base.m_range,
                    m: base.m}
            },
            Self::STY_MULTI_POINT_Z => {
                Shape::MultiPointZ {
                    bounding_box: base.bounding_box,
                    points: base.points,
                    z_range: base.z_range,
                    z: base.z,
                    m_range: base.m_range,
                    m: base.m}
            },
            Self::STY_MULTI_PATCH => {
                Shape::MultiPatch {
                    bounding_box: base.bounding_box,
                    parts: base.parts,
                    part_types: base.part_types,
                    points: base.points,
                    z_range: base.z_range,
                    z: base.z,
                    m_range: base.m_range,
                    m: base.m}
            },
            Self::STY_NULL_SHAPE => {
                Shape::NullShape
            },
            _ => {
                // Probably a sane default
                Shape::NullShape
            }
        }
    }

    /// Parses a shape from the input stream.
    pub fn parse<T: Read>(file: &mut T) -> Result<(Self, usize), Error> {
        let shape_type = try!(file.read_i32::<LittleEndian>());
        let mut length = 4usize;

        // Get the points out of here, they're too special
        match shape_type {
            Self::STY_POINT
            | Self::STY_POINT_M
            | Self::STY_POINT_Z => {
                let (sh, sz) = try!(Self::parse_point_type(file, shape_type));
                return Ok((sh, sz + length))
            },
            _ => (),
        }

        // For all types which have a bounding box, read it first.
        let mut base: ShapeBaseData = ShapeBaseData::new();

        match shape_type {
            Self::STY_POLY_LINE
            | Self::STY_POLYGON
            | Self::STY_POLY_LINE_M
            | Self::STY_POLYGON_M
            | Self::STY_POLY_LINE_Z
            | Self::STY_POLYGON_Z
            | Self::STY_MULTI_PATCH => {
                length += 40usize;
                base.bounding_box = try!(BoundingBox::parse(file));
                base.num_parts = try!(file.read_i32::<LittleEndian>());
                base.num_points = try!(file.read_i32::<LittleEndian>());
                length += 4 * base.num_parts as usize;
                base.parts = try!(Self::parse_i32_array(file, base.num_parts as usize));

                if shape_type == Self::STY_MULTI_PATCH {
                    let part_types_id = try!(Self::parse_i32_array(file, base.num_parts as usize));
                    length += 4 * base.num_parts as usize;
                    base.part_types = part_types_id.iter()
                                                   .map(|x| Self::get_patch_type_from_id(x).unwrap())
                                                   .collect();
                }

                length += 16 * base.num_points as usize;
                base.points = try!(Self::parse_point_array(file, base.num_points as usize));
            },
            Self::STY_MULTI_POINT
            | Self::STY_MULTI_POINT_M
            | Self::STY_MULTI_POINT_Z => {
                length += 36usize;
                base.bounding_box = try!(BoundingBox::parse(file));
                base.num_points = try!(file.read_i32::<LittleEndian>());
                length += 16 * base.num_points as usize;
                base.points = try!(Self::parse_point_array(file, base.num_points as usize));
            },
            _ => ()
        };

        match shape_type {
            Self::STY_POLY_LINE_Z
            | Self::STY_POLYGON_Z
            | Self::STY_MULTI_POINT_Z
            | Self::STY_MULTI_PATCH => {
                let (z_range, z) = try!(Self::parse_f64_range_and_array(file, base.num_points as usize));
                let (m_range, m) = try!(Self::parse_f64_range_and_array(file, base.num_points as usize));
                base.z_range = z_range;
                base.z = z;
                base.m_range = m_range;
                base.m = m;
                length += 32usize + 16 * base.num_points as usize;
            },
            Self::STY_POLY_LINE_M
            | Self::STY_POLYGON_M
            | Self::STY_MULTI_POINT_M => {
                let (m_range, m) = try!(Self::parse_f64_range_and_array(file, base.num_points as usize));
                base.m_range = m_range;
                base.m = m;
                length += 16usize + 8 * base.num_points as usize;
            },
            _ => ()
        }

        Ok((Self::shape_from_base_data(shape_type, base), length))
    }
}

impl BoundingBoxZ {
    /// Creates a BoundingBoxZ with all zeros
    pub fn new() -> Self {
        BoundingBoxZ {
            x_min: 0f64,
            y_min: 0f64,
            x_max: 0f64,
            y_max: 0f64,
            z_min: 0f64,
            z_max: 0f64,
            m_min: 0f64,
            m_max: 0f64
        }
    }

    /// Parses a BoundingBoxZ from the binary input stream
    pub fn parse<T: Read>(file: &mut T) -> Result<Self, Error> {
        let mut result = Self::new();

        // As per the spec, read all the fields sequentially as f64s in little endian
        result.x_min = try!(file.read_f64::<LittleEndian>());
        result.y_min = try!(file.read_f64::<LittleEndian>());
        result.x_max = try!(file.read_f64::<LittleEndian>());
        result.y_max = try!(file.read_f64::<LittleEndian>());
        result.z_min = try!(file.read_f64::<LittleEndian>());
        result.z_max = try!(file.read_f64::<LittleEndian>());
        result.m_min = try!(file.read_f64::<LittleEndian>());
        result.m_max = try!(file.read_f64::<LittleEndian>());

        // Return what we've got
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{Shape, BoundingBox, Point, PatchType};
    use std::io::Cursor;
    use byteorder::{LittleEndian, WriteBytesExt};

    fn write_vec<T, F>(vec: &Vec<T>, mut w_func: F, buffer: &mut Vec<u8>)
    where F: FnMut(&mut Vec<u8>, &T) -> ()
    {
        for elem in vec {
            w_func(buffer, elem);
        }
    }

    fn write_box(bounding_box: &BoundingBox, buffer: &mut Vec<u8>) {
        let _ = buffer.write_f64::<LittleEndian>(bounding_box.x_min);
        let _ = buffer.write_f64::<LittleEndian>(bounding_box.y_min);
        let _ = buffer.write_f64::<LittleEndian>(bounding_box.x_max);
        let _ = buffer.write_f64::<LittleEndian>(bounding_box.y_max);
    }

    fn write_point_vec(points: &Vec<Point>, buffer: &mut Vec<u8>) {
        write_vec(points, |w, elem| {
            let _ = w.write_f64::<LittleEndian>(elem.x);
            let _ = w.write_f64::<LittleEndian>(elem.y);
        }, buffer);
    }

    fn write_i32_vec(input: &Vec<i32>, buffer: &mut Vec<u8>) {
        write_vec(input, |w, elem| w.write_i32::<LittleEndian>(*elem).unwrap(), buffer);
    }

    fn write_f64_vec(input: &Vec<f64>, buffer: &mut Vec<u8>) {
        write_vec(input, |w, elem| w.write_f64::<LittleEndian>(*elem).unwrap(), buffer);
    }

    #[test]
    fn test_parse_nullshape() {
        let mut input: Vec<u8> = vec![];
        let _ = input.write_i32::<LittleEndian>(0);

        if let (Shape::NullShape, 4) = Shape::parse(&mut Cursor::new(input)).unwrap() {
            // No data to validate
        } else {
            panic!();
        }
    }

    #[test]
    fn test_parse_point() {
        let mut input: Vec<u8> = vec![];
        let _ = input.write_i32::<LittleEndian>(1);
        let _ = input.write_f64::<LittleEndian>(0.25f64);
        let _ = input.write_f64::<LittleEndian>(0.5f64);

        if let (Shape::Point {point: p}, 20) = Shape::parse(&mut Cursor::new(input)).unwrap() {
            if p.x != 0.25f64 || p.y != 0.5f64 {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_parse_multipoint() {
        let mut input: Vec<u8> = vec![];
        // Shape type
        let _ = input.write_i32::<LittleEndian>(8);
        // Bounding Box
        let bounding_box = BoundingBox{x_min: -0.25f64, y_min: -0.125f64, x_max: 0.25f64, y_max: 0.125f64};
        write_box(&bounding_box, &mut input);
        // Number of points
        let points = vec![Point{x: 1f64, y: 1f64},Point{x: 2f64, y: 2f64},Point{x: 5f64, y: 5f64}];
        let _ = input.write_i32::<LittleEndian>(points.len() as i32);
        write_point_vec(&points, &mut input);

        if let (Shape::MultiPoint {bounding_box: bb, points: p}, 88) = Shape::parse(&mut Cursor::new(input)).unwrap() {
            if bb != bounding_box || p != points {
                panic!();
            }
        } else {
            panic!();
        }

    }

    #[test]
    fn test_parse_polygon_polyline() {
        let mut input: Vec<u8> = vec![];
        // Shape type - 4 bytes
        input.write_i32::<LittleEndian>(3).unwrap();
        // Bounding Box - 32 bytes
        let bounding_box = BoundingBox{x_min: -0.25f64, y_min: -0.125f64, x_max: 0.25f64, y_max: 0.125f64};
        write_box(&bounding_box, &mut input);

        let parts: Vec<i32> = vec![0,2];
        let points = vec![Point{x: 1f64, y: 1f64},Point{x: 2f64, y: 2f64},Point{x: 5f64, y: 5f64},Point{x: 6f64, y: 6f64}];

        // Write lengths - 8 bytes
        input.write_i32::<LittleEndian>(parts.len() as i32).unwrap();
        input.write_i32::<LittleEndian>(points.len() as i32).unwrap();

        // Write values - 8 + 64 bytes
        write_i32_vec(&parts, &mut input);
        write_point_vec(&points, &mut input);

        // Then see whether the data gets parsed correctly
        let polyline: Shape;
        if let (Shape::PolyLine {bounding_box: bb, parts: pa, points: pt}, 116) = Shape::parse(&mut Cursor::new(&input)).unwrap() {
            if bb != bounding_box || pa != parts || pt != points {
                panic!()
            }
            // Keep track of the parsed data
            polyline = Shape::PolyLine{bounding_box: bb, parts: pa, points: pt};
        } else {
            panic!()
        }

        // Put 5 as shape type instead of three (structure is the same)
        let mut temp: Vec<u8> = vec![];
        let _ = temp.write_i32::<LittleEndian>(5);
        temp.extend_from_slice(&input[4..]);
        let input = temp;

        // Parse that and see whether the two are equal by fields
        if let (Shape::Polygon {bounding_box: bb, parts: pa, points: pt}, 116) = Shape::parse(&mut Cursor::new(&input)).unwrap() {
            if let Shape::PolyLine {bounding_box: lb, parts: ln, points: lp} = polyline  {
                if bb != lb || pa != ln || pt != lp {
                    panic!()
                }
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }

    #[test]
    fn test_parse_pointm() {
        let mut input: Vec<u8> = Vec::new();
        let _ = input.write_i32::<LittleEndian>(21);
        let _ = input.write_f64::<LittleEndian>(1.0);
        let _ = input.write_f64::<LittleEndian>(1.2);
        let _ = input.write_f64::<LittleEndian>(1.4);

        if let (Shape::PointM {point}, 28) = Shape::parse(&mut Cursor::new(&input)).unwrap() {
            if point.x != 1.0 || point.y != 1.2 || point.m != 1.4 {
                panic!()
            }
        } else {
            panic!()
        }
    }

    #[test]
    fn test_parse_multipointm() {
        let mut input: Vec<u8> = vec![];
        // Shape type
        input.write_i32::<LittleEndian>(28).unwrap();
        // Bounding Box
        let bounding_box = BoundingBox{x_min: -0.25f64, y_min: -0.125f64, x_max: 0.25f64, y_max: 0.125f64};
        write_box(&bounding_box, &mut input);
        // Number of points
        let points = vec![Point{x: 1f64, y: 1f64},Point{x: 2f64, y: 2f64},Point{x: 5f64, y: 5f64}];
        input.write_i32::<LittleEndian>(points.len() as i32).unwrap();
        write_point_vec(&points, &mut input);

        // Measures
        let mrange = vec![10.1, 91.1];
        let ms = vec![33.0, 91.1, 10.1];
        write_f64_vec(&mrange, &mut input);
        write_f64_vec(&ms, &mut input);

        if let (Shape::MultiPointM {bounding_box: bb, points: p, m, m_range: rm}, 128) = Shape::parse(&mut Cursor::new(input)).unwrap() {
            if bb != bounding_box || p != points || m != ms || rm.min != mrange[0] || rm.max != mrange[1] {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_parse_polylinem_polygonm() {
        let mut input: Vec<u8> = vec![];
        // Shape type - 4 bytes
        input.write_i32::<LittleEndian>(23).unwrap();
        // Bounding Box - 32 bytes
        let bounding_box = BoundingBox{x_min: -0.25f64, y_min: -0.125f64, x_max: 0.25f64, y_max: 0.125f64};
        write_box(&bounding_box, &mut input);

        let parts: Vec<i32> = vec![0,2];
        let points = vec![Point{x: 1f64, y: 1f64},Point{x: 2f64, y: 2f64},Point{x: 5f64, y: 5f64},Point{x: 6f64, y: 6f64}];

        // Write lengths - 8 bytes
        input.write_i32::<LittleEndian>(parts.len() as i32).unwrap();
        input.write_i32::<LittleEndian>(points.len() as i32).unwrap();

        // Write values - 8 + 64 bytes
        write_i32_vec(&parts, &mut input);
        write_point_vec(&points, &mut input);

        // Write M shit - 16 bytes + 4 * 8 bytes
        let m_range = vec![1.0, 50.0];
        let m = vec![1.0, 50.0, 49.1, 26.1];
        write_f64_vec(&m_range, &mut input);
        write_f64_vec(&m, &mut input);

        // Then see whether the data gets parsed correctly
        let polyline: Shape;
        if let (Shape::PolyLineM {bounding_box: bb, parts: pa, points: pt, m_range: rm, m: ms}, 164) = Shape::parse(&mut Cursor::new(&input)).unwrap() {
            if bb != bounding_box || pa != parts || pt != points || ms != m || rm.min != m_range[0] || rm.max != m_range[1] {
                panic!()
            }
            // Keep track of the parsed data
            polyline = Shape::PolyLineM {bounding_box: bb, parts: pa, points: pt, m_range: rm, m: ms};
        } else {
            panic!()
        }

        // Put 5 as shape type instead of three (structure is the same)
        let mut temp: Vec<u8> = vec![];
        let _ = temp.write_i32::<LittleEndian>(25);
        temp.extend_from_slice(&input[4..]);
        let input = temp;

        // Parse that and see whether the two are equal by fields
        if let (Shape::PolygonM {bounding_box: bb, parts: pa, points: pt, m_range: rm, m: ms}, 164) = Shape::parse(&mut Cursor::new(&input)).unwrap() {
            if let Shape::PolyLineM {bounding_box: bbb, parts: bpa, points: bpt, m_range: brm, m: bms} = polyline  {
                if bb != bbb || pa != bpa || pt != bpt || rm != brm || ms != bms {
                    panic!()
                }
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }

    #[test]
    fn test_parse_pointz() {
        let mut input: Vec<u8> = Vec::new();
        let _ = input.write_i32::<LittleEndian>(11);
        let _ = input.write_f64::<LittleEndian>(1.0);
        let _ = input.write_f64::<LittleEndian>(1.2);
        let _ = input.write_f64::<LittleEndian>(1.4);
        let _ = input.write_f64::<LittleEndian>(1.6);

        if let (Shape::PointZ {point}, 36) = Shape::parse(&mut Cursor::new(&input)).unwrap() {
            if point.x != 1.0 || point.y != 1.2 || point.z != 1.4 || point.m != 1.6 {
                panic!()
            }
        } else {
            panic!()
        }
    }

    #[test]
    fn test_parse_multipointz() {
        let mut input: Vec<u8> = vec![];
        // Shape type
        input.write_i32::<LittleEndian>(18).unwrap();
        // Bounding Box
        let bounding_box = BoundingBox{x_min: -0.25f64, y_min: -0.125f64, x_max: 0.25f64, y_max: 0.125f64};
        write_box(&bounding_box, &mut input);
        // Number of points
        let points = vec![Point{x: 1f64, y: 1f64},Point{x: 2f64, y: 2f64},Point{x: 5f64, y: 5f64}];
        input.write_i32::<LittleEndian>(points.len() as i32).unwrap();
        write_point_vec(&points, &mut input);

        // Z values
        let z_range = vec![11.1, 92.1];
        let z = vec![85.5, 92.1, 11.1];
        write_f64_vec(&z_range, &mut input);
        write_f64_vec(&z, &mut input);

        // Measures
        let m_range = vec![10.1, 91.1];
        let m = vec![33.0, 91.1, 10.1];
        write_f64_vec(&m_range, &mut input);
        write_f64_vec(&m, &mut input);

        if let (Shape::MultiPointZ {bounding_box: bb, points: p, z: zs, z_range: rz, m: ms, m_range: rm}, 168) = Shape::parse(&mut Cursor::new(input)).unwrap() {
            if bb != bounding_box || p != points || m != ms || rm.min != m_range[0] || rm.max != m_range[1] || z != zs || rz.min != z_range[0] || rz.max != z_range[1] {
                panic!();
            }
        } else {
            panic!();
        }
    }

    #[test]
    fn test_parse_polylinez_polygonz() {
        let mut input: Vec<u8> = vec![];
        // Shape type - 4 bytes
        input.write_i32::<LittleEndian>(13).unwrap();
        // Bounding Box - 32 bytes
        let bounding_box = BoundingBox{x_min: -0.25f64, y_min: -0.125f64, x_max: 0.25f64, y_max: 0.125f64};
        write_box(&bounding_box, &mut input);

        let parts: Vec<i32> = vec![0,2];
        let points = vec![Point{x: 1f64, y: 1f64},Point{x: 2f64, y: 2f64},Point{x: 5f64, y: 5f64},Point{x: 6f64, y: 6f64}];

        // Write lengths - 8 bytes
        input.write_i32::<LittleEndian>(parts.len() as i32).unwrap();
        input.write_i32::<LittleEndian>(points.len() as i32).unwrap();

        // Write values - 8 + 64 bytes
        write_i32_vec(&parts, &mut input);
        write_point_vec(&points, &mut input);

        // Write Z shit - 16 bytes + 4 * 8 bytes
        let z_range = vec![51.0, 100.0];
        let z = vec![51.0, 100.0, 99.1, 76.1];
        write_f64_vec(&z_range, &mut input);
        write_f64_vec(&z, &mut input);

        // Write M shit - 16 bytes + 4 * 8 bytes
        let m_range = vec![1.0, 50.0];
        let m = vec![1.0, 50.0, 49.1, 26.1];
        write_f64_vec(&m_range, &mut input);
        write_f64_vec(&m, &mut input);

        // Then see whether the data gets parsed correctly
        let polyline: Shape;
        if let (Shape::PolyLineZ {bounding_box: bb, parts: pa, points: pt, z_range: rz, z: zs, m_range: rm, m: ms}, 212) = Shape::parse(&mut Cursor::new(&input)).unwrap() {
            if bb != bounding_box || pa != parts || pt != points || ms != m || rm.min != m_range[0] || rm.max != m_range[1] || zs != z || rz.min != z_range[0] || rz.max != z_range[1] {
                panic!()
            }
            // Keep track of the parsed data
            polyline = Shape::PolyLineZ {bounding_box: bb, parts: pa, points: pt, z_range: rz, z: zs, m_range: rm, m: ms};
        } else {
            panic!()
        }

        // Put 5 as shape type instead of three (structure is the same)
        let mut temp: Vec<u8> = vec![];
        let _ = temp.write_i32::<LittleEndian>(15);
        temp.extend_from_slice(&input[4..]);
        let input = temp;

        // Parse that and see whether the two are equal by fields
        if let (Shape::PolygonZ {bounding_box: bb, parts: pa, points: pt, z_range: rz, z: zs, m_range: rm, m: ms}, 212) = Shape::parse(&mut Cursor::new(&input)).unwrap() {
            if let Shape::PolyLineZ {bounding_box: bbb, parts: bpa, points: bpt, z_range: brz, z: bzs, m_range: brm, m: bms} = polyline  {
                if bb != bbb || pa != bpa || pt != bpt || rm != brm || ms != bms || rz != brz || zs != bzs {
                    panic!()
                }
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }

    #[test]
    fn test_parse_multipatch() {
        let mut input: Vec<u8> = vec![];
        // Shape type - 4 bytes
        input.write_i32::<LittleEndian>(31).unwrap();

        // Bounding Box - 32 bytes @36
        let bounding_box = BoundingBox{x_min: -0.25f64, y_min: -0.125f64, x_max: 0.25f64, y_max: 0.125f64};
        write_box(&bounding_box, &mut input);

        // Parts, part types and Points - 8 + 2 * (3 * 4) + (12 * 16) bytes @260
        input.write_i32::<LittleEndian>(3).unwrap();    // number of parts
        input.write_i32::<LittleEndian>(12).unwrap();   // number of points
        write_i32_vec(&vec![0, 4, 8], &mut input);      // parts
        write_i32_vec(&vec![0, 0, 0], &mut input);      // part types
        write_point_vec(&vec![                          // points
            Point {x: 0.0, y: 0.0},
            Point {x: 1.0, y: 0.5},
            Point {x: 0.0, y: 1.0},
            Point {x: 1.0, y: 1.5},

            Point {x: 0.0, y: 2.0},
            Point {x: 1.0, y: 2.5},
            Point {x: 0.0, y: 3.0},
            Point {x: 1.0, y: 3.5},

            Point {x: 0.0, y: 4.0},
            Point {x: 1.0, y: 4.5},
            Point {x: 0.0, y: 5.0},
            Point {x: 1.0, y: 5.5},
            ], &mut input);

        // Z values - 2 * 8 + 12 * 8 bytes @ 372
        write_f64_vec(&vec![0.0, 2.0], &mut input);
        write_f64_vec(&vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0], &mut input);

        // M values - 2 * 8 + 12 * 8 bytes @ 484
        write_f64_vec(&vec![0.17, 0.98], &mut input);
        write_f64_vec(&vec![0.32, 0.56, 0.98, 0.17, 0.55, 0.51, 0.501, 0.42, 0.47, 0.6, 0.51, 0.5], &mut input);

        if let (Shape::MultiPatch {bounding_box: bb, parts, part_types, points, z_range, z, m_range, m}, 484) = Shape::parse(&mut Cursor::new(&input)).unwrap() {
            if bounding_box != bb || parts[0] != 0 || part_types[1] != PatchType::TriangleStrip || points[8].y != 4f64
            || z_range.max != 2.0 || z[8] != 2.0 || m_range.max != 0.98 || m[8] != 0.47 {
                panic!()
            }
        } else {
            panic!()
        }
    }
}
