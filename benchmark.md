# offload benchmark 

## cargo bench

run `crago bench`, you can get the similar results here below:

```txt
test wire::bench_emit_tcp          ... bench:          55 ns/iter (+/- 5)
test wire::bench_emit_udp          ... bench:          47 ns/iter (+/- 2)
test wire::bench_emit_udp_offload  ... bench:           9 ns/iter (+/- 0)
test wire::bench_parse_tcp         ... bench:          42 ns/iter (+/- 1)
test wire::bench_parse_tcp_offload ... bench:           9 ns/iter (+/- 0)
```

so, we may easily conclude that the bottleneck of ingress/egress of userspace netstack is the checksum of Layer4

## iperf3 throughput bench

### preparation

assume the iperf3 server machine's public ip is a.b.c.d

1 `cargo run --release --example proxy --features default -- --interface wlo1 > /tmp/smoltcp.log` (to redirect the output to log file is crucial to the performance)   
2. `ip rule add to a.b.c.d table 200`  
3. `ip route add default dev utun8 table 200`  

then, run `iperf3 -s` on the server side, run `iperf3 --client a.b.c.d -t 60` on the client side

### bench environment

#### machine configs

1. iperf3 client1
    - ISP: Oracle Public Cloud
    - geo location: korea
    - os: Ubuntu 20.04.6 LTS
    - mm: 1G
    - cpu: x86_64; 2 core; AMD EPYC 7551 32-Core Processor
2. iperf3 client2
    - ISP: Oracle Public Cloud
    - geo location: usa
    - os: Ubuntu 22.04.4 LTS
    - mm: 1G
    - cpu: x86_64; 2 core; AMD EPYC 7551 32-Core Processor
3. iperf3 server
    - ISP: Oracle Public Cloud
    - geo location: korea
    - os: Ubuntu 20.04.6 LTS
    - mm: 1G
    - cpu: x86_64; 2 core; AMD EPYC 7551 32-Core Processor

#### iperf3 bench result


**client1 -> server(high workload, the bottleneck is CPU & IO)**

the `htop` shows that, when running our proxy based on tun&smoltcp, the avarage load of 2 cpu cores is about 70%

```txt

[ ID] Interval           Transfer     Bitrate         Retr

# bare connection, without tun
[  5]   0.00-10.00  sec   592 MBytes   497 Mbits/sec    0             sender
[  5]   0.00-10.04  sec   589 MBytes   492 Mbits/sec                  receiver

# with tun of smoltcp, with offload
[  5]   0.00-60.01  sec  1.05 GBytes   151 Mbits/sec   84             sender
[  5]   0.00-60.05  sec  1.05 GBytes   150 Mbits/sec                  receiver

# with tun of smoltcp, without offload
[  5]   0.00-60.00  sec  1.07 GBytes   153 Mbits/sec   83             sender
[  5]   0.00-60.05  sec  1.06 GBytes   152 Mbits/sec                  receiver

# with tun of lwip, without offload(not supported yet)
[  5]   0.00-60.00  sec   536 MBytes  75.0 Mbits/sec   32             sender
[  5]   0.00-60.05  sec   535 MBytes  74.7 Mbits/sec                  receiver

```

**client2 -> server (low workload, the bottleneck is IO)**

the `htop` shows that, when running our proxy based on tun&smoltcp, the avarage load of 2 cpu cores is about 25%

```txt

[ ID] Interval           Transfer     Bitrate         Retr

# bare connection, without tun
[  5]   0.00-60.00  sec   355 MBytes  49.6 Mbits/sec  4435             sender
[  5]   0.00-60.18  sec   353 MBytes  49.2 Mbits/sec                  receiver

# with tun of smoltcp, with offload
[  5]   0.00-60.00  sec   354 MBytes  49.5 Mbits/sec   26             sender
[  5]   0.00-60.20  sec   348 MBytes  48.5 Mbits/sec                  receiver

# with tun of smoltcp, without offload
[  5]   0.00-60.00  sec   341 MBytes  47.7 Mbits/sec   15             sender
[  5]   0.00-60.18  sec   335 MBytes  46.6 Mbits/sec                  receiver

# with tun of lwip, without offload(not supported yet)
[  5]   0.00-60.00  sec   352 MBytes  49.1 Mbits/sec    4             sender
[  5]   0.00-60.18  sec   350 MBytes  48.8 Mbits/sec                  receiver

```

### conslusions

1. `netstack-smoltcp` is better than `netstack-lwip`, it's performances in high CPU workload situations are more desirable; on the platform that both netstack crate is supported, you should always consider `netstack-smoltcp`
2. the checksum offload **CANNOT** lead to a huge leap in iperf throughput, but in the debug version(not listed), the difference is noticable, i guess that's because the debug version of binary file contains more instructions, so the CPU becomes a bottleneck, and the checksum offload of TUN can alleviate this issue


### TODOs

- flamegraph analyze 