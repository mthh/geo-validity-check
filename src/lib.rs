use geo::Intersects;
use geo::{Contains, RemoveRepeatedPoints};
use geo_types::{
    Coord, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Point, Polygon,
};

fn check_coord_is_not_finite(geom: &Coord) -> bool {
    if geom.x.is_finite() && geom.y.is_finite() {
        return false;
    }
    true
}

fn check_too_few_points(geom: &LineString, is_ring: bool) -> bool {
    let n_pts = if is_ring { 4 } else { 2 };
    if geom.remove_repeated_points().0.len() < n_pts {
        return true;
    }
    false
}

trait Valid {
    fn is_valid(&self) -> bool;
    fn invalidity_reason(&self) -> Option<String>;
}

impl Valid for Coord {
    fn is_valid(&self) -> bool {
        if check_coord_is_not_finite(&self) {
            return false;
        }
        true
    }
    fn invalidity_reason(&self) -> Option<String> {
        let mut reason = Vec::new();

        if check_coord_is_not_finite(&self) {
            reason.push("Coordinates have to be finite numbers.".to_string());
        }

        // Return the reason(s) of invalidity, or None if valid
        if reason.is_empty() {
            None
        } else {
            Some(reason.join("\n"))
        }
    }
}

/// In PostGIS, Point don't have any validity constraint.
/// Here we choose to check that points are finite numbers (i.e. not NaN or infinite)
impl Valid for Point {
    fn is_valid(&self) -> bool {
        self.0.is_valid()
    }
    fn invalidity_reason(&self) -> Option<String> {
        self.0.invalidity_reason()
    }
}

/// In PostGIS, MultiPoint don't have any validity constraint.
/// Here we choose to check that points are finite numbers (i.e. not NaN or infinite)
impl Valid for MultiPoint {
    fn is_valid(&self) -> bool {
        for point in &self.0 {
            if !point.is_valid() {
                return false;
            }
        }
        true
    }

    fn invalidity_reason(&self) -> Option<String> {
        let mut reason = Vec::new();

        for (i, point) in self.0.iter().enumerate() {
            if check_coord_is_not_finite(&point.0) {
                reason.push(format!(
                    "Coordinates (of point {}) have to be finite numbers.",
                    i
                ));
            }
        }

        if reason.is_empty() {
            None
        } else {
            Some(reason.join("\n"))
        }
    }
}

/// In postGIS, a LineString is valid if it has at least 2 points
/// and have a non-zero length (i.e. the first and last points are not the same).
/// Here we also check that all its points are finite numbers.
impl Valid for LineString {
    fn is_valid(&self) -> bool {
        if check_too_few_points(self, false) {
            return false;
        }
        for coord in &self.0 {
            if !coord.is_valid() {
                return false;
            }
        }
        true
    }

    fn invalidity_reason(&self) -> Option<String> {
        let mut reason = Vec::new();

        // Perform the various checks
        if check_too_few_points(self, false) {
            reason.push("LineString must have at least 2 different points".to_string());
        }

        for (i, point) in self.0.iter().enumerate() {
            if check_coord_is_not_finite(point) {
                reason.push(format!(
                    "Coordinates (of point {}) have to be finite numbers.",
                    i
                ));
            }
        }

        // Return the reason(s) of invalidity, or None if valid
        if reason.is_empty() {
            None
        } else {
            Some(reason.join("\n"))
        }
    }
}

/// MultiLineString is valid if all its LineStrings are valid.
impl Valid for MultiLineString {
    fn is_valid(&self) -> bool {
        for line in &self.0 {
            if !line.is_valid() {
                return false;
            }
        }
        true
    }
    fn invalidity_reason(&self) -> Option<String> {
        let mut reason = Vec::new();

        for (j, line) in self.0.iter().enumerate() {
            // Perform the various checks
            if check_too_few_points(line, false) {
                reason.push(format!("LineString {} must have at least 2 points", j));
            }

            for (i, point) in line.0.iter().enumerate() {
                if check_coord_is_not_finite(point) {
                    reason.push(format!(
                        "Coordinates (of point {}, on LineString {}) have to be finite numbers.",
                        i, j
                    ));
                }
            }
        }
        // Return the reason(s) of invalidity, or None if valid
        if reason.is_empty() {
            None
        } else {
            Some(reason.join("\n"))
        }
    }
}

