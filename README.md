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

## examples

the example `proxy` uses our crate - `netstack-smoltcp` as the userspace implementation, you can run it via 

1. `sudo cargo run --example proxy -- --interface /your/if/name` (macos & linux)
2. `cargo run run --example proxy -- --interface /your/if/name` (windows, but you need to run the terminal as admin)

but on windows, you should also have `wintun.dll` installed in `C:\Windows\System32`

after that, you can set the route table by the following instructions:

### windows(run terminal as admin)

1. `Get-NetAdapter` to check the utun8's if index, assume the index is `INDEX`
2. `route add 1.1.1.1 mask 255.255.255.255 10.10.10..2 if INDEX`

### linux

`sudo ip route add 1.1.1.1/32 dev utun8`

### macos 

`sudo route add 146.190.81.132 -interface utun8`

so now the have both the proxy program running and the routing table setup correctly, we can have a shot by running: `curl 1.1.1.1`

or, you can replace the `1.1.1.1` with your server's ipv4 address, and run the iperf3 performance test.

## benchmark 

see [benchmark result](./benchmark.md)

## License

This project is licensed under the [Apache License 2.0](./LICENSE).
