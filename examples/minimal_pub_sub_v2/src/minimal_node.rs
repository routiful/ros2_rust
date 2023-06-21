use std::{
    env,
    thread,
};

use anyhow::{Error, Result};

#[tokio::main]
async fn main() -> Result<(), Error> {
    rclrs::init(env::args()).await?;

    let mut node = rclrs::create_node_with_default_context("minimal_node")?;

    let publisher =
        node.create_publisher::<std_msgs::msg::String>("topic", rclrs::QOS_PROFILE_DEFAULT)?;

    let mut message = std_msgs::msg::String::default();

    let mut publish_count: u32 = 1;

    let _subscription = node.create_subscription::<std_msgs::msg::String, _>(
        "topic",
        rclrs::QOS_PROFILE_DEFAULT,
        move |msg: std_msgs::msg::String| {
            println!("I heard: '{}'", msg.data);
        },
    )?;

    thread::spawn(move || {
        _ = rclrs::spin(&node).expect("Failed to read ROS node");
    });

    while rclrs::ok() {
        message.data = format!("Hello, world! {}", publish_count);
        println!("Publishing: [{}]", message.data);
        publisher.publish(&message)?;
        publish_count += 1;
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    Ok(())
}
