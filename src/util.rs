use fraction::Fraction;

use crate::{interpreter::Time, parser::TimeExpression};

pub fn frac_to_time(frac: &fraction::prelude::GenericFraction<u64>) -> Time {
    Time {
        num: (*frac.fract().numer().unwrap_or(&0)
            + frac.trunc().numer().unwrap_or(&0) * *frac.fract().denom().unwrap_or(&0))
            as isize
            * (if frac.is_sign_positive() { 1 } else { -1 }),
        denom: *frac.fract().denom().unwrap_or(&0) as usize,
    }
}

pub fn time_to_frac(time: &Time) -> fraction::prelude::GenericFraction<u64> {
    Fraction::new(time.num as u64, time.denom as u64)
}

pub fn time_expression_to_time(time_expression: &TimeExpression) -> Time {
    Time {
        num: (time_expression.beat * 4 + time_expression.sixteenth.unwrap_or(0)) as isize,
        denom: 16,
    }
}
