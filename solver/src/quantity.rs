//! K8s resource quantity parser — converts notation like "512Mi" to f64.
//!
//! Kubernetes uses multiple unit systems depending on resource type:
//! - CPU: millicores ("300m" = 0.3 cores)
//! - Memory/Storage: binary units ("512Mi" = 0.5 GiB, "8Gi" = 8.0 GiB)
//! - Throughput (Disk/Network): decimal units ("500M" = 500.0 MB/s)
//!
//! This module is context-aware: the same suffix "M" means MiB (memory)
//! vs MB/s (throughput) depending on the resource field being parsed.

/// Determines which unit conversion table to use for parsing.
///
/// ```
/// # use kuberina_solver::quantity::QuantityContext;
/// let ctx = QuantityContext::Memory;  // "512Mi" → 0.5 GiB
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuantityContext {
    /// CPU: "300m" → 0.3 cores, "4" → 4.0 cores
    Cpu,
    /// RAM/Storage: binary units, output in GiB.
    /// "512Mi" → 0.5, "8Gi" → 8.0, "2Ti" → 2048.0
    Memory,
    /// Disk/Network throughput: decimal units, output in MB/s.
    /// "500M" → 500.0, "10G" → 10000.0
    Throughput,
}

/// Parse a K8s resource quantity string into f64 using context-specific rules.
///
/// Accepts both bare numbers (`cpu: 0.3`) and suffixed strings (`ram: "512Mi"`).
///
/// ```
/// # use kuberina_solver::quantity::{parse_quantity, QuantityContext};
/// // CPU millicores
/// assert!((parse_quantity("300m", QuantityContext::Cpu).unwrap() - 0.3).abs() < 1e-9);
/// assert!((parse_quantity("4", QuantityContext::Cpu).unwrap() - 4.0).abs() < 1e-9);
///
/// // Memory (binary, output GiB)
/// assert!((parse_quantity("512Mi", QuantityContext::Memory).unwrap() - 0.5).abs() < 1e-9);
/// assert!((parse_quantity("8Gi", QuantityContext::Memory).unwrap() - 8.0).abs() < 1e-9);
///
/// // Throughput (decimal, output MB/s)
/// assert!((parse_quantity("500M", QuantityContext::Throughput).unwrap() - 500.0).abs() < 1e-9);
/// assert!((parse_quantity("10G", QuantityContext::Throughput).unwrap() - 10000.0).abs() < 1e-9);
/// ```
pub fn parse_quantity(val: &str, ctx: QuantityContext) -> Result<f64, String> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return Err("empty quantity string".into());
    }

    // Try bare number first (works for all contexts)
    if let Ok(n) = trimmed.parse::<f64>() {
        return Ok(n);
    }

    match ctx {
        QuantityContext::Cpu => parse_cpu(trimmed),
        QuantityContext::Memory => parse_memory(trimmed),
        QuantityContext::Throughput => parse_throughput(trimmed),
    }
}

/// Parse CPU quantity: only "m" (millicore) suffix supported.
fn parse_cpu(val: &str) -> Result<f64, String> {
    if let Some(num_str) = val.strip_suffix('m') {
        let n: f64 = num_str.parse().map_err(|_| {
            format!("invalid CPU quantity '{val}': numeric part '{num_str}' is not a number")
        })?;
        return Ok(n / 1000.0);
    }
    Err(format!(
        "unrecognized CPU suffix in '{val}', expected bare number or 'm' (millicore)"
    ))
}

/// Parse memory/storage quantity with binary suffixes, output in GiB.
fn parse_memory(val: &str) -> Result<f64, String> {
    // WHY: check longer suffixes first to avoid "Ti" matching "i" before "T"
    let (num_str, multiplier) = extract_suffix_memory(val)?;
    let n: f64 = num_str.parse().map_err(|_| {
        format!("invalid memory quantity '{val}': numeric part '{num_str}' is not a number")
    })?;
    Ok(n * multiplier)
}

/// Extract numeric part and GiB multiplier from memory suffix.
fn extract_suffix_memory(val: &str) -> Result<(&str, f64), String> {
    // Binary suffixes (IEC): Ki, Mi, Gi, Ti — output in GiB
    if let Some(n) = val.strip_suffix("Ti") {
        return Ok((n, 1024.0));
    }
    if let Some(n) = val.strip_suffix("Gi") {
        return Ok((n, 1.0));
    }
    if let Some(n) = val.strip_suffix("Mi") {
        return Ok((n, 1.0 / 1024.0));
    }
    if let Some(n) = val.strip_suffix("Ki") {
        return Ok((n, 1.0 / (1024.0 * 1024.0)));
    }
    Err(format!(
        "unrecognized memory suffix in '{val}', expected Ki/Mi/Gi/Ti or bare number"
    ))
}

