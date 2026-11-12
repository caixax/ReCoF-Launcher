//! The address of a server to join.
//!
//! The game joins one with `+connect <address>` on its command line. What
//! the user types goes there, so only what an address is made of gets
//! through: anything else (a space, a `;`, a `+`) would let the box run
//! other console commands.

/// The longest host name DNS allows.
const MAX_HOST: usize = 253;

/// Checks a server address as typed or pasted and gives it back in the
/// form `+connect` takes, or `None` when it is not one.
///
/// Accepts a host name or IPv4 address with an optional port
/// (`203.0.113.5:27015`, `cof.example.org`), and the whole
/// `connect 203.0.113.5:27015` line a server owner tends to share.
pub fn server_address(text: &str) -> Option<String> {
    let mut address = text.trim();
    if let Some(rest) = address
        .get(..8)
        .filter(|start| start.eq_ignore_ascii_case("connect "))
        .and_then(|_| address.get(8..))
    {
        address = rest.trim();
    }
    // The word alone, with nothing after it, is not a server.
    if address.eq_ignore_ascii_case("connect") {
        return None;
    }

    let (host, port) = match address.split_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (address, None),
    };

    let host_ok = !host.is_empty()
        && host.len() <= MAX_HOST
        && !host.starts_with(['.', '-'])
        && !host.ends_with('-')
        && !host.contains("..")
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    if !host_ok {
        return None;
    }

    match port {
        None => Some(host.to_owned()),
        Some(port) => {
            let number: u16 = port
                .parse()
                .ok()
                .filter(|_| port.chars().all(|c| c.is_ascii_digit()))?;
            (number > 0).then(|| format!("{host}:{number}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::server_address;

    #[test]
    fn takes_addresses_with_and_without_a_port() {
        assert_eq!(
            server_address("203.0.113.5:27015").as_deref(),
            Some("203.0.113.5:27015")
        );
        assert_eq!(
            server_address("203.0.113.5").as_deref(),
            Some("203.0.113.5")
        );
        assert_eq!(
            server_address("cof.example.org:27016").as_deref(),
            Some("cof.example.org:27016")
        );
        assert_eq!(server_address("localhost").as_deref(), Some("localhost"));
    }

    #[test]
    fn takes_a_shared_connect_line_and_spaces_around_it() {
        assert_eq!(
            server_address("  connect 203.0.113.5:27015 ").as_deref(),
            Some("203.0.113.5:27015")
        );
        assert_eq!(
            server_address("CONNECT  cof.example.org").as_deref(),
            Some("cof.example.org")
        );
    }

    #[test]
    fn refuses_what_would_reach_the_console() {
        for text in [
            "",
            "   ",
            "connect ",
            "203.0.113.5;quit",
            "203.0.113.5 +exec x",
            "203.0.113.5:27015;rcon",
            "+quit",
            "host name",
            "\"203.0.113.5\"",
        ] {
            assert_eq!(server_address(text), None, "{text:?}");
        }
    }

    #[test]
    fn refuses_bad_ports_and_hosts() {
        for text in [
            "203.0.113.5:",
            "203.0.113.5:0",
            "203.0.113.5:65536",
            "203.0.113.5:+27015",
            "203.0.113.5:27015:1",
            ":27015",
            ".example.org",
            "-example.org",
            "example-",
            "example..org",
            "ex\u{e4}mple.org",
            "[::1]:27015",
        ] {
            assert_eq!(server_address(text), None, "{text:?}");
        }
        assert_eq!(server_address(&"a".repeat(254)), None);
        assert!(server_address(&"a".repeat(253)).is_some());
    }
}
