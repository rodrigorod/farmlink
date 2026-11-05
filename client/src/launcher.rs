//! Farming Simulator 25 execution and custom URL scheme handling.

use std::process::Command;
use tracing::info;

use crate::error::{ClientError, Result};

/// Steam AppID for Farming Simulator 25.
pub const FS25_STEAM_APP_ID: u32 = 2300320;

/// Parsed parameters from a `farmlink://join` protocol URI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinUrlParams {
    pub server_id: Option<String>,
    pub address: Option<String>,
}

/// Parses a `farmlink://join?server=...&addr=...` URI.
pub fn parse_join_url(url: &str) -> Option<JoinUrlParams> {
    if !url.starts_with("farmlink://") {
        return None;
    }

    let without_scheme = &url["farmlink://".len()..];
    let query_str = without_scheme
        .split_once('?')
        .map(|(_, q)| q)
        .unwrap_or(without_scheme);

    let mut server_id = None;
    let mut address = None;

    for pair in query_str.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            let decoded_v = percent_decode(v);
            match k {
                "server" => server_id = Some(decoded_v),
                "addr" | "address" => address = Some(decoded_v),
                _ => {}
            }
        }
    }

    Some(JoinUrlParams { server_id, address })
}

/// Simple percent-decoding helper without heavy extra dependencies.
fn percent_decode(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(b) = u8::from_str_radix(&hex, 16) {
                    output.push(b as char);
                    continue;
                }
            }
            output.push('%');
            output.push_str(&hex);
        } else if ch == '+' {
            output.push(' ');
        } else {
            output.push(ch);
        }
    }

    output
}

/// Launches Farming Simulator 25 locally, optionally targeting a specific server IP/port.
pub fn launch_fs25(server_address: Option<&str>) -> Result<()> {
    info!(
        "attempting to launch Farming Simulator 25 (address: {:?})",
        server_address
    );

    #[cfg(target_os = "windows")]
    {
        let steam_target = match server_address {
            Some(addr) => format!("steam://run/{}//+connect {}", FS25_STEAM_APP_ID, addr),
            None => format!("steam://run/{}", FS25_STEAM_APP_ID),
        };

        Command::new("cmd")
            .args(["/C", "start", &steam_target])
            .spawn()
            .map_err(|e| ClientError::GameLaunchFailed(format!("failed spawning process: {e}")))?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        // Try steam first
        let steam_args = match server_address {
            Some(addr) => vec![
                "-applaunch".to_string(),
                FS25_STEAM_APP_ID.to_string(),
                format!("+connect {}", addr),
            ],
            None => vec!["-applaunch".to_string(), FS25_STEAM_APP_ID.to_string()],
        };

        if let Ok(child) = Command::new("steam").args(&steam_args).spawn() {
            let _ = child.id();
            return Ok(());
        }

        // Fallback to xdg-open steam:// URL
        let steam_url = match server_address {
            Some(addr) => format!("steam://run/{}//+connect {}", FS25_STEAM_APP_ID, addr),
            None => format!("steam://run/{}", FS25_STEAM_APP_ID),
        };

        Command::new("xdg-open")
            .arg(&steam_url)
            .spawn()
            .map_err(|e| {
                ClientError::GameLaunchFailed(format!("failed invoking steam/xdg-open: {e}"))
            })?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let steam_url = format!("steam://run/{}", FS25_STEAM_APP_ID);
        Command::new("open")
            .arg(&steam_url)
            .spawn()
            .map_err(|e| ClientError::GameLaunchFailed(format!("failed invoking open: {e}")))?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(ClientError::GameLaunchFailed(
        "unsupported operating system".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_custom_farmlink_uri() {
        let uri =
            "farmlink://join?server=srv_zielonka_01&addr=zielonka.farmlink-servers.eu%3A10823";
        let params = parse_join_url(uri).expect("parsed uri");

        assert_eq!(params.server_id.as_deref(), Some("srv_zielonka_01"));
        assert_eq!(
            params.address.as_deref(),
            Some("zielonka.farmlink-servers.eu:10823")
        );
    }

    #[test]
    fn ignores_invalid_scheme() {
        assert!(parse_join_url("https://google.com").is_none());
        assert!(parse_join_url("steam://run/2300320").is_none());
    }
}
