use crate::{utils, Problem, ProblemAtPosition, ProblemPosition, Valid};
use geo_types::Coord;

impl Valid for Coord {
    fn is_valid(&self) -> bool {
        if utils::check_coord_is_not_finite(self) {
            return false;
        }
        true
    }
    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>> {
        let mut reason = Vec::new();

        if utils::check_coord_is_not_finite(self) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Point,
            ));
        }

        // Return the reason(s) of invalidity, or None if valid
        if reason.is_empty() {
            None
        } else {
            Some(reason)
        }
    }
}
