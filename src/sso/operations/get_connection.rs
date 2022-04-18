use std::error::Error;

use async_trait::async_trait;

use crate::sso::{Connection, Sso};

#[async_trait]
pub trait GetConnection {
    async fn get_connection(&self, connection_id: &str) -> Result<Connection, Box<dyn Error>>;
}

#[async_trait]
impl<'a> GetConnection for Sso<'a> {
    async fn get_connection(&self, connection_id: &str) -> Result<Connection, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let url = self.workos.base_url().join(&format!(
            "/connections/{connection_id}",
            connection_id = connection_id
        ))?;
        let response = client
            .get(url)
            .bearer_auth(self.workos.api_key())
            .send()
            .await?;
        let connection = response.json::<Connection>().await?;

        Ok(connection)
    }
}

#[cfg(test)]
mod test {
    use crate::WorkOs;

    use super::*;

    use mockito::{self, mock};
    use serde_json::json;
    use tokio;

    #[tokio::test]
    async fn it_calls_the_get_connection_endpoint() {
        let workos = WorkOs::builder(&"sk_example_123456789")
            .base_url(&mockito::server_url())
            .unwrap()
            .build();

        let _mock = mock("GET", "/connections/conn_01E4ZCR3C56J083X43JQXF3JK5")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                  "object": "connection",
                  "id": "conn_01E4ZCR3C56J083X43JQXF3JK5",
                  "organization_id": "org_01EHWNCE74X7JSDV0X3SZ3KJNY",
                  "connection_type": "GoogleOAuth",
                  "name": "Foo Corp",
                  "state": "active",
                  "created_at": "2021-06-25T19:07:33.155Z",
                  "updated_at": "2021-06-25T19:07:33.155Z",
                  "domains": [
                    {
                      "id": "conn_domain_01EHWNFTAFCF3CQAE5A9Q0P1YB",
                      "object": "connection_domain",
                      "domain": "foo-corp.com"
                    }
                  ]
                })
                .to_string(),
            )
            .create();

        let connection = workos
            .sso()
            .get_connection("conn_01E4ZCR3C56J083X43JQXF3JK5")
            .await
            .unwrap();

        assert_eq!(connection.id, "conn_01E4ZCR3C56J083X43JQXF3JK5")
    }
}
