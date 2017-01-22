//! Module for SHP files
//!
//! The structure of SHP files is described in http://www.esri.com/library/whitepapers/pdfs/shapefile.pdf,
//! page 2. The struct and field names try to follow the specification, where possible.
//!
//! SHP files contain an arbitrary number of geometric data records. They are all of the same type.
//!

use std::io::{Error, ErrorKind, BufReader, Read, SeekFrom, Seek};
use std::path::Path;
use std::fs::File;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use super::{ShpFile, ShxFile, FileHeader, BoundingBoxZ};
use super::shxfile;

/// A bounding box limited to X and Y axes. For axis definitions, see the BoundinxBoxZ struct.
#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
pub struct Point {
    /// The latitude of the point.
    pub x: f64,

    /// The longitude of the point.
    pub y: f64,
}

/// A generic range from a minimum to maximum value, over a type T.
#[derive(Debug, PartialEq)]
pub struct Range<T> {
    pub minimum: T,
    pub maximum: T,
}

/// Minimum and maximum on the Measure axis.
pub type MeasureRange = Range<f64>;

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
    pub measure: f64,
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
    pub measure: f64,
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
        z_values: Vec<f64>,
        measure_range: MeasureRange,
        measures: Vec<f64>
    },

    /// See Polygon. Has additional altitude and measure coordinates.
    PolygonZ {
        bounding_box: BoundingBox,
        parts: Vec<i32>,
        points: Vec<Point>,
        z_range: ZRange,
        z_values: Vec<f64>,
        measure_range: MeasureRange,
        measures: Vec<f64>
    },

    /// See MultiPoint. Has additional altitude and measure coordinates.
    MultiPointZ {
        bounding_box: BoundingBox,
        points: Vec<Point>,
        z_range: ZRange,
        z_values: Vec<f64>,
        measure_range: MeasureRange,
        measures: Vec<f64>
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
        measure_range: MeasureRange,
        measures: Vec<f64>
    },

    /// See Polygon. Has additional measure coordinates.
    PolygonM {
        bounding_box: BoundingBox,
        parts: Vec<i32>,
        points: Vec<Point>,
        measure_range: MeasureRange,
        measures: Vec<f64>
    },

    /// See MultiPoint. Has additional measure coordinates.
    MultiPointM {
        bounding_box: BoundingBox,
        points: Vec<Point>,
        measure_range: MeasureRange,
        measures: Vec<f64>
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
        z_values: Vec<f64>,
        measure_range: MeasureRange,
        measures: Vec<f64>
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
            z_range: Range::<f64> {minimum: 0f64, maximum: 0f64},
            z: vec![],
            m_range: Range::<f64> {minimum: 0f64, maximum: 0f64},
            m: vec![],
        }
    }
}

impl BoundingBox {
    /// Returns a bounding box initialized to all zeroes.
    pub fn new() -> BoundingBox {
        BoundingBox {
            x_min: 0f64,
            y_min: 0f64,
            x_max: 0f64,
            y_max: 0f64
        }
    }

    /// Parses a bounding box by consuming four doubles from the input stream.
    pub fn parse<T: Read>(file: &mut T) -> Result<BoundingBox, Error> {
        let mut result = BoundingBox::new();

        result.x_min = try!(file.read_f64::<LittleEndian>());
        result.y_min = try!(file.read_f64::<LittleEndian>());
        result.x_max = try!(file.read_f64::<LittleEndian>());
        result.y_max = try!(file.read_f64::<LittleEndian>());

        Ok(result)
    }
}

impl Point {
    /// Returns a point initialized to (0,0)
    pub fn new() -> Point {
        Point {x: 0f64, y: 0f64}
    }

