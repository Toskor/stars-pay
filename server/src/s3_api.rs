use anyhow::Result;

use aws_config::BehaviorVersion;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use dotenv_codegen::dotenv;

use crate::app_state::AppState;

impl AppState {
    pub async fn put_file_to_s3(
        &self,
        file: Vec<u8>,
        content_type: &str,
        s3_path: &str,
    ) -> Result<()> {
        let bucket_name = dotenv!("S3_BUCKET_NAME");
        self.s3_client
            .put_object()
            .bucket(bucket_name)
            .key(s3_path)
            .body(ByteStream::from(file))
            .content_type(content_type)
            .send()
            .await?;
        Ok(())
    }

    pub async fn file_exists_in_s3(&self, s3_path: &str) -> Result<bool> {
        let bucket_name = dotenv!("S3_BUCKET_NAME");

        match self
            .s3_client
            .head_object()
            .bucket(bucket_name)
            .key(s3_path)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(err) => {
                use aws_sdk_s3::error::SdkError;
                use aws_sdk_s3::operation::head_object::HeadObjectError;

                match err {
                    SdkError::ServiceError(service_err) => match service_err.err() {
                        HeadObjectError::NotFound(_) => Ok(false),
                        _ => Err(anyhow::anyhow!("S3 service error: {:?}", service_err)),
                    },
                    _ => Err(anyhow::anyhow!("S3 error: {:?}", err)),
                }
            }
        }
    }

    pub async fn get_file_from_s3(&self, s3_path: &str) -> Result<Vec<u8>> {
        let bucket_name = dotenv!("S3_BUCKET_NAME");
        let file = self
            .s3_client
            .get_object()
            .bucket(bucket_name)
            .key(s3_path)
            .send()
            .await?;

        let bytes = file.body.collect().await?;
        Ok(bytes.to_vec())
    }

    pub async fn remove_file_from_s3(&self, s3_path: &str) -> Result<()> {
        let bucket_name = dotenv!("S3_BUCKET_NAME");
        self.s3_client
            .delete_object()
            .bucket(bucket_name)
            .key(s3_path)
            .send()
            .await?;
        Ok(())
    }

    /// Remove all files with the specified prefix (folder) from S3.
    /// This is equivalent to `s3cmd rm --recursive s3://bucket/prefix/`
    pub async fn remove_folder_from_s3(&self, s3_prefix: &str) -> Result<()> {
        let bucket_name = dotenv!("S3_BUCKET_NAME");

        // List all objects with the specified prefix
        let list_response = self
            .s3_client
            .list_objects_v2()
            .bucket(bucket_name)
            .prefix(s3_prefix)
            .send()
            .await?;

        // Check if there are any objects to delete
        let objects = list_response.contents();
        if objects.is_empty() {
            return Ok(());
        }

        // Prepare objects for batch deletion
        let objects_to_delete: Vec<_> = objects
            .iter()
            .filter_map(|obj| {
                obj.key().map(|key| {
                    aws_sdk_s3::types::ObjectIdentifier::builder()
                        .key(key)
                        .build()
                        .expect("Failed to build ObjectIdentifier")
                })
            })
            .collect();

        if !objects_to_delete.is_empty() {
            let delete_request = aws_sdk_s3::types::Delete::builder()
                .set_objects(Some(objects_to_delete))
                .build()
                .expect("Failed to build Delete request");

            self.s3_client
                .delete_objects()
                .bucket(bucket_name)
                .delete(delete_request)
                .send()
                .await?;
        }

        Ok(())
    }
}

pub async fn s3_client() -> Client {
    let access_key = dotenv!("S3_ACCESS_KEY");
    let secret_key = dotenv!("S3_SECRET_KEY");
    let region_name = dotenv!("S3_REGION");
    let endpoint_url = dotenv!("S3_ENDPOINT_URL");

    let credentials =
        aws_sdk_s3::config::Credentials::new(access_key, secret_key, None, None, "scaleway-s3");

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(region_name))
        .credentials_provider(credentials)
        .endpoint_url(endpoint_url)
        .load()
        .await;

    Client::new(&config)
}
