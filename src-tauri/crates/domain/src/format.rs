//! Formatação numérica dos arquivos gerados: sem zeros à direita e sem "-0".

pub fn format_number(value: f64, decimals: u8) -> String {
    let mut s = format!("{:.*}", decimals as usize, value);
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" {
        s = "0".into();
    }
    s
}

pub fn round_to(value: f64, decimals: u8) -> f64 {
    let f = 10f64.powi(decimals as i32);
    (value * f).round() / f
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_trailing_zeros() {
        assert_eq!(format_number(7.30, 2), "7.3");
        assert_eq!(format_number(823.0, 2), "823");
        assert_eq!(format_number(-0.001, 2), "0");
        assert_eq!(format_number(-88.44, 2), "-88.44");
        assert_eq!(format_number(160.0, 0), "160");
    }
}
