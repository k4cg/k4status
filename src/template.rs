use crate::StatusError;

pub async fn read_template(fname: &str) -> Result<spaceapi::Status, StatusError> {
    let content = tokio::fs::read_to_string(fname)
        .await
        .map_err(|e| StatusError::Template(format!("Failed to read template {} ({})", fname, e)))?;

    let status: spaceapi::Status = serde_json::from_str(&content)
        .map_err(|e| StatusError::Template(format!("Failed to parse template ({})", e)))?;

    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FILE_TEMPLATE;
    use tokio::runtime::Runtime;

    #[test]
    fn parse() {
        Runtime::new().unwrap().block_on(async {
            read_template(FILE_TEMPLATE).await.unwrap();
        });
    }
}
