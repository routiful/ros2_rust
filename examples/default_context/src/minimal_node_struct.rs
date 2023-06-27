use std::{
    env,
    thread,
    thread::JoinHandle,
};
use std::sync::{Arc, Mutex};

use anyhow::{Error, Result};

#[allow(dead_code)]
pub struct Publisher {
    rcl_node: rclrs::Node,
    publisher: Arc<Mutex<rclrs::Publisher<std_msgs::msg::String>>>,
}

impl Publisher {
    pub fn new(node_name: &str) -> Result<Self, rclrs::RclrsError> {
        let rcl_node = rclrs::create_node_with_default_context(node_name).unwrap();
        let publisher =
            Arc::new(Mutex::new(rcl_node.create_publisher::<std_msgs::msg::String>("chatter", rclrs::QOS_PROFILE_DEFAULT)?));

        Ok(
            Publisher {
                rcl_node,
                publisher,
            }
        )
    }

    pub fn spin(&self) -> JoinHandle<()> {
        let mut message = std_msgs::msg::String::default();
        let mut publish_count: u32 = 1;
        let publisher = Arc::clone(&self.publisher);
        thread::spawn(move || {
            while rclrs::ok() {
                message.data = format!("Hello, world! {}", publish_count);
                println!("Publishing: [{}]", message.data);
                publisher.lock().unwrap().publish(&message).unwrap();
                publish_count += 1;
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        })
    }
}

#[allow(dead_code)]
pub struct Subscription {
    rcl_node: Arc<Mutex<rclrs::Node>>,
    subscription: Arc<rclrs::Subscription<std_msgs::msg::String>>,
    msgs: Arc<Mutex<Option<std_msgs::msg::String>>>,
}

impl Subscription {
    pub fn new(node_name: &str) -> Result<Self, rclrs::RclrsError> {
        let rcl_node = Arc::new(Mutex::new(rclrs::create_node_with_default_context(node_name)?));
        let string_msgs = Arc::new(Mutex::new(None));
        let string_msgs_cb = Arc::clone(&string_msgs);
        let subscription = rcl_node.lock().unwrap().create_subscription(
            "chatter",
            rclrs::QOS_PROFILE_DEFAULT,
            move |msg: std_msgs::msg::String| {
                println!("Subscription: [{}]", msg.data);
                *string_msgs_cb.lock().unwrap() = Some(msg);
            },
        ).unwrap();

        Ok(
            Subscription {
                rcl_node,
                subscription,
                msgs: Arc::clone(&string_msgs),
            }
        )
    }

    pub fn spin(&self) -> JoinHandle<()> {
        let rcl_node = Arc::clone(&self.rcl_node);
        thread::spawn(move || {
            rclrs::spin(&rcl_node.lock().unwrap()).expect("Failed to spin ROS node");
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    rclrs::init(env::args()).await?;

    let publisher = Publisher::new("talker")?;
    let subscription = Subscription::new("listener")?;

    let publisher_thread = publisher.spin();
    let subscription_thread = subscription.spin();

    publisher_thread.join().unwrap();
    subscription_thread.join().unwrap();

    Ok(())
}
