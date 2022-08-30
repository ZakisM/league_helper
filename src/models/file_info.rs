use std::fs::Metadata;
use std::path::Path;
use tokio::io::AsyncReadExt;

use crate::Result;

#[derive(Debug)]
pub struct FileInfo {
    pub metadata: Metadata,
    pub data: Vec<u8>,
}

impl FileInfo {
    pub async fn read(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = tokio::fs::File::open(path).await?;
        let metadata = file.metadata().await?;

        let mut data = Vec::with_capacity(metadata.len().try_into().unwrap_or(0));

        file.read_to_end(&mut data).await?;

        Ok(Self { metadata, data })
    }
}
