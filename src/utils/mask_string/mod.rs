pub fn mask_secret(secret: &str, visible_prefix: usize) -> String {
    if secret.is_empty() {
        return "<empty>".into();
    }
    let prefix: String = secret.chars().take(visible_prefix).collect();
    let masked_len = secret.chars().count().saturating_sub(prefix.chars().count());
    let stars = "*".repeat(masked_len.max(4));
    format!("{prefix}{stars}")
}

pub fn mask_email(email: &str) -> String {
    let Some((local, domain)) = email.split_once('@') else {
        return mask_secret(email, 2);
    };
    let n = local.chars().count();
    let keep = (n + 1) / 2; // ceil half
    let kept: String = local.chars().take(keep.max(1)).collect();
    let stars = "*".repeat(n.saturating_sub(keep).max(1));
    format!("{kept}{stars}@{domain}")
}