/// Polygon must follow the following rules to be valid:
/// - [ ] the polygon boundary rings (the exterior shell ring and interior hole rings) are simple (do not cross or self-touch). Because of this a polygon cannnot have cut lines, spikes or loops. This implies that polygon holes must be represented as interior rings, rather than by the exterior ring self-touching (a so-called "inverted hole").
/// - [ ] boundary rings do not cross
/// - [ ] boundary rings may touch at points but only as a tangent (i.e. not in a line)
/// - [x] interior rings are contained in the exterior ring
/// - [ ] the polygon interior is simply connected (i.e. the rings must not touch in a way that splits the polygon into more than one part)
impl Valid for Polygon {
    fn is_valid(&self) -> bool {
        for ring in self.interiors().iter().chain([self.exterior()]) {
            if check_too_few_points(ring, true) {
                return false;
            }
            for coord in ring {
                if !coord.is_valid() {
                    return false;
                }
            }
        }
        for interior in self.interiors() {
            if !self.exterior().contains(interior) {
                return false;
            }
            for interior2 in self.interiors() {
                if interior != interior2 && interior.intersects(interior2) {
                    return false;
                }
            }
            // if self.exterior().intersects(interior) {
            //     return false;
            // }
        }

        true
    }
    fn invalidity_reason(&self) -> Option<String> {
        let mut reason = Vec::new();

        for (j, ring) in self.interiors().iter().chain([self.exterior()]).enumerate() {
            let role = if j == 0 { "Exterior" } else { "Interior" };
            // Perform the various checks
            if check_too_few_points(ring, true) {
                reason.push(format!(
                    "{} ring {}must have at least 3 different points",
                    role,
                    if j == 0 {
                        "".to_string()
                    } else {
                        format!("{} ", j - 1)
                    }
                ));
            }

            for (i, point) in ring.0.iter().enumerate() {
                if check_coord_is_not_finite(point) {
                    reason.push(format!(
                        "Coordinates (of point {}, on {} ring{}) have to be finite numbers.",
                        i,
                        role,
                        if j == 0 {
                            "".to_string()
                        } else {
                            format!("{} ", j - 1)
                        }
                    ));
                }
            }
        }

        for (j, interior) in self.interiors().iter().enumerate() {
            if !self.exterior().contains(interior) {
                reason.push(format!(
                    "Interior ring {} must be contained in the exterior ring",
                    j
                ));
            }
            for (i, interior2) in self.interiors().iter().enumerate() {
                if j != i && interior.intersects(interior2) {
                    reason.push(format!(
                        "Interior ring {} must not intersect the interior ring {}",
                        j, i,
                    ));
                }
            }
            // if self.exterior().intersects(interior) {
            //     reason.push(format!(
            //         "Interior ring {} must not intersect the exterior ring",
            //         j
            //     ));
            // }
        }

        // Return the reason(s) of invalidity, or None if valid
        if reason.is_empty() {
            None
        } else {
            Some(reason.join("\n"))
        }
    }
}

/// MultiPolygon is valid if:
/// - [x] all its polygons are valid,
/// - [x] elements do not overlaps (i.e. their interiors must not intersect)
/// - [ ] elements touch only at points
impl Valid for MultiPolygon {
    fn is_valid(&self) -> bool {
        for (j, pol) in self.0.iter().enumerate() {
            if !pol.is_valid() {
                return false;
            }
            for (i, pol2) in self.0.iter().enumerate() {
                if j != i && pol.intersects(pol2) {
                    return false;
                }
            }
        }
        true
    }
    fn invalidity_reason(&self) -> Option<String> {
        None
    }
}

