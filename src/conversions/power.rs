pub fn watts_to_dbm(watts: f64) -> f64 {
    10.0 * (watts.log10() + 3.0)
}

pub fn dbm_to_watts(dbm: f64) -> f64 {
    10.0_f64.powf((dbm - 30.0) / 10.0)
}

#[cfg(test)]
mod tests {

    #[test]
    fn watts_to_dbm() {
        let watts: f64 = 1.0;

        let dbm: f64 = super::watts_to_dbm(watts);

        assert_eq!(30.0, dbm);
    }

    #[test]
    fn another_watts_to_dbm() {
        let watts: f64 = 20.0;

        let dbm: f64 = super::watts_to_dbm(watts);

        // not worrying about floating point precision here
        assert_eq!(43.01029995663981, dbm);
    }

    #[test]
    fn dbm_to_watts() {
        // not worrying about floating point precision here
        let dbm: f64 = 43.010_299_956_639_805;

        let watts: f64 = super::dbm_to_watts(dbm);

        // not worrying about floating point precision here
        assert_eq!(19.99999999999997, watts);
    }

    #[test]
    fn another_dbm_to_watts() {
        let dbm: f64 = 30.0;

        let watts: f64 = super::dbm_to_watts(dbm);

        assert_eq!(1.0, watts);
    }
}
