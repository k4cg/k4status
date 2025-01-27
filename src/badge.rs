use crate::StatusError;
use std::path::Path;
use tokio::fs;

const BADGE_OPEN: &str = "open.svg";
const BADGE_CLOSED: &str = "closed.svg";
const BADGE_UNKNOWN: &str = "unknown.svg";

#[derive(Clone)]
pub struct Badges {
    pub open: String,
    pub closed: String,
    pub unknown: String,
}

async fn read_file(dir: &str, fname: &str) -> Result<String, StatusError> {
    let path = Path::new(dir).join(fname);
    fs::read_to_string(&path)
        .await
        .map_err(|e| StatusError::File(format!("Failed to read {} ({})", path.display(), e)))
}

impl Badges {
    pub async fn new(dir: &str) -> Result<Self, StatusError> {
        Ok(Badges {
            open: read_file(dir, BADGE_OPEN).await?,
            closed: read_file(dir, BADGE_CLOSED).await?,
            unknown: read_file(dir, BADGE_UNKNOWN).await?,
        })
    }
}
