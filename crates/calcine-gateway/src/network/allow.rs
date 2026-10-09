//! Which addresses may connect over the network: CIDR ranges, the private
//! ones by default.

use std::net::IpAddr;

/// Home and office networks, carrier-grade NAT (Tailscale), link-local, and
/// this machine itself.
pub const PRIVATE: &[&str] = &[
    "10.0.0.0/8",
    "172.16.0.0/12",
    "192.168.0.0/16",
    "100.64.0.0/10",
    "169.254.0.0/16",
    "127.0.0.0/8",
    "fc00::/7",
    "fe80::/10",
    "::1/128",
];

/// An address range: `192.168.1.0/24`, `fd00::/8`, or one address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    network: IpAddr,
    prefix: u8,
}

impl Range {
    pub fn parse(text: &str) -> Result<Self, String> {
        let text = text.trim();
        let invalid = || format!("{text} isn't an address or a range like 192.168.1.0/24");
        let (address, prefix) = match text.split_once('/') {
            Some((address, prefix)) => (address, Some(prefix)),
            None => (text, None),
        };
        let network: IpAddr = address.parse().map_err(|_| invalid())?;
        let max = if network.is_ipv4() { 32 } else { 128 };
        let prefix = match prefix {
            Some(prefix) => prefix.parse::<u8>().map_err(|_| invalid())?,
            None => max,
        };
        if prefix > max {
            return Err(invalid());
        }
        Ok(Self { network, prefix })
    }

    pub fn contains(&self, address: IpAddr) -> bool {
        // An IPv4 client on a dual-stack socket shows as `::ffff:a.b.c.d`.
        let address = match address {
            IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(address, IpAddr::V4),
            IpAddr::V4(_) => address,
        };
        match (self.network, address) {
            (IpAddr::V4(network), IpAddr::V4(address)) => {
                let mask = u32::MAX
                    .checked_shl(32 - u32::from(self.prefix))
                    .unwrap_or(0);
                u32::from(network) & mask == u32::from(address) & mask
            }
            (IpAddr::V6(network), IpAddr::V6(address)) => {
                let mask = u128::MAX
                    .checked_shl(128 - u32::from(self.prefix))
                    .unwrap_or(0);
                u128::from(network) & mask == u128::from(address) & mask
            }
            _ => false,
        }
    }
}

/// The ranges in `allowed`, or the private ones when it's empty. Invalid
/// entries are refused when saving, so here they're skipped.
pub fn ranges(allowed: &[String]) -> Vec<Range> {
    if allowed.is_empty() {
        return PRIVATE
            .iter()
            .filter_map(|range| Range::parse(range).ok())
            .collect();
    }
    allowed
        .iter()
        .filter_map(|range| Range::parse(range).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(text: &str) -> IpAddr {
        text.parse().unwrap()
    }

    #[test]
    fn parses_ranges_and_single_addresses() {
        assert!(Range::parse("192.168.1.0/24").is_ok());
        assert!(Range::parse("fd00::/8").is_ok());
        assert!(Range::parse("10.1.2.3").unwrap().contains(ip("10.1.2.3")));
        assert!(Range::parse("10.0.0.0/33").is_err());
        assert!(Range::parse("example.com").is_err());
        assert!(Range::parse("10.0.0.0/x").is_err());
    }

    #[test]
    fn matches_addresses() {
        let home = Range::parse("192.168.1.0/24").unwrap();
        assert!(home.contains(ip("192.168.1.42")));
        assert!(!home.contains(ip("192.168.2.42")));
        assert!(!home.contains(ip("fe80::1")));
        // IPv4 clients seen through a dual-stack socket.
        assert!(home.contains(ip("::ffff:192.168.1.42")));
        let everything = Range::parse("0.0.0.0/0").unwrap();
        assert!(everything.contains(ip("8.8.8.8")));
    }

    #[test]
    fn defaults_to_private_networks() {
        let private = ranges(&[]);
        for address in [
            "192.168.0.10",
            "10.1.1.1",
            "100.101.102.103",
            "127.0.0.1",
            "fd12::1",
        ] {
            assert!(
                private.iter().any(|range| range.contains(ip(address))),
                "{address}"
            );
        }
        assert!(!private.iter().any(|range| range.contains(ip("8.8.8.8"))));
        let only = ranges(&["10.0.0.0/8".into()]);
        assert!(!only.iter().any(|range| range.contains(ip("127.0.0.1"))));
    }
}
