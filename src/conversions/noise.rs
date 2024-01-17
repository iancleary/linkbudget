pub fn noise_temperature_from_noise_factor(noise_factor: f64) -> f64 {
    290.0 * (noise_factor - 1.0)
}

pub fn noise_temperature_from_noise_figure(noise_figure: f64) -> f64 {
    let noise_factor: f64 = noise_factor_from_noise_figure(noise_figure);
    noise_temperature_from_noise_factor(noise_factor)
}

pub fn noise_factor_from_noise_figure(noise_figure: f64) -> f64 {
    10.0_f64.powf(noise_figure / 10.0)
}

pub fn noise_factor_from_noise_temperature(noise_temperature: f64) -> f64 {
    1.0 + (noise_temperature / 290.0)
}

pub fn noise_figure_from_noise_temperature(noise_temperature: f64) -> f64 {
    let noise_factor: f64 = noise_factor_from_noise_temperature(noise_temperature);
    noise_figure_from_noise_factor(noise_factor)
}

pub fn noise_figure_from_noise_factor(noise_factor: f64) -> f64 {
    10.0_f64 * noise_factor.log10()
}

pub fn noise_power_from_bandwidth(temperature: f64, bandwidth: f64) -> f64 {
    1.38e-23 * temperature * bandwidth
}

// Noise Figure of Passive Device
// https://www.microwaves101.com/encyclopedias/noise-temperature
// "Linear passive devices have noise figure equal to their loss. Expressed in dB, the NF is equal to -S21(dB). Something with one dB loss has one dB noise figure.
// May I suggest a more refined definition of this rule? This statement is true only if the passive linear device is at room temperature. However, if it is at a different physical temperature than room temperature (or To for that matter), the noise figure will be different. If I did my calculation properly, I believe that the noise figure would be
// F = 1+(1/G-1)*Tp/To
// Where G is the gain of the device (less than or equal to 1), and Tp is the physical temperature of the device. Therefore, I would recommend that the statement should say, "Linear passive devices at room temperature have a noise figure equal to their loss. Expressed in dB, the NF is equal to -S21(dB). Something with one dB loss has one dB noise figure at room temperature." I know that the NF wouldn't change very much if the device is at a physical temperature near room temperature, but if some poor slob is working at temperatures very different than room temperature, their assumption that the NF would be equal to the loss would be incorrect.
// I hope that this helps."

#[cfg(test)]
mod tests {

    #[test]
    fn noise_temperature_from_noise_factor() {
        let noise_factor: f64 = 2.0;

        let noise_temperature: f64 = super::noise_temperature_from_noise_factor(noise_factor);

        assert_eq!(290.0, noise_temperature);
    }

    #[test]
    fn another_noise_temperature_from_noise_factor() {
        let noise_factor: f64 = 4.0;

        let noise_temperature: f64 = super::noise_temperature_from_noise_factor(noise_factor);

        assert_eq!(870.0, noise_temperature);
    }

    #[test]
    fn noise_temperature_from_noise_figure() {
        let noise_figure: f64 = 3.0;

        let noise_temperature: f64 = super::noise_temperature_from_noise_figure(noise_figure);

        assert_eq!(288.62607134097505, noise_temperature);
    }

    #[test]
    fn another_noise_temperature_from_noise_figure() {
        let noise_figure: f64 = 6.0;

        let noise_temperature: f64 = super::noise_temperature_from_noise_figure(noise_figure);

        assert_eq!(864.510794605142, noise_temperature);
    }

    #[test]
    fn noise_factor_from_noise_temperature() {
        let noise_temperature: f64 = 290.0;

        let noise_factor: f64 = super::noise_factor_from_noise_temperature(noise_temperature);

        assert_eq!(2.0, noise_factor);
    }

    #[test]
    fn another_noise_factor_from_noise_temperature() {
        let noise_temperature: f64 = 290.0;

        let noise_factor: f64 = super::noise_factor_from_noise_temperature(noise_temperature);

        assert_eq!(2.0, noise_factor);
    }

    #[test]
    fn noise_factor_from_noise_figure() {
        let noise_figure: f64 = 3.010299956639812;

        let noise_factor: f64 = super::noise_factor_from_noise_figure(noise_figure);

        assert_eq!(2.0, noise_factor);
    }

    #[test]
    fn another_noise_factor_from_noise_figure() {
        let noise_figure: f64 = 6.020599913279624;

        let noise_factor: f64 = super::noise_factor_from_noise_figure(noise_figure);

        assert_eq!(4.0, noise_factor);
    }

    #[test]
    fn noise_figure_from_noise_temperature() {
        let noise_temperature: f64 = 864.510794605142;

        let noise_figure: f64 = super::noise_figure_from_noise_temperature(noise_temperature);

        assert_eq!(6.0, noise_figure);
    }

    #[test]
    fn another_noise_figure_from_noise_temperature() {
        let noise_temperature: f64 = 290.0;

        let noise_figure: f64 = super::noise_figure_from_noise_temperature(noise_temperature);

        assert_eq!(3.010299956639812, noise_figure);
    }

    #[test]
    fn noise_figure_from_noise_factor() {
        let noise_factor: f64 = 2.0;

        let noise_figure: f64 = super::noise_figure_from_noise_factor(noise_factor);

        assert_eq!(3.010299956639812, noise_figure);
    }

    #[test]
    fn another_noise_figure_from_noise_factor() {
        let noise_factor: f64 = 4.0;

        let noise_figure: f64 = super::noise_figure_from_noise_factor(noise_factor);

        assert_eq!(6.020599913279624, noise_figure);
    }

    #[test]
    fn noise_power_from_bandwidth() {
        let bandwidth: f64 = 100.0e6;
        let temperature: f64 = 290.0;

        let noise_power: f64 = super::noise_power_from_bandwidth(temperature, bandwidth);

        let noise_power_dbm: f64 = 10.0 * (noise_power.log10() + 3.0);

        assert_eq!(-93.97722915699808, noise_power_dbm);
    }
}
