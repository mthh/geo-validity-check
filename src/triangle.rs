use crate::{utils, CoordinatePosition, Problem, ProblemAtPosition, ProblemPosition, Valid};
use geo_types::Triangle;

/// As stated in geo-types/src/geometry/triangles.rs,
/// "the three vertices must not be collinear and they must be distinct"
impl Valid for Triangle {
    fn is_valid(&self) -> bool {
        if utils::check_coord_is_not_finite(&self.0)
            || utils::check_coord_is_not_finite(&self.1)
            || utils::check_coord_is_not_finite(&self.2)
        {
            return false;
        }

        if &self.0 == &self.1 || &self.1 == &self.2 || &self.2 == &self.0 {
            return false;
        }

        if utils::check_points_are_collinear(&self.0, &self.1, &self.2) {
            return false;
        }
        true
    }
    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>> {
        let mut reason = Vec::new();

        if utils::check_coord_is_not_finite(&self.0) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Triangle(CoordinatePosition(0)),
            ));
        }
        if utils::check_coord_is_not_finite(&self.1) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Triangle(CoordinatePosition(1)),
            ));
        }
        if utils::check_coord_is_not_finite(&self.2) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Triangle(CoordinatePosition(2)),
            ));
        }

        if &self.0 == &self.1 || &self.0 == &self.2 {
            reason.push(ProblemAtPosition(
                Problem::IdenticalCoords,
                ProblemPosition::Triangle(CoordinatePosition(0)),
            ));
        }

        if &self.1 == &self.2 {
            reason.push(ProblemAtPosition(
                Problem::IdenticalCoords,
                ProblemPosition::Triangle(CoordinatePosition(1)),
            ));
        }

        if utils::check_points_are_collinear(&self.0, &self.1, &self.2) {
            reason.push(ProblemAtPosition(
                Problem::CollinearCoords,
                ProblemPosition::Triangle(CoordinatePosition(-1)),
            ));
        }

        if reason.is_empty() {
            None
        } else {
            Some(reason)
        }
    }
}
