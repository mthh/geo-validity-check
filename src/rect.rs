use crate::{utils, CoordinatePosition, Problem, ProblemAtPosition, ProblemPosition, Valid};
use geo_types::Rect;

impl Valid for Rect {
    fn is_valid(&self) -> bool {
        if utils::check_coord_is_not_finite(&self.min())
            || utils::check_coord_is_not_finite(&self.max())
        {
            return false;
        }
        true
    }
    fn explain_invalidity(&self) -> Option<Vec<ProblemAtPosition>> {
        let mut reason = Vec::new();

        if utils::check_coord_is_not_finite(&self.min()) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Rect(CoordinatePosition(0)),
            ));
        }
        if utils::check_coord_is_not_finite(&self.max()) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Rect(CoordinatePosition(1)),
            ));
        }

        if reason.is_empty() {
            None
        } else {
            Some(reason)
        }
    }
}
