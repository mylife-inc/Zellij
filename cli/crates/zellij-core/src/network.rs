use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// Wildcard bindings are useful to listeners, but not browser destinations.
pub fn browser_address(host: IpAddr, port: u16) -> SocketAddr {
    let host = match host.to_canonical() {
        IpAddr::V4(ip) if ip.is_unspecified() => Ipv4Addr::LOCALHOST.into(),
        IpAddr::V6(ip) if ip.is_unspecified() => Ipv6Addr::LOCALHOST.into(),
        ip => ip,
    };
    SocketAddr::new(host, port)
}

pub fn parse_host(value: &str) -> Result<IpAddr, String> {
    if value.eq_ignore_ascii_case("localhost") {
        return Ok(Ipv4Addr::LOCALHOST.into());
    }
    value
        .parse()
        .map_err(|_| "use localhost or an IPv4/IPv6 address without a scheme, port or path".into())
}
