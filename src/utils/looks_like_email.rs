pub fn looks_like_email(s: &str) -> bool {
    s.contains('@') && s.split('@').all(|part| !part.is_empty()) && s.len() >= 5
}
