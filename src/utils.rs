pub const fn parse_bool(input: Option<&str>) -> bool {
    if let Some(inp) = input {
        return matches!(inp.as_bytes(), b"true");
    }
    false
}
pub fn parse_u32(input: Option<&str>) -> u32 {
    input.map(|s| s.parse::<u32>().unwrap_or(0)).unwrap_or(0)
}
