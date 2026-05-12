pub trait Approach: Copy {
    #[allow(dead_code)]
    fn approach(self, to: Self, speed: Self) -> Self;

    /// In degrees
    fn approach_angle(self, to: Self, speed: Self) -> Self;
}

macro_rules! impl_approach_float {
    ($t:ty) => {
        impl Approach for $t {
            fn approach(self, to: Self, speed: Self) -> Self {
                assert!(speed > 0.0, "Approach speed has to be positive!");

                let diff = to - self;
                if diff.abs() <= speed {
                    to
                } else {
                    self + diff.signum() * speed
                }
            }

            fn approach_angle(self, to: Self, speed: Self) -> Self {
                assert!(speed > 0.0, "Approach speed has to be positive!");

                let diff = (to - self) % 360.0;
                let dist = (2.0 * diff) % 360.0 - diff;
                if dist.abs() <= speed {
                    (to + 360.0) % 360.0
                } else {
                    (self + dist.signum() * speed + 360.0) % 360.0
                }
            }
        }
    };
}

impl_approach_float!(f32);
impl_approach_float!(f64);

#[cfg(test)]
mod tests {
    use super::*;
    use godot::builtin::math::assert_eq_approx;

    #[test]
    #[should_panic(expected = "Approach speed has to be positive!")]
    fn approach_positive_speed() {
        1.0.approach(10.0, -1.0);
    }

    #[test]
    fn f32_approach() {
        let value = 0.0f32;
        assert_eq_approx!(value.approach(1.0, 1.0), 1.0);
        assert_eq_approx!(value.approach(1.0, 2.0), 1.0);
        assert_eq_approx!(value.approach(2.0, 1.0), 1.0);
        assert_eq_approx!(value.approach(2.0, 0.5), 0.5);
        assert_eq_approx!(value.approach(-1.0, 1.0), -1.0);
        assert_eq_approx!(value.approach(-1.0, 2.0), -1.0);
        assert_eq_approx!(value.approach(-2.0, 1.0), -1.0);
        assert_eq_approx!(value.approach(-2.0, 0.5), -0.5);
    }

    #[test]
    fn f64_approach() {
        let value = 0.0f64;
        assert_eq_approx!(value.approach(1.0, 1.0), 1.0);
        assert_eq_approx!(value.approach(1.0, 2.0), 1.0);
        assert_eq_approx!(value.approach(2.0, 1.0), 1.0);
        assert_eq_approx!(value.approach(2.0, 0.5), 0.5);
        assert_eq_approx!(value.approach(-1.0, 1.0), -1.0);
        assert_eq_approx!(value.approach(-1.0, 2.0), -1.0);
        assert_eq_approx!(value.approach(-2.0, 1.0), -1.0);
        assert_eq_approx!(value.approach(-2.0, 0.5), -0.5);
    }

    #[test]
    fn f32_approach_angle_positives() {
        let value = 10.0f32;
        assert_eq_approx!(value.approach_angle(0.0, 10.0), 0.0);
        assert_eq_approx!(value.approach_angle(20.0, 10.0), 20.0);
        assert_eq_approx!(value.approach_angle(180.0, 30.0), 40.0);
    }

    #[test]
    fn f32_approach_angle_negatives() {
        let value = -10.0f32;
        assert_eq_approx!(value.approach_angle(-0.0, 10.0), 0.0);
        assert_eq_approx!(value.approach_angle(-20.0, 10.0), 340.0);
        assert_eq_approx!(value.approach_angle(-180.0, 30.0), 320.0);
    }

    #[test]
    fn f32_approach_angle_positives_wrap() {
        let value = 10.0f32;
        assert_eq_approx!(value.approach_angle(350.0, 10.0), 0.0);
        assert_eq_approx!(value.approach_angle(340.0, 20.0), 350.0);
    }

    #[test]
    fn f32_approach_angle_negatives_wrap() {
        let value = -10.0f32;
        assert_eq_approx!(value.approach_angle(-350.0, 10.0), 0.0);
        assert_eq_approx!(value.approach_angle(-340.0, 20.0), 10.0);
    }

    #[test]
    fn f32_approach_angle_mixed() {
        let value = 10.0f32;
        assert_eq_approx!(value.approach_angle(-10.0, 10.0), 0.0);
        assert_eq_approx!(value.approach_angle(-20.0, 20.0), 350.0);

        let value = 170.0f32;
        assert_eq_approx!(value.approach_angle(-170.0, 10.0), 180.0);
        assert_eq_approx!(value.approach_angle(-170.0, 20.0), 190.0);

        let value = 170.0f32;
        assert_eq_approx!(value.approach_angle(-180.0, 5.0), 175.0);
        assert_eq_approx!(value.approach_angle(-180.0, 10.0), 180.0);
    }
}
