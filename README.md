# gabe_channels
A simple shared memory buffer for Rust

## Reasoning
Despite Go's many dubious design decisions (e.g. error handling), the inter-thread channels are a simple and easy to use
abstraction over sending and receiving information. I have as such set out to make a safe rust implementation (and then
extend it to allow for safe device and networked communication across the same API).

## Example Usage
```rust
fn main(){
    let queue = Channel::<i32>::new(ChannelType::Application, 10);

    let consumer_queue = queue.clone();
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

    let producer_queue = queue.clone();
    let producer_thread = thread::spawn(move || {
        for i in 0..1000 {
            producer_queue.write(i).expect("Failed to send");
        }
    });

    producer_thread.join().unwrap();
    consumer_thread.join().unwrap();
}
```

## TODO:
- Write a nice README.md
- Add support for OS and Network shared memory buffers
