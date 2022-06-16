# DNSBench

DNSBench is a simple command line utility that benchmarks DNS servers to determine the fastest round-trip time out of each of them. DNS lookup is a pivotal part of today's internet as DNS servers are the phonebooks of the internet. Each time you visit a webpage, your browser sends a query to a DNS server and it returns the IP address of the website's origin server you are trying to visit. If this DNS resolving process takes a long time, this can result in a degraded experience for the user.

## Compiling from source

If you are on another platform, compile the binary yourself to try it out:

```sh
git clone https://github.com/tropicbliss/dnsbench
cd dnsbench
cargo build --release
```

Compiling from source requires the latest stable version of Rust. Older Rust versions may be able to compile `buckshot`, but they are not guaranteed to keep working.

The binary will be located in `target/release`.

Alternatively:

```sh
cargo install dnsbench
```

## Usage

```
USAGE:
    dnsbench.exe [OPTIONS] --domain-name <DOMAIN_NAME> --file <FILE>

OPTIONS:
    -a, --attempts <ATTEMPTS>          Number of attempts [default: 10]
    -d, --domain-name <DOMAIN_NAME>    Dummy domain name to lookup
    -f, --file <FILE>                  File containing newline delimited DNS addresses to measure
    -h, --help                         Print help information
    -r, --rate-limit <RATE_LIMIT>      Rate limited delay between each query of the same DNS server
                                       in seconds [default: 5]
    -V, --version                      Print version information
```

Before running this program, you must create a file that contains the IP addresses of the DNS servers you want to benchmark. Each IP address should be on a separate line.

```
# ip.txt
1.1.1.1
8.8.8.8
```

### Example

- Passing the path of the IP address text file (`ip.txt`) as a command line argument and using `www.wikipedia.org` as a dummy domain to test against.

```sh
./dnsbench -d www.wikipedia.org -f ip.txt
```
