use crate::{utils, CoordinatePosition, Problem, ProblemAtPosition, ProblemPosition, Valid};
use geo_types::Line;

impl Valid for Line {
    fn is_valid(&self) -> bool {
        if utils::check_coord_is_not_finite(&self.start)
            || utils::check_coord_is_not_finite(&self.end)
        {
            return false;
        }
        true
    }
    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>> {
        let mut reason = Vec::new();

        if utils::check_coord_is_not_finite(&self.start) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Line(CoordinatePosition(0)),
            ));
        }
        if utils::check_coord_is_not_finite(&self.end) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Line(CoordinatePosition(1)),
            ));
        }

        if reason.is_empty() {
            None
        } else {
            Some(reason)
        }
    }
}
