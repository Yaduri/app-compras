//! Formatação e leitura de números no padrão brasileiro

/// Formata um valor em reais: 1234.5 -> "R$ 1.234,50"
pub fn brl(value: f64) -> String {
    let value = if value.is_finite() { value } else { 0.0 };
    let cents = (value.abs() * 100.0).round() as u64;
    let integer = (cents / 100).to_string();
    let fraction = cents % 100;

    let mut grouped = String::with_capacity(integer.len() + integer.len() / 3);
    for (i, ch) in integer.chars().enumerate() {
        if i > 0 && (integer.len() - i) % 3 == 0 {
            grouped.push('.');
        }
        grouped.push(ch);
    }

    let sign = if value < 0.0 && cents > 0 { "-" } else { "" };
    format!("{}R$ {},{:02}", sign, grouped, fraction)
}

/// Formata quantidades sem zeros desnecessários: 1.0 -> "1", 1.5 -> "1,5"
pub fn quantity(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if rounded.fract() == 0.0 {
        format!("{}", rounded as i64)
    } else {
        format!("{:.2}", rounded)
            .trim_end_matches('0')
            .replace('.', ",")
    }
}

/// Lê um número digitado pelo usuário aceitando "12,50", "12.50", "1.234,56" e "R$ 10"
pub fn parse_decimal(input: &str) -> Option<f64> {
    let cleaned: String = input
        .trim()
        .trim_start_matches("R$")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    if cleaned.is_empty() {
        return None;
    }

    let normalized = if cleaned.contains(',') {
        // Vírgula é o separador decimal; pontos são de milhar
        cleaned.replace('.', "").replace(',', ".")
    } else {
        cleaned
    };

    normalized.parse::<f64>().ok().filter(|v| v.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_brl() {
        assert_eq!(brl(0.0), "R$ 0,00");
        assert_eq!(brl(12.5), "R$ 12,50");
        assert_eq!(brl(1234.567), "R$ 1.234,57");
        assert_eq!(brl(1_000_000.0), "R$ 1.000.000,00");
        assert_eq!(brl(-3.2), "-R$ 3,20");
    }

    #[test]
    fn formats_quantity() {
        assert_eq!(quantity(1.0), "1");
        assert_eq!(quantity(1.5), "1,5");
        assert_eq!(quantity(0.25), "0,25");
    }

    #[test]
    fn parses_decimal() {
        assert_eq!(parse_decimal("12,50"), Some(12.5));
        assert_eq!(parse_decimal("12.50"), Some(12.5));
        assert_eq!(parse_decimal("1.234,56"), Some(1234.56));
        assert_eq!(parse_decimal("R$ 10"), Some(10.0));
        assert_eq!(parse_decimal(""), None);
        assert_eq!(parse_decimal("abc"), None);
    }
}
