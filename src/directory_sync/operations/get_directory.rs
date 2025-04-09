use async_trait::async_trait;
use thiserror::Error;

use crate::directory_sync::{Directory, DirectoryId, DirectorySync};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// An error returned from [`GetDirectory`].
#[derive(Debug, Error)]
pub enum GetDirectoryError {}

impl From<GetDirectoryError> for WorkOsError<GetDirectoryError> {
    fn from(err: GetDirectoryError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Get a Directory](https://workos.com/docs/reference/directory-sync/directory/get)
#[async_trait]
pub trait GetDirectory {
    /// Retrieves a [`Directory`] by its ID.
    ///
    /// [WorkOS Docs: Get a Directory](https://workos.com/docs/reference/directory-sync/directory/get)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos::WorkOsResult;
    /// # use workos::directory_sync::*;
    /// use workos::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), GetDirectoryError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let directory = workos
    ///     .directory_sync()
    ///     .get_directory(&DirectoryId::from("directory_01ECAZ4NV9QMV47GW873HDCX74"))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn get_directory(&self, id: &DirectoryId) -> WorkOsResult<Directory, GetDirectoryError>;
}

#[async_trait]
impl GetDirectory for DirectorySync<'_> {
    async fn get_directory(&self, id: &DirectoryId) -> WorkOsResult<Directory, GetDirectoryError> {
        let url = self
            .workos
            .base_url()
            .join(&format!("/directories/{id}", id = id))?;
        let directory = self
            .workos
            .client()
            .get(url)
            .bearer_auth(self.workos.key())
            .send()
            .await?
            .handle_unauthorized_or_generic_error()?
            .json::<Directory>()
            .await?;

        Ok(directory)
    }
}

#[cfg(test)]
mod test {
    use matches::assert_matches;
    use mockito::{self, mock};
    use serde_json::json;
    use tokio;

    use super::*;
    use crate::{ApiKey, WorkOs};

    #[tokio::test]
    async fn it_calls_the_get_directory_endpoint() {
        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&mockito::server_url())
            .unwrap()
            .build();

        let _mock = mock("GET", "/directories/directory_01ECAZ4NV9QMV47GW873HDCX74")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                  "id": "directory_01ECAZ4NV9QMV47GW873HDCX74",
                  "domain": "foo-corp.com",
                  "name": "Foo Corp",
                  "organization_id": "org_01EHZNVPK3SFK441A1RGBFSHRT",
                  "state": "unlinked",
                  "type": "gsuite directory",
                  "created_at": "2021-06-25T19:07:33.155Z",
                  "updated_at": "2021-06-25T19:07:33.155Z"
                })
                .to_string(),
            )
            .create();

        let directory = workos
            .directory_sync()
            .get_directory(&DirectoryId::from("directory_01ECAZ4NV9QMV47GW873HDCX74"))
            .await
            .unwrap();

        assert_eq!(
            directory.id,
            DirectoryId::from("directory_01ECAZ4NV9QMV47GW873HDCX74")
        )
    }

    #[tokio::test]
    async fn it_returns_an_error_when_the_get_directory_endpoint_returns_unauthorized() {
        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&mockito::server_url())
            .unwrap()
            .build();

        let _mock = mock("GET", "/directories/directory_01ECAZ4NV9QMV47GW873HDCX74")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(401)
            .with_body(
                json!({
                    "message": "Unauthorized"
                })
                .to_string(),
            )
            .create();

        let result = workos
            .directory_sync()
            .get_directory(&DirectoryId::from("directory_01ECAZ4NV9QMV47GW873HDCX74"))
            .await;

        assert_matches!(result, Err(WorkOsError::Unauthorized))
    }
}
