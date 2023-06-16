use std::env;
use std::sync::Arc;

use anyhow::{Error, Result};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let context_p = Arc::new(rclrs::Context::new(env::args())?);
    let context = Arc::clone(&context_p);

    rclrs::install_signal_handler(Arc::clone(&context_p)).await;

    let node = rclrs::create_node(&context, "signal_handler_example")?;

    let publisher =
        node.create_publisher::<std_msgs::msg::String>("topic", rclrs::QOS_PROFILE_DEFAULT)?;

    let mut message = std_msgs::msg::String::default();

    let mut publish_count: u32 = 1;

    println!("Press ctrl-c whenever you want to exit");
    while context.ok() {
        message.data = format!("Hello, world! {}", publish_count);
        println!("Publishing: [{}]", message.data);
        publisher.publish(&message)?;
        publish_count += 1;
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    Ok(())
}
