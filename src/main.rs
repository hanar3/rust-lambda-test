use lambda_runtime::{service_fn, LambdaEvent};
use log::{info, error};
use time::OffsetDateTime;
use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
struct Request {
    pub body: String,
}

#[derive(Debug, Serialize)]
struct SuccessResponse {
    pub body: String,
}

#[derive(Debug, Serialize)]
struct FailureResponse {
    pub body: String,
}
impl std::fmt::Display for FailureResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.body)
    }
}

impl std::error::Error for FailureResponse {}

type Response = Result<SuccessResponse, FailureResponse>;

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    env_logger::init();

    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn handler(event: LambdaEvent<Request>)  -> Response {
    info!("Handling a request...");
    let bucket_name = std::env::var("BUCKET_NAME")
        .expect("A BUCKET_NAME must be set in this app's Lambda environment variables.");

    let config  = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&config);
    let filename = format!("{}.txt", OffsetDateTime::now_utc().unix_timestamp());

    let _ = s3_client
        .put_object()   
        .bucket(bucket_name)
        .body(event.payload.body.as_bytes().to_owned().into())
        .key(&filename)
        .content_type("text/plain")
        .send()
        .await
        .map_err(|err| {
            error!("failed to upload file '{}' to s3 with error {}", &filename, err);
            FailureResponse {
                body: format!("The lambda encountered an error and your message was not saved: {}", err).to_owned(),
            }
        })?;
    info!(
        "Successfully stored the incoming request in S3 with the name '{}'",
        &filename
    );
    
    Ok(SuccessResponse {
        body: format!(
            "the lambda has successfully stored the your request in S3 with name '{}'",
            filename
        ),
    })
}