use serde::{Deserialize, Deserializer};

pub fn csv_to_vec<'de, D>(de: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(de)?;
    let vec: Vec<String> = s
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .map(|x| x.to_string())
        .collect();
    Ok(if vec.is_empty() { None } else { Some(vec) })
}
