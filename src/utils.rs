use geo::Intersects;
use geo::RemoveRepeatedPoints;
use geo_types::{Coord, LineString};

pub(crate) fn check_coord_is_not_finite(geom: &Coord) -> bool {
    if geom.x.is_finite() && geom.y.is_finite() {
        return false;
    }
    true
}

pub(crate) fn check_points_are_collinear(p0: &Coord, p1: &Coord, p2: &Coord) -> bool {
    let a = p1.x - p0.x;
    let b = p1.y - p0.y;
    let c = p2.x - p0.x;
    let d = p2.y - p0.y;
    let det = a * d - b * c;
    if det.abs() < 1e-10 {
        return true;
    }
    false
}

pub(crate) fn check_too_few_points(geom: &LineString, is_ring: bool) -> bool {
    let n_pts = if is_ring { 4 } else { 2 };
    if geom.remove_repeated_points().0.len() < n_pts {
        return true;
    }
    false
}

pub(crate) fn linestring_has_self_intersection(geom: &LineString) -> bool {
    // This need more test to see if we detect "spikes" correctly.
    // Maybe we could also use https://docs.rs/geo/latest/geo/algorithm/line_intersection/fn.line_intersection.html
    // to compute the intersection, see if it is a single point or not, etc.
    for (i, line) in geom.lines().enumerate() {
        for (j, other_line) in geom.lines().enumerate() {
            if i != j {
                if line.intersects(&other_line)
                    && line.start != other_line.end
                    && line.end != other_line.start
                {
                    return true;
                }
            }
        }
    }
    false
}
