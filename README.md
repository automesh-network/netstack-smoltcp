# Netstack Smoltcp

A netstack for the special purpose of turning packets from/to a TUN interface into TCP streams and UDP packets. It uses smoltcp-rs as the backend netstack.

## Example
```rust
use tun::{Device, TunPacket};
// let device = tun::create_as_async(&cfg)?;
// let framed = device.into_framed();

// let mut builder = StackBuilder::default();
// let (runner, udp_socket, tcp_listener, stack) = builder.build();
// tokio::task::spawn(runner);
let (udp_socket, tcp_listener, stack) = StackBuilder::default().run();

let (mut stack_sink, mut stack_stream) = stack.split();
let (mut tun_sink, mut tun_stream) = framed.split();

// Reads packet from stack and sends to TUN.
tokio::spawn(async move {
    while let Some(pkt) = stack_stream.next().await {
        if let Ok(pkt) = pkt {
            tun_sink.send(TunPacket::new(pkt)).await.unwrap();
        }
    }
});

// Reads packet from TUN and sends to stack.
tokio::spawn(async move {
    while let Some(pkt) = tun_stream.next().await {
        if let Ok(pkt) = pkt {
            stack_sink.send(pkt.into_bytes().into()).await.unwrap();
        }
    }
});

// Extracts TCP connections from stack and sends them to the dispatcher.
tokio::spawn(async move {
    handle_inbound_stream(tcp_listener).await;
});

// Receive and send UDP packets between netstack and NAT manager. The NAT
// manager would maintain UDP sessions and send them to the dispatcher.
tokio::spawn(async move {
    handle_inbound_datagram(udp_socket).await;
});
```

## License

This project is licensed under the [Apache License 2.0](./LICENSE).