/// Parse throughput quantity with decimal suffixes, output in MB/s.
fn parse_throughput(val: &str) -> Result<f64, String> {
    let (num_str, multiplier) = extract_suffix_throughput(val)?;
    let n: f64 = num_str.parse().map_err(|_| {
        format!("invalid throughput quantity '{val}': numeric part '{num_str}' is not a number")
    })?;
    Ok(n * multiplier)
}

/// Extract numeric part and MB/s multiplier from throughput suffix.
fn extract_suffix_throughput(val: &str) -> Result<(&str, f64), String> {
    // Decimal suffixes: k, M, G — output in MB/s
    if let Some(n) = val.strip_suffix('G') {
        return Ok((n, 1000.0));
    }
    if let Some(n) = val.strip_suffix('M') {
        return Ok((n, 1.0));
    }
    if let Some(n) = val.strip_suffix('k') {
        return Ok((n, 0.001));
    }
    Err(format!(
        "unrecognized throughput suffix in '{val}', expected k/M/G or bare number"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // === CPU ===

    #[test]
    fn cpu_bare_number() {
        assert!((parse_quantity("4", QuantityContext::Cpu).unwrap() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn cpu_bare_float() {
        assert!((parse_quantity("0.5", QuantityContext::Cpu).unwrap() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn cpu_millicores() {
        assert!((parse_quantity("300m", QuantityContext::Cpu).unwrap() - 0.3).abs() < 1e-9);
    }

    #[test]
    fn cpu_millicores_large() {
        assert!((parse_quantity("6000m", QuantityContext::Cpu).unwrap() - 6.0).abs() < 1e-9);
    }

    // === Memory ===

    #[test]
    fn memory_bare_number() {
        assert!((parse_quantity("8.0", QuantityContext::Memory).unwrap() - 8.0).abs() < 1e-9);
    }

    #[test]
    fn memory_kibibytes() {
        // 1048576 Ki = 1 GiB
        let result = parse_quantity("1048576Ki", QuantityContext::Memory).unwrap();
        assert!((result - 1.0).abs() < 1e-6);
    }

    #[test]
    fn memory_mebibytes() {
        assert!((parse_quantity("512Mi", QuantityContext::Memory).unwrap() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn memory_gibibytes() {
        assert!((parse_quantity("8Gi", QuantityContext::Memory).unwrap() - 8.0).abs() < 1e-9);
    }

    #[test]
    fn memory_tebibytes() {
        assert!((parse_quantity("2Ti", QuantityContext::Memory).unwrap() - 2048.0).abs() < 1e-9);
    }

    #[test]
    fn memory_256gi() {
        assert!((parse_quantity("256Gi", QuantityContext::Memory).unwrap() - 256.0).abs() < 1e-9);
    }

    // === Throughput ===

    #[test]
    fn throughput_bare_number() {
        assert!(
            (parse_quantity("500.0", QuantityContext::Throughput).unwrap() - 500.0).abs() < 1e-9
        );
    }

    #[test]
    fn throughput_kilo() {
        // 1000k = 1 MB/s
        assert!((parse_quantity("1000k", QuantityContext::Throughput).unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn throughput_mega() {
        assert!(
            (parse_quantity("500M", QuantityContext::Throughput).unwrap() - 500.0).abs() < 1e-9
        );
    }

    #[test]
    fn throughput_giga() {
        assert!(
            (parse_quantity("10G", QuantityContext::Throughput).unwrap() - 10000.0).abs() < 1e-9
        );
    }

    #[test]
    fn throughput_5g() {
        assert!((parse_quantity("5G", QuantityContext::Throughput).unwrap() - 5000.0).abs() < 1e-9);
    }

    // === Error cases ===

    #[test]
    fn empty_string_error() {
        assert!(parse_quantity("", QuantityContext::Cpu).is_err());
    }

    #[test]
    fn invalid_suffix_error() {
        let err = parse_quantity("500X", QuantityContext::Memory).unwrap_err();
        assert!(
            err.contains("unrecognized"),
            "error should describe issue: {err}"
        );
    }

    #[test]
    fn invalid_numeric_part_error() {
        let err = parse_quantity("abcMi", QuantityContext::Memory).unwrap_err();
        assert!(
            err.contains("not a number"),
            "error should mention numeric parse: {err}"
        );
    }
}
