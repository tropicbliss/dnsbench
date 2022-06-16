use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    net::{IpAddr, SocketAddr, UdpSocket},
    path::Path,
    str::FromStr,
    time::{Duration, Instant},
};
use trust_dns_client::{
    op::{Message, MessageType, OpCode, Query},
    rr::{Name, RecordType},
    serialize::binary::{BinEncodable, BinEncoder},
};

pub fn resolve(domain_name: Name, dns_server: IpAddr) -> Result<Duration> {
    let dns_server = SocketAddr::new(dns_server, 53);
    let mut request_as_bytes = Vec::with_capacity(512);
    let mut response_as_bytes = [0; 512];
    let mut msg = Message::new();
    msg.set_id(rand::random::<u16>())
        .set_message_type(MessageType::Query)
        .add_query(Query::query(domain_name, RecordType::A))
        .set_op_code(OpCode::Query)
        .set_recursion_desired(true);
    let mut encoder = BinEncoder::new(&mut request_as_bytes);
    msg.emit(&mut encoder)?;
    let start = Instant::now();
    let localhost = UdpSocket::bind("0.0.0.0:0").map_err(|_| DnsError::DNSError)?;
    let timeout = Duration::from_secs(3);
    localhost
        .set_read_timeout(Some(timeout))
        .map_err(|_| DnsError::DNSError)?;
    localhost.set_nonblocking(false)?;
    localhost
        .send_to(&request_as_bytes, dns_server)
        .map_err(|_| DnsError::DNSError)?;
    localhost
        .recv_from(&mut response_as_bytes)
        .map_err(|_| DnsError::DNSError)?;
    let elapsed = start.elapsed();
    let dns_message = Message::from_vec(&response_as_bytes).context("unable to parse response")?;
    for answer in dns_message.answers() {
        if answer.record_type() == RecordType::A {
            let resource = answer.data().unwrap();
            resource
                .to_ip_addr()
                .context("invalid IP address received")?;
        }
    }
    Ok(elapsed)
}

pub fn parse_dns_addrs<T: AsRef<Path>>(path: T) -> Result<Vec<IpAddr>> {
    let mut result = Vec::new();
    let file = File::open(path.as_ref())?;
    let buf_reader = BufReader::new(file);
    for addr in buf_reader.lines() {
        let addr = IpAddr::from_str(&addr?)?;
        result.push(addr);
    }
    Ok(result)
}

#[derive(Debug)]
pub enum DnsError {
    DNSError,
}

impl std::fmt::Display for DnsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for DnsError {}
