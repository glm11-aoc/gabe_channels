use std::thread;

use gabe_channels::{Channel, ChannelType};

#[test]
fn application_channel_deadlock_test() {
    let queue = Channel::<i32>::new(ChannelType::Application, 10);

    let mut consumer_queue = queue.clone();
    let consumer_thread = thread::spawn(move || {
        for expected in 0..1000 {
            let value = consumer_queue.read().expect("Failed to receive");
            assert_eq!(
                value, expected,
                "Received value {} but expected {} (out of order)",
                value, expected
            );
        }
    });

    let mut producer_queue = queue.clone();
    let producer_thread = thread::spawn(move || {
        for i in 0..1000 {
            producer_queue.write(i).expect("Failed to send");
        }
    });

    producer_thread.join().unwrap();
    consumer_thread.join().unwrap();
}
