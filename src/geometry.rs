use crate::{ProblemAtPosition, Valid};
use geo_types::Geometry;

impl Valid for Geometry {
    fn is_valid(&self) -> bool {
        match self {
            Geometry::Point(e) => e.is_valid(),
            Geometry::Line(e) => e.is_valid(),
            Geometry::Rect(e) => e.is_valid(),
            Geometry::Triangle(e) => e.is_valid(),
            Geometry::LineString(e) => e.is_valid(),
            Geometry::Polygon(e) => e.is_valid(),
            Geometry::MultiPoint(e) => e.is_valid(),
            Geometry::MultiLineString(e) => e.is_valid(),
            Geometry::MultiPolygon(e) => e.is_valid(),
            Geometry::GeometryCollection(e) => e.is_valid(),
        }
    }
    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>> {
        match self {
            Geometry::Point(e) => e.invalidity_reason(),
            Geometry::Line(e) => e.invalidity_reason(),
            Geometry::Rect(e) => e.invalidity_reason(),
            Geometry::Triangle(e) => e.invalidity_reason(),
            Geometry::LineString(e) => e.invalidity_reason(),
            Geometry::Polygon(e) => e.invalidity_reason(),
            Geometry::MultiPoint(e) => e.invalidity_reason(),
            Geometry::MultiLineString(e) => e.invalidity_reason(),
            Geometry::MultiPolygon(e) => e.invalidity_reason(),
            Geometry::GeometryCollection(e) => e.invalidity_reason(),
        }
    }
}
