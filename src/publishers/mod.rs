pub mod indexers;

use crate::config::config;
use aws_sdk_sqs::Error;

async fn send_sqs_message(queue_url: &str, message: &str) -> Result<(), Error> {
    let config = config().await;
    let rsp = config.sqs_client().send_message().queue_url(queue_url).message_body(message).send().await?;
    Ok(())
}
