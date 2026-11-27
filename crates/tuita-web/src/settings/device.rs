//! Naming a session's device.

/// Browsers by a token of the `User-Agent`, most specific first.
const BROWSERS: [(&[&str], &str); 6] = [
    (&["Edg/"], "Edge"),
    (&["OPR/", "Opera"], "Opera"),
    (&["SamsungBrowser"], "Samsung Internet"),
    (&["Firefox/"], "Firefox"),
    (&["Chrome/"], "Chrome"),
    (&["Safari/"], "Safari"),
];

/// Systems by a token of the `User-Agent`, most specific first.
const SYSTEMS: [(&[&str], &str); 7] = [
    (&["Windows"], "Windows"),
    (&["Android"], "Android"),
    (&["iPhone", "iPod"], "iPhone"),
    (&["iPad"], "iPad"),
    (&["CrOS"], "ChromeOS"),
    (&["Mac OS"], "macOS"),
    (&["Linux"], "Linux"),
];

/// Device families the server reports in `SessionDto.client` when the agent names no system.
fn client_system(client: &str) -> Option<&'static str> {
    Some(match client {
        "android-app" | "android" => "Android",
        "iphone" => "iPhone",
        "ipad" => "iPad",
        "windows" => "Windows",
        "mac" => "macOS",
        "linux" => "Linux",
        "chromeos" => "ChromeOS",
        _ => return None,
    })
}

/// The browser and system named by a `User-Agent`; the system falls back to the `client` family.
#[must_use]
pub(crate) fn device_parts(
    user_agent: &str,
    client: Option<&str>,
) -> (Option<&'static str>, Option<&'static str>) {
    let find = |table: &[(&[&str], &'static str)]| {
        table
            .iter()
            .find(|(tokens, _)| tokens.iter().any(|token| user_agent.contains(token)))
            .map(|(_, name)| *name)
    };
    // `curl` matches case-insensitively.
    let browser = find(&BROWSERS).or_else(|| {
        user_agent
            .to_ascii_lowercase()
            .contains("curl")
            .then_some("curl")
    });
    let system = find(&SYSTEMS).or_else(|| client.and_then(client_system));
    (browser, system)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reads_browser_and_system_from_a_user_agent() {
        let ua = "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0";
        assert_eq!(device_parts(ua, None), (Some("Firefox"), Some("Linux")));
    }

    #[test]
    fn falls_back_to_the_client_family_when_the_agent_names_no_system() {
        assert_eq!(
            device_parts("okhttp/4.12", Some("android-app")),
            (None, Some("Android"))
        );
        assert_eq!(device_parts("okhttp/4.12", Some("toaster")), (None, None));
    }

    #[test]
    fn chrome_wins_over_the_safari_token_it_also_carries() {
        let ua =
            "Mozilla/5.0 (Windows NT 10.0) AppleWebKit/537.36 Chrome/130.0 Safari/537.36 Edg/130.0";
        assert_eq!(device_parts(ua, None), (Some("Edge"), Some("Windows")));
        assert_eq!(device_parts("curl/8.5.0", None), (Some("curl"), None));
    }
}
