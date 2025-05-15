pub fn parse_number(s: &str) -> (String, String) {
    let (negative, s) = extract_negative(s);
    let (base, exponent) = split_exponent(s);
    let number = format!("{}{}", if negative { "-" } else { "" }, base.replace('_', ""));
    (number, exponent.replace('_', ""))
}

pub fn parse_rational(s: &str) -> (String, String, String) {
    let (negative, s) = extract_negative(s);
    let (fraction, exponent) = split_exponent(s);
    let parts: Vec<&str> = fraction.split('/').collect();
    let number = format!("{}{}", if negative { "-" } else { "" }, parts[0].replace('_', ""));
    (number, parts[1].replace('_', ""), exponent.replace('_', ""))
}

fn extract_negative(s: &str) -> (bool, &str) {
    match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    }
}

fn split_exponent(s: &str) -> (&str, String) {
    if let Some(e_pos) = s.find(['e', 'E']) {
        (&s[..e_pos], s[e_pos + 1..].to_string())
    } else {
        (s, "0".to_string())
    }
}
