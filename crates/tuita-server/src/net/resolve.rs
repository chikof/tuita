//! Address classes the server never connects to, and the resolvers that enforce them.

use std::error::Error;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;

use reqwest::dns::{Addrs, Name, Resolve, Resolving};

/// Whether `ip` is not a public unicast address: loopback, private, link-local, CGNAT,
/// multicast, unspecified, unique-local, benchmarking, documentation, reserved or broadcast.
/// IPv4-mapped, NAT64 and 6to4 IPv6 addresses count as the IPv4 address they embed.
#[must_use]
pub fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => blocked_v4(v4),
        IpAddr::V6(v6) => blocked_v6(v6),
    }
}

fn blocked_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    a == 0
        || ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_broadcast()
        || (a == 100 && b & 0xc0 == 64)
        || (a == 192 && b == 0 && c <= 2)
        || (a == 198 && b & 0xfe == 18)
        || (a == 198 && b == 51 && c == 100)
        || (a == 203 && b == 0 && c == 113)
        || a >= 240
}

fn blocked_v6(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return blocked_v4(v4);
    }
    let s = ip.segments();
    let embedded = |hi: u16, lo: u16| {
        let [a, b] = hi.to_be_bytes();
        let [c, d] = lo.to_be_bytes();
        blocked_v4(Ipv4Addr::new(a, b, c, d))
    };
    // ::/96 holds `::`, `::1` and the deprecated IPv4-compatible addresses.
    s[..6] == [0; 6]
        || ip.is_multicast()
        || s[0] & 0xfe00 == 0xfc00
        || s[0] & 0xffc0 == 0xfe80
        || s[0] & 0xffc0 == 0xfec0
        || (s[0] == 0x2001 && s[1] == 0x0db8)
        || (s[..6] == [0x0064, 0xff9b, 0, 0, 0, 0] && embedded(s[6], s[7]))
        || (s[0] == 0x2002 && embedded(s[1], s[2]))
}

/// Which address classes beyond public unicast the safe client may connect to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Reach {
    /// Public unicast addresses only.
    Public,
    /// Also `127.0.0.0/8`; for tests that fetch from a local server.
    Loopback,
    /// Every address: `FEDERATION_ALLOW_PRIVATE_NETS`, for development and tests.
    Anywhere,
}

/// Whether the safe client may connect to `ip`.
pub(super) fn permits(ip: IpAddr, reach: Reach) -> bool {
    match reach {
        Reach::Anywhere => true,
        Reach::Loopback => !is_blocked_ip(ip) || matches!(ip, IpAddr::V4(v4) if v4.is_loopback()),
        Reach::Public => !is_blocked_ip(ip),
    }
}

/// Resolution found no address the client may connect to.
#[derive(Debug)]
pub(super) struct BlockedHost(Box<str>);

impl fmt::Display for BlockedHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} resolves only to blocked addresses", self.0)
    }
}

impl Error for BlockedHost {}

/// The operating system's resolver, through `tokio::net::lookup_host`.
pub(super) struct SystemResolver;

impl Resolve for SystemResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host: Box<str> = name.as_str().into();
        Box::pin(async move {
            let addrs: Vec<SocketAddr> = tokio::net::lookup_host((&*host, 0)).await?.collect();
            Ok(Box::new(addrs.into_iter()) as Addrs)
        })
    }
}

/// Wraps a resolver and drops every address [`permits`] refuses.
pub(super) struct Guarded {
    inner: Arc<dyn Resolve>,
    reach: Reach,
}

impl Guarded {
    pub(super) fn new(inner: Arc<dyn Resolve>, reach: Reach) -> Self {
        Self { inner, reach }
    }
}

impl Resolve for Guarded {
    fn resolve(&self, name: Name) -> Resolving {
        let host: Box<str> = name.as_str().into();
        let lookup = self.inner.resolve(name);
        let reach = self.reach;
        Box::pin(async move {
            let kept: Vec<SocketAddr> = lookup
                .await?
                .filter(|addr| permits(addr.ip(), reach))
                .collect();
            if kept.is_empty() {
                return Err(Box::new(BlockedHost(host)) as Box<dyn Error + Send + Sync>);
            }
            Ok(Box::new(kept.into_iter()) as Addrs)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn blocked(ip: &str) -> bool {
        is_blocked_ip(ip.parse().expect("test address"))
    }

    #[test]
    fn blocks_non_public_ipv4() {
        for ip in [
            "0.0.0.0",
            "0.1.2.3",
            "10.0.0.1",
            "10.255.255.255",
            "100.64.0.1",
            "100.127.255.255",
            "127.0.0.1",
            "127.255.255.254",
            "169.254.169.254",
            "172.16.0.1",
            "172.31.255.255",
            "192.0.0.1",
            "192.0.2.1",
            "192.168.1.1",
            "198.18.0.1",
            "198.19.255.255",
            "198.51.100.1",
            "203.0.113.1",
            "224.0.0.1",
            "239.255.255.255",
            "240.0.0.1",
            "255.255.255.255",
        ] {
            assert!(blocked(ip), "{ip} must be blocked");
        }
    }

    #[test]
    fn allows_public_ipv4() {
        for ip in [
            "1.1.1.1",
            "8.8.8.8",
            "93.184.216.34",
            "100.63.255.255",
            "100.128.0.1",
            "172.15.255.255",
            "172.32.0.1",
            "192.0.3.1",
            "198.17.255.255",
            "198.20.0.1",
            "223.255.255.255",
        ] {
            assert!(!blocked(ip), "{ip} must be allowed");
        }
    }

    #[test]
    fn blocks_non_public_ipv6() {
        for ip in [
            "::",
            "::1",
            "::2",
            "fc00::1",
            "fd12:3456::1",
            "fe80::1",
            "febf::1",
            "fec0::1",
            "ff02::1",
            "2001:db8::1",
            "::ffff:127.0.0.1",
            "::ffff:10.0.0.1",
            "::ffff:169.254.169.254",
            "::ffff:192.168.0.1",
            "64:ff9b::7f00:1",
            "64:ff9b::a9fe:a9fe",
            "2002:7f00:1::1",
            "2002:c0a8:101::",
        ] {
            assert!(blocked(ip), "{ip} must be blocked");
        }
    }

    #[test]
    fn allows_public_ipv6() {
        for ip in [
            "2606:4700:4700::1111",
            "2001:4860:4860::8888",
            "::ffff:8.8.8.8",
            "64:ff9b::808:808",
            "2002:808:808::1",
        ] {
            assert!(!blocked(ip), "{ip} must be allowed");
        }
    }

    #[test]
    fn loopback_exception_covers_ipv4_loopback_only() {
        let lo = "127.0.0.1".parse().expect("test address");
        let private = "10.0.0.1".parse().expect("test address");
        let lo6 = "::1".parse().expect("test address");
        assert!(permits(lo, Reach::Loopback));
        assert!(!permits(lo, Reach::Public));
        assert!(!permits(private, Reach::Loopback));
        assert!(!permits(lo6, Reach::Loopback));
        for ip in [lo, private, lo6] {
            assert!(permits(ip, Reach::Anywhere));
        }
    }
}
