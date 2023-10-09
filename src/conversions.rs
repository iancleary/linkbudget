use std::f64::consts::PI;

pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    #[test]
    fn one_hundred_eighty() {
        let radians = super::degrees_to_radians(180.0);
        assert_eq!(PI, radians);
    }

    #[test]
    fn ninety() {
        let radians = super::degrees_to_radians(90.0);
        assert_eq!(PI / 2.0, radians);
    }

    #[test]
    fn fourty_five() {
        let radians = super::degrees_to_radians(45.0);
        assert_eq!(PI / 4.0, radians);
    }

    #[test]
    fn negative_fourty_five() {
        let radians = super::degrees_to_radians(-45.0);
        assert_eq!(-PI / 4.0, radians);
    }
}
