use aws_sdk_s3::{presigning::PresigningConfig, primitives::ByteStream, Client};
use std::time::Duration;

use crate::application::{errors::AppError, services::PosterStorage};

pub struct S3PosterStorage {
    client: Client,
    bucket: String,
    public_base_url: String,
}

impl S3PosterStorage {
    pub fn new(
        client: Client,
        bucket: impl Into<String>,
        public_base_url: impl Into<String>,
    ) -> Self {
        Self {
            client,
            bucket: bucket.into(),
            public_base_url: public_base_url.into(),
        }
    }

    pub async fn put_object(
        &self,
        key: &str,
        content_type: &str,
        bytes: Vec<u8>,
    ) -> Result<String, AppError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(ByteStream::from(bytes))
            .send()
            .await
            .map_err(|e| AppError::StorageError(format!("poster upload error: {e:?}")))?;

        Ok(self.public_url(key))
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), AppError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::StorageError(format!("poster delete error: {e:?}")))?;

        Ok(())
    }
}

impl PosterStorage for S3PosterStorage {
    /// Generates a pre-signed PUT URL. The client uploads the image directly to S3.
    fn presign_upload_url(
        &self,
        key: &str,
        content_type: &str,
        expires_in_secs: u32,
    ) -> Result<String, AppError> {
        // presign() is async in the AWS SDK but the trait is sync.
        // We use tokio::task::block_in_place so this can be called from
        // an async context without spawning a new thread.
        let url = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let presigning_config =
                    PresigningConfig::expires_in(Duration::from_secs(expires_in_secs as u64))
                        .map_err(|e| {
                            AppError::StorageError(format!("presign config error: {e}"))
                        })?;

                self.client
                    .put_object()
                    .bucket(&self.bucket)
                    .key(key)
                    .content_type(content_type)
                    .presigned(presigning_config)
                    .await
                    .map(|req| req.uri().to_string())
                    .map_err(|e| AppError::StorageError(format!("presign error: {e}")))
            })
        })?;

        Ok(url)
    }

    /// Constructs the public HTTPS URL for an object key.
    /// Called by the handler after the client uploads, before passing
    /// the resolved URL to the create_event use case.
    fn public_url(&self, key: &str) -> String {
        format!(
            "{}/{}",
            self.public_base_url.trim_end_matches('/'),
            key.trim_start_matches('/')
        )
    }
}
