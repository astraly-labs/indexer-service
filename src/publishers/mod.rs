pub mod indexers;

use aws_sdk_sqs::Error;

use crate::config::config;

async fn send_sqs_message_with_delay(queue_url: &str, message: &str, delay_seconds: u16) -> Result<(), Error> {
    let config = config().await;
    config
        .sqs_client()
        .send_message()
        .queue_url(queue_url)
        .message_body(message)
        .delay_seconds(delay_seconds.into())
        .send()
        .await?;
    Ok(())
}

async fn send_sqs_message(queue_url: &str, message: &str) -> Result<(), Error> {
    send_sqs_message_with_delay(queue_url, message, 0).await?;
    Ok(())
}
