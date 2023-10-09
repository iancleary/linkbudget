pub const SPEED_OF_LIGHT: f64 = 299792458.0;

#[cfg(test)]
mod tests {

    #[test]
    fn speed_of_light() {
        use super::SPEED_OF_LIGHT;

        assert_eq!(299792458.0, SPEED_OF_LIGHT);
    }
}
