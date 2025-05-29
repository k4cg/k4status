use crate::StatusError;
use std::path::Path;
use tokio::fs;

pub type Image = Vec<u8>;

const ICON_OPEN: &str = "open.png";
const ICON_CLOSED: &str = "closed.png";

#[derive(Clone)]
pub struct Icons {
    pub open: Image,
    pub closed: Image,
}

async fn read_file(dir: &str, fname: &str) -> Result<Image, StatusError> {
    let path = Path::new(dir).join(fname);
    fs::read(&path)
        .await
        .map_err(|e| StatusError::File(format!("Failed to read {} ({})", path.display(), e)))
}

impl Icons {
    pub async fn new(dir: &str) -> Result<Self, StatusError> {
        Ok(Icons {
            open: read_file(dir, ICON_OPEN).await?,
            closed: read_file(dir, ICON_CLOSED).await?,
        })
    }
}
