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

1 `cargo run --example proxy --features default -- --interface wlo1`   
2. `ip rule add to a.b.c.d table 200`  
3. `ip route add default dev utun8 table 200`  

then, run `iperf3 -s` on the server side, run `iperf3 --client 158.180.83.36 -t 60` on the client side

### bench environment

#### iperf3 client config

- ISP: Oracle Public Cloud
- geo location: usa
- os: Ubuntu 22.04.4 LTS
- mm: 1G
- cpu: x86_64; 2 core; AMD EPYC 7551 32-Core Processor

#### iperf3 server config

- ISP: Oracle Public Cloud
- geo location: korea
- os: Ubuntu 20.04.6 LTS
- mm: 1G
- cpu: x86_64; 2 core; AMD EPYC 7551 32-Core Processor

#### iperf3 bench result

the `htop` shows that, when running our proxy based on tun&smoltcp, the avarage load of 2 cpu cores is about 70%

```txt
# bare tcp, without tun
[  5]   0.00-10.20  sec  48.6 MBytes  39.9 Mbits/sec                  receiver

# with tun, with offload, test1
[  5]   0.00-60.16  sec   131 MBytes  18.3 Mbits/sec                  receiver

# with tun, with offload, test2
[  5]   0.00-60.22  sec   131 MBytes  22.3 Mbits/sec                  receiver

# with tun, without offload
[  5]   0.00-60.22  sec  99.1 MBytes  13.8 Mbits/sec                  receiver

# with tun, without offload, test2
[  5]   0.00-60.15  sec   101 MBytes  14.1 Mbits/sec                  receiver
```

the improvement lead by checksum offload is pretty considerable. 