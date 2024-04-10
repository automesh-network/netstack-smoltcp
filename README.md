# Netstack Smoltcp

A netstack for the special purpose of turning packets from/to a TUN interface into TCP streams and UDP packets. It uses smoltcp-rs as the backend netstack.

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Apache licensed, Version 2.0][apache-badge]][apache-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/netstack-smoltcp.svg
[crates-url]: https://crates.io/crates/netstack-smoltcp
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/automesh-network/netstack-smoltcp/blob/master/LICENSE-MIT
[apache-badge]: https://img.shields.io/badge/license-APACHE2.0-blue.svg
[apache-url]: https://github.com/automesh-network/netstack-smoltcp/blob/master/LICENSE-APACHE
[actions-badge]: https://github.com/automesh-network/netstack-smoltcp/workflows/CI/badge.svg
[actions-url]: https://github.com/automesh-network/netstack-smoltcp/actions?query=workflow%3ACI+branch%3Amain

## Example
```rust
// let device = tun2::create_as_async(&cfg)?;
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
            tun_sink.send(pkt).await.unwrap();
        }
    }
});

// Reads packet from TUN and sends to stack.
tokio::spawn(async move {
    while let Some(pkt) = tun_stream.next().await {
        if let Ok(pkt) = pkt {
            stack_sink.send(pkt.to_vec()).await.unwrap();
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

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in cc-rs by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