impl Valid for GeometryCollection {
    fn is_valid(&self) -> bool {
        // for geom in self.0.iter() {
        //     if !geom.is_valid() {
        //         return false;
        //     }
        // }
        true
    }
    fn invalidity_reason(&self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::Geometry;

    #[test]
    fn test_point_valid() {
        let p = Point::new(0., 0.);
        assert!(p.is_valid());
        assert!(p.invalidity_reason().is_none());
    }

    #[test]
    fn test_point_invalid() {
        let p = Point::new(f64::NAN, f64::NAN);
        assert!(!p.is_valid());
        assert_eq!(
            p.invalidity_reason(),
            Some("Coordinates have to be finite numbers.".to_string())
        );
    }

    #[test]
    fn test_multipoint() {
        let mp = MultiPoint(vec![Point::new(0., 0.), Point::new(1., 1.)]);
        assert!(mp.is_valid());
        assert!(mp.invalidity_reason().is_none());
    }

    #[test]
    fn test_linestring_valid() {
        let ls = LineString(vec![Coord { x: 0., y: 0. }, Coord { x: 1., y: 1. }]);
        assert!(ls.is_valid());
        assert!(ls.invalidity_reason().is_none());
    }

    #[test]
    fn test_linestring_invalid_too_few_points_without_duplicate() {
        let ls = LineString(vec![Coord { x: 0., y: 0. }]);
        assert!(!ls.is_valid());
        assert_eq!(
            ls.invalidity_reason(),
            Some("LineString must have at least 2 different points".to_string())
        );
    }

    #[test]
    fn test_linestring_invalid_too_few_points_with_duplicate() {
        let ls = LineString(vec![Coord { x: 0., y: 0. }, Coord { x: 0., y: 0. }]);
        assert!(!ls.is_valid());
        assert_eq!(
            ls.invalidity_reason(),
            Some("LineString must have at least 2 different points".to_string())
        );
    }

    #[test]
    fn test_multilinestring_valid() {
        let mls = MultiLineString(vec![
            LineString(vec![Coord { x: 0., y: 0. }, Coord { x: 1., y: 1. }]),
            LineString(vec![Coord { x: 3., y: 1. }, Coord { x: 4., y: 1. }]),
        ]);
        assert!(mls.is_valid());
        assert!(mls.invalidity_reason().is_none());
    }

    #[test]
    fn test_polygon_valid() {
        // Unclosed rings are automatically closed by geo_types
        // so the following should be valid
        let p = Polygon::new(
            LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 1., y: 1. },
                Coord { x: 0., y: 1. },
            ]),
            vec![],
        );
        assert!(p.is_valid());
        assert!(p.invalidity_reason().is_none());
    }

    #[test]
    fn test_polygon_invalid_too_few_point_exterior_ring() {
        // Unclosed rings are automatically closed by geo_types
        // but there is still two few points in this ring
        // to be a non-empty polygon
        let p = Polygon::new(
            LineString(vec![Coord { x: 0., y: 0. }, Coord { x: 1., y: 1. }]),
            vec![],
        );
        assert!(!p.is_valid());
        assert_eq!(
            p.invalidity_reason(),
            Some("Exterior ring must have at least 3 different points".to_string())
        );
    }

    #[test]
    fn test_polygon_invalid_interior_intersect_exterior() {
        // Unclosed rings are automatically closed by geo_types
        // but there is still two few points in this ring
        // to be a non-empty polygon
        let p = Polygon::new(
            LineString::from(vec![
                (0.5, 0.5),
                (3., 0.5),
                (3., 2.5),
                (0.5, 2.5),
                (0.5, 0.5),
            ]),
            vec![LineString::from(vec![
                (1., 1.),
                (1., 2.),
                (2.5, 2.),
                (3.5, 1.),
                (1., 1.),
            ])],
        );
        assert!(!p.is_valid());
        assert_eq!(
            p.invalidity_reason(),
            Some("Interior ring 0 must be contained in the exterior ring".to_string())
        );
    }

    #[test]
    fn test_multipolygon() {
        //
        let mp = MultiPolygon(vec![
            Polygon::new(
                LineString::from(vec![
                    (0.5, 0.5),
                    (3., 0.5),
                    (3., 2.5),
                    (0.5, 2.5),
                    (0.5, 0.5),
                ]),
                vec![LineString::from(vec![
                    (1., 1.),
                    (1., 2.),
                    (2.5, 2.),
                    (3.5, 1.),
                    (1., 1.),
                ])],
            ),
            Polygon::new(
                LineString::from(vec![
                    (0.5, 0.5),
                    (3., 0.5),
                    (3., 2.5),
                    (0.5, 2.5),
                    (0.5, 0.5),
                ]),
                vec![LineString::from(vec![
                    (1., 1.),
                    (1., 2.),
                    (2.5, 2.),
                    (3.5, 1.),
                    (1., 1.),
                ])],
            ),
        ]);
        assert!(!mp.is_valid());
        // assert!(!mp.invalidity_reason().is_none());
    }

    #[test]
    fn test_geometrycollection() {
        let gc = GeometryCollection(vec![Geometry::Point(Point::new(0., 0.))]);
        assert!(gc.is_valid());
        assert!(gc.invalidity_reason().is_none());
    }
}
