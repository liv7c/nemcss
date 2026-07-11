pub fn format_number(value: f64) -> String {
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_number_drops_trailing_zero_fraction() {
        assert_eq!(format_number(1.0), "1");
        assert_eq!(format_number(16.0), "16");
    }

    #[test]
    fn format_number_keeps_meaningful_fraction() {
        assert_eq!(format_number(1.5), "1.5");
        assert_eq!(format_number(0.125), "0.125");
        assert_eq!(format_number(1.25), "1.25");
    }
}
