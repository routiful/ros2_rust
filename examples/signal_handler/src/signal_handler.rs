use std::env;
use std::sync::{Arc, Mutex};

use anyhow::{Error, Result};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let context = Arc::new(Mutex::new(rclrs::Context::new(env::args())?));
    rclrs::install_signal_handler(Arc::clone(&context)).await;

    let node;
    {
        let context = context.lock().unwrap();
        node = rclrs::create_node(&context, "signal_handler_example")?;
    }

    let publisher =
        node.create_publisher::<std_msgs::msg::String>("topic", rclrs::QOS_PROFILE_DEFAULT)?;

    let mut message = std_msgs::msg::String::default();

    let mut publish_count: u32 = 1;

    println!("Press ctrl-c whenever you want to exit");
    while context.lock().unwrap().ok() {
        message.data = format!("Hello, world! {}", publish_count);
        println!("Publishing: [{}]", message.data);
        publisher.publish(&message)?;
        publish_count += 1;
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    Ok(())
}
