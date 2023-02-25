#[macro_export]
macro_rules! impl_store {
    ($($name:ty),+ $(,)?) => ($(
        use async_trait::async_trait;
        use anyhow::{Context as AnyhyowContext};

        use $crate::store::Store;

        #[async_trait]
        impl Store for $name {
            async fn new() -> Result<Self> {
                let path = Self::path()?;

                if fs::metadata(path.clone()).await.is_err() {
                    return Self::default().save().await;
                }

                let mut file = File::open(path.clone())
                    .await
                    .context("Error opening file")?;

                let mut buffer = String::new();
                file.read_to_string(&mut buffer).await?;

                serde_json::from_str(&buffer).context("Failed to deserialize")
            }

            async fn save(&self) -> Result<Self> {
                let path = Self::path()?;

                fs::create_dir_all(path.parent().context("Failed to get store directory")?)
                    .await
                    .context("Failed to create store directory")?;

                let mut file = File::create(path.clone())
                    .await
                    .context("Error opening file")?;

                file.write_all(
                    serde_json::to_string(&self)
                        .context("Failed to serialize")?
                        .as_bytes(),
                )
                .await
                .context("Failed to write store")?;

                Ok(self.clone())
            }
        }
    )+)
}
