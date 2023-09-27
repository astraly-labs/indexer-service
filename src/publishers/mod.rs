pub mod indexers;

use aws_sdk_sqs::Error;

use crate::config::config;

async fn send_sqs_message(queue_url: &str, message: &str) -> Result<(), Error> {
    let config = config().await;
    config.sqs_client().send_message().queue_url(queue_url).message_body(message).send().await?;
    Ok(())
}
