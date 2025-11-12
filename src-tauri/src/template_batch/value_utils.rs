pub(super) fn trim_trailing_zeros(input: &str) -> Option<String> {
    if let Some(point_pos) = input.find('.') {
        let mut end = input.len();
        while end > point_pos + 1 && input.as_bytes()[end - 1] == b'0' {
            end -= 1;
        }
        if end > point_pos + 1 && input.as_bytes()[end - 1] == b'.' {
            end -= 1;
        }
        if end != input.len() {
            return Some(input[..end].to_string());
        }
    }
    None
}
