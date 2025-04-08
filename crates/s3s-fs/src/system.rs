use crate::error::Result;

use s3s::S3;

use camino::Utf8Path;

#[derive(Clone)]
pub struct System {}

impl System {
    #[allow(clippy::unused_async, unused_variables)] // TODO
    pub async fn new(root: &Utf8Path) -> Result<Self> {
        Ok(System {})
    }

    #[allow(clippy::unused_async)] // TODO
    pub async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl S3 for System {}
