use std::env;

use anyhow::{Error, Result};

#[tokio::main]
async fn main() -> Result<(), Error> {
    rclrs::init(env::args()).await?;

    let node = rclrs::create_node_with_default_context("signal_handler_example")?;

    let publisher =
        node.create_publisher::<std_msgs::msg::String>("topic", rclrs::QOS_PROFILE_DEFAULT)?;

    let mut message = std_msgs::msg::String::default();

    let mut publish_count: u32 = 1;

    while rclrs::ok() {
        message.data = format!("Hello, world! {}", publish_count);
        println!("Publishing: [{}]", message.data);
        publisher.publish(&message)?;
        publish_count += 1;
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    rclrs::shutdown()?;

    Ok(())
}