    /// Parses a file by reading two f64s in little-endian format from the input stream.
    pub fn parse<T: Read>(file: &mut T) -> Result<Point, Error> {
        let mut result = Point::new();

        result.x = try!(file.read_f64::<LittleEndian>());
        result.y = try!(file.read_f64::<LittleEndian>());

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
    pub fn new() -> Shape {
        Shape::NullShape
    }

    /// Consumes an array of num i32's from the input stream and returns them in a Vec.
    fn parse_i32_array<T: Read>(file: &mut T, num: usize) -> Result<Vec<i32>,Error> {
        let mut result: Vec<i32> = vec![];
        for _ in 0..num {
            result.push(try!(file.read_i32::<LittleEndian>()));
        }
        Ok(result)
    }

    /// Consumes an array of num points from the input stream and returns them in a Vec.
    fn parse_point_array<T: Read>(file: &mut T, num: usize) -> Result<Vec<Point>,Error> {
        let mut result: Vec<Point> = vec![];
        for _ in 0..num {
            result.push(try!(Point::parse(file)));
        }
        Ok(result)
    }

    /// Consumes an array of num f64's from the input stream and returns them in a Vec.
    fn parse_f64_array<T: Read>(file: &mut T, num: usize) -> Result<Vec<f64>,Error> {
        let mut result: Vec<f64> = vec![];
        for _ in 0..num {
            result.push(try!(file.read_f64::<LittleEndian>()));
        }
        Ok(result)
    }

    /// Gets called internally for parsing a point.
    fn parse_point_type<T: Read>(file: &mut T, shape_type: i32) -> Result<(Shape, usize), Error> {
        match shape_type {
            // Points come first
            Shape::STY_POINT => {
                // X and Y, both double and little endian
                let x = try!(file.read_f64::<LittleEndian>());
                let y = try!(file.read_f64::<LittleEndian>());
                Ok((Shape::Point {point: Point{x: x, y: y}}, 16))
            },
            Shape::STY_POINT_M => {
                // X and Y, both double and little endian
                let x = try!(file.read_f64::<LittleEndian>());
                let y = try!(file.read_f64::<LittleEndian>());
                let m = try!(file.read_f64::<LittleEndian>());
                Ok((Shape::PointM {point: PointM{x: x, y: y, measure: m}}, 24))
            },
            Shape::STY_POINT_Z => {
                // X and Y, both double and little endian
                let x = try!(file.read_f64::<LittleEndian>());
                let y = try!(file.read_f64::<LittleEndian>());
                let z = try!(file.read_f64::<LittleEndian>());
                let m = try!(file.read_f64::<LittleEndian>());
                Ok((Shape::PointZ {point: PointZ{x: x, y: y, measure: m, z: z}}, 32))
            },
            _ => Err(Error::new(ErrorKind::Other, "Supposed point not of any point type!")),
        }
    }

    /// Given the encoded ID of a patch type (see MultiPatch), returns the right enum variant for it.
    fn get_patch_type_from_id(id: &i32) -> Option<PatchType> {
        match *id {
            Shape::PTY_TRIANGLE_STRIP => {
                Some(PatchType::TriangleStrip)
            },
            Shape::PTY_TRIANGLE_FAN => {
                Some(PatchType::TriangleFan)
            },
            Shape::PTY_INNER_RING => {
                Some(PatchType::InnerRing)
            },
            Shape::PTY_OUTER_RING => {
                Some(PatchType::OuterRing)
            },
            Shape::PTY_FIRST_RING => {
                Some(PatchType::FirstRing)
            },
            Shape::PTY_RING => {
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
    fn parse_f64_range_and_array<T: Read>(file: &mut T, num: usize) -> Result<(Range<f64>, Vec<f64>), Error> {
        let min = try!(file.read_f64::<LittleEndian>());
        let max = try!(file.read_f64::<LittleEndian>());
        let range = Range::<f64> {minimum: min, maximum: max};
        let arr = try!(Shape::parse_f64_array(file, num));
        Ok((range, arr))
    }

    /// Given a Shape type ID and the parsed base data, we can already construct a valid shape
    /// object.
    fn shape_from_base_data(shape_type: i32, base: ShapeBaseData) -> Shape {
        match shape_type {
            // The poly lines
            Shape::STY_POLY_LINE => {
                Shape::PolyLine {bounding_box: base.bounding_box, parts: base.parts, points: base.points}
            },
            Shape::STY_POLY_LINE_M => {
                Shape::PolyLineM {
                    bounding_box: base.bounding_box,
                    parts: base.parts,
                    points: base.points,
                    measure_range: base.m_range,
                    measures: base.m}
            },
            Shape::STY_POLY_LINE_Z => {
                Shape::PolyLineZ {
                    bounding_box: base.bounding_box,
                    parts: base.parts,
                    points: base.points,
                    z_values: base.z,
                    z_range: base.z_range,
                    measure_range: base.m_range,
                    measures: base.m}
            },
            // The polygons
            Shape::STY_POLYGON => {
                Shape::Polygon {bounding_box: base.bounding_box, parts: base.parts, points: base.points}
            },
            Shape::STY_POLYGON_M => {
                Shape::PolygonM {
                    bounding_box: base.bounding_box,
                    parts: base.parts,
                    points: base.points,
                    measure_range: base.m_range,
                    measures: base.m}
            },
            Shape::STY_POLYGON_Z => {
                Shape::PolygonZ {
                    bounding_box: base.bounding_box,
                    parts: base.parts,
                    points: base.points,
                    z_range: base.z_range,
                    z_values: base.z,
                    measure_range: base.m_range,
                    measures: base.m}
            },
            // Then the multipoints
            Shape::STY_MULTI_POINT => {
                Shape::MultiPoint {bounding_box: base.bounding_box, points: base.points}
            },
            Shape::STY_MULTI_POINT_M => {
                Shape::MultiPointM {
                    bounding_box: base.bounding_box,
                    points: base.points,
                    measure_range: base.m_range,
                    measures: base.m}
            },
            Shape::STY_MULTI_POINT_Z => {
                Shape::MultiPointZ {
                    bounding_box: base.bounding_box,
                    points: base.points,
                    z_range: base.z_range,
                    z_values: base.z,
                    measure_range: base.m_range,
                    measures: base.m}
            },
            Shape::STY_MULTI_PATCH => {
                Shape::MultiPatch {
                    bounding_box: base.bounding_box,
                    parts: base.parts,
                    part_types: base.part_types,
                    points: base.points,
                    z_range: base.z_range,
                    z_values: base.z,
                    measure_range: base.m_range,
                    measures: base.m}
            },
            Shape::STY_NULL_SHAPE => {
                Shape::NullShape
            },
            _ => {
                // Probably a sane default
                Shape::NullShape
            }
        }
    }

    /// Parses a shape from the input stream.
    pub fn parse<T: Read>(file: &mut T) -> Result<(Shape, usize), Error> {
        let shape_type = try!(file.read_i32::<LittleEndian>());
        let mut length = 4usize;

        // Get the points out of here, they're too special
        match shape_type {
            Shape::STY_POINT
            | Shape::STY_POINT_M
            | Shape::STY_POINT_Z => {
                return Shape::parse_point_type(file, shape_type);
            },
            _ => (),
        }

        // For all types which have a bounding box, read it first.
        let mut base: ShapeBaseData = ShapeBaseData::new();

        match shape_type {
            Shape::STY_POLY_LINE
            | Shape::STY_POLYGON
            | Shape::STY_POLY_LINE_M
            | Shape::STY_POLYGON_M
            | Shape::STY_POLY_LINE_Z
            | Shape::STY_POLYGON_Z
            | Shape::STY_MULTI_PATCH => {
                length += 40usize;
                base.bounding_box = try!(BoundingBox::parse(file));
                base.num_parts = try!(file.read_i32::<LittleEndian>());
                base.num_points = try!(file.read_i32::<LittleEndian>());
                length += 4 * base.num_parts as usize;
                base.parts = try!(Shape::parse_i32_array(file, base.num_parts as usize));

                if shape_type == Shape::STY_MULTI_PATCH {
                    let patch_types_id = try!(Shape::parse_i32_array(file, base.num_parts as usize));
                    length += 4 * base.num_parts as usize;
                    base.part_types = patch_types_id.iter().map(Shape::get_patch_type_from_id).map(Option::<PatchType>::unwrap).collect();
                }

                length += 16 * base.num_points as usize;
                base.points = try!(Shape::parse_point_array(file, base.num_points as usize));
            },
            Shape::STY_MULTI_POINT
            | Shape::STY_MULTI_POINT_M
            | Shape::STY_MULTI_POINT_Z => {
                length += 36usize;
                base.bounding_box = try!(BoundingBox::parse(file));
                base.num_points = try!(file.read_i32::<LittleEndian>());
                length += 16 * base.num_points as usize;
                base.points = try!(Shape::parse_point_array(file, base.num_points as usize));
            },
            _ => ()
        };

        match shape_type {
            Shape::STY_POLY_LINE_Z
            | Shape::STY_POLYGON_Z
            | Shape::STY_MULTI_POINT_Z
            | Shape::STY_MULTI_PATCH => {
                let (z_range, z) = try!(Shape::parse_f64_range_and_array(file, base.num_points as usize));
                let (m_range, m) = try!(Shape::parse_f64_range_and_array(file, base.num_points as usize));
                base.z_range = z_range;
                base.z = z;
                base.m_range = m_range;
                base.m = m;
                length += 32usize + 16 * base.num_points as usize;
            },
            Shape::STY_POLY_LINE_M
            | Shape::STY_POLYGON_M
            | Shape::STY_MULTI_POINT_M => {
                let (m_range, m) = try!(Shape::parse_f64_range_and_array(file, base.num_points as usize));
                base.m_range = m_range;
                base.m = m;
                length += 16usize + 8 * base.num_points as usize;
            },
            _ => ()
        }

        Ok((Shape::shape_from_base_data(shape_type, base), length))
    }
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

impl BoundingBoxZ {
    /// Creates a BoundingBoxZ with all zeros
    pub fn new() -> BoundingBoxZ {
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
    pub fn parse<T: Read>(file: &mut T) -> Result<BoundingBoxZ, Error> {
        let mut result = BoundingBoxZ::new();

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

impl FileHeader {
    /// The magic number at the beginning of SHP files
    const SHP_MAGIC_NUMBER: i32 = 9994;
    /// The supported version
    const SHP_VERSION: i32 = 1000;

    /// Creates a new empty file header
    pub fn new() -> FileHeader {
        FileHeader {file_length: 0, shape_type: 0, bounding_box: BoundingBoxZ::new()}
    }

    /// Reads a file header from the given input stream
    pub fn parse<T: Read + Seek>(file: &mut T) -> Result<FileHeader, Error> {
        // Confirm magic number - Big Endian
        if try!(file.read_i32::<BigEndian>()) != FileHeader::SHP_MAGIC_NUMBER {
            return Err(Error::new(ErrorKind::Other, "SHP header magic number mismatch!"));
        }

        let mut result = FileHeader::new();

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
        if try!(file.read_i32::<LittleEndian>()) != FileHeader::SHP_VERSION {
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
    pub fn parse_header(mut self) -> Result<ShpFile, Error> {
        try!(self.file.seek(SeekFrom::Start(0)));

        // Try parsing the header
        self.header = try!(FileHeader::parse(&mut self.file));

        Ok(self)
    }

    /// Given a file name, parses the SHP file and returns the result.
    pub fn parse_file(path: &Path) -> Result<Self, Error> {
        let result = ShpFile {file: BufReader::new(try!(File::open(path))), header: FileHeader::new()};

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
        let rec: shxfile::Record;
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

#[cfg(test)]
mod tests {
    use super::Shape;
    use std::io::Cursor;
    use byteorder::{LittleEndian, WriteBytesExt};

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

        // Then see whether the data gets parsed correctly
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

        // Put 5 as shape type instead of three (structure is the same)
        let mut temp: Vec<u8> = vec![];
        let _ = temp.write_i32::<LittleEndian>(5);
        temp.extend_from_slice(&input[4..]);
        let input = temp;

        // Parse that and see whether the two are equal by fields
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
