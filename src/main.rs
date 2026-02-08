use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tokio::time;

const IPV4_CHECK_URL: &str = "https://checkip.amazonaws.com";
const IPV6_CHECK_URL: &str = "https://v6.ident.me";
const CLOUDFLARE_API_BASE: &str = "https://api.cloudflare.com/client/v4";

#[derive(Debug, Deserialize)]
struct DnsRecordsResponse {
    result: Vec<DnsRecord>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DnsRecord {
    id: String,
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    proxied: bool,
}

struct Config {
    api_token: String,
    zone_id: String,
    host: String,
    interval: Option<Duration>,
    update_ipv4: bool,
    update_ipv6: bool,
}

impl Config {
    fn from_env_and_args() -> Result<Self, String> {
        let mut interval = None;
        let mut update_ipv4 = true;
        let mut update_ipv6 = false;

        // Parse command line arguments manually for minimal dependencies
        let args: Vec<String> = env::args().collect();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "-duration" => {
                    if i + 1 < args.len() {
                        interval = Some(parse_duration(&args[i + 1])?);
                        i += 2;
                    } else {
                        return Err("Missing value for -duration".to_string());
                    }
                }
                "-ipv4" => {
                    if i + 1 < args.len() {
                        update_ipv4 = args[i + 1] == "true";
                        i += 2;
                    } else {
                        return Err("Missing value for -ipv4".to_string());
                    }
                }
                "-ipv6" => {
                    if i + 1 < args.len() {
                        update_ipv6 = args[i + 1] == "true";
                        i += 2;
                    } else {
                        return Err("Missing value for -ipv6".to_string());
                    }
                }
                _ => i += 1,
            }
        }

        let api_token = env::var("CLOUDFLARE_APITOKEN")
            .map_err(|_| "CLOUDFLARE_APITOKEN environment variable is required".to_string())?;
        let zone_id = env::var("CLOUDFLARE_ZONEID")
            .map_err(|_| "CLOUDFLARE_ZONEID environment variable is required".to_string())?;
        let host = env::var("CLOUDFLARE_HOST")
            .map_err(|_| "CLOUDFLARE_HOST environment variable is required".to_string())?;

        Ok(Config {
            api_token,
            zone_id,
            host,
            interval,
            update_ipv4,
            update_ipv6,
        })
    }
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    let mut num_str = String::new();
    let mut unit = String::new();
    
    for c in s.chars() {
        if c.is_ascii_digit() {
            if !unit.is_empty() {
                return Err(format!("Invalid duration format: {}", s));
            }
            num_str.push(c);
        } else {
            unit.push(c);
        }
    }
    
    let num: u64 = num_str.parse()
        .map_err(|_| format!("Invalid duration number: {}", num_str))?;
    
    match unit.as_str() {
        "s" => Ok(Duration::from_secs(num)),
        "m" => Ok(Duration::from_secs(num * 60)),
        "h" => Ok(Duration::from_secs(num * 3600)),
        _ => Err(format!("Invalid duration unit: {}. Use s, m, or h", unit)),
    }
}

async fn get_public_ip(url: &str) -> Result<String, String> {
    // Get current time
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let secs = now.as_secs();
    let timestamp = format_timestamp(secs);
    
    print!("{} Get {} ", timestamp, url);
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let mut retries = 0;
    loop {
        print!(".");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        match client.get(url).send().await {
            Ok(response) => {
                println!();
                let ip = response.text().await
                    .map_err(|e| format!("Failed to read response: {}", e))?
                    .trim()
                    .to_string();
                return Ok(ip);
            }
            Err(e) if (e.is_timeout() || e.is_connect()) && retries < 3 => {
                retries += 1;
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                println!();
                return Err(format!("Failed to get IP from {}: {}", url, e));
            }
        }
    }
}

fn format_timestamp(secs: u64) -> String {
    // Simple timestamp formatter to avoid chrono dependency
    const SECONDS_PER_DAY: u64 = 86400;
    const SECONDS_PER_HOUR: u64 = 3600;
    const SECONDS_PER_MINUTE: u64 = 60;
    
    let days_since_epoch = secs / SECONDS_PER_DAY;
    let remaining = secs % SECONDS_PER_DAY;
    let hours = remaining / SECONDS_PER_HOUR;
    let remaining = remaining % SECONDS_PER_HOUR;
    let minutes = remaining / SECONDS_PER_MINUTE;
    let seconds = remaining % SECONDS_PER_MINUTE;
    
    // Simplified date calculation (approximate)
    let year = 1970 + (days_since_epoch / 365);
    let day_of_year = days_since_epoch % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;
    
    format!("{:04}/{:02}/{:02} {:02}:{:02}:{:02}", year, month, day, hours, minutes, seconds)
}

async fn get_dns_records(
    client: &reqwest::Client,
    api_token: &str,
    zone_id: &str,
    host: &str,
    record_type: &str,
) -> Result<Vec<DnsRecord>, String> {
    let url = format!(
        "{}/zones/{}/dns_records?type={}&name={}",
        CLOUDFLARE_API_BASE, zone_id, record_type, host
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .send()
        .await
        .map_err(|e| format!("Failed to get DNS records: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", status, body));
    }

    let records: DnsRecordsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse DNS records: {}", e))?;

    Ok(records.result)
}

async fn update_dns_record(
    client: &reqwest::Client,
    api_token: &str,
    zone_id: &str,
    record: &DnsRecord,
    new_ip: &str,
) -> Result<(), String> {
    let url = format!(
        "{}/zones/{}/dns_records/{}",
        CLOUDFLARE_API_BASE, zone_id, record.id
    );

    let mut updated_record = record.clone();
    updated_record.content = new_ip.to_string();

    let response = client
        .put(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .json(&updated_record)
        .send()
        .await
        .map_err(|e| format!("Failed to update DNS record: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Failed to update record, status {}: {}", status, body));
    }

    Ok(())
}

async fn update_record(
    client: &reqwest::Client,
    config: &Config,
    ip: &str,
    record_type: &str,
    last_ip: &mut String,
) -> Result<(), String> {
    println!("IP is {} changed={}", ip, ip != *last_ip);

    if ip == *last_ip {
        println!("No change in IP, not updating record");
        return Ok(());
    }

    let records = get_dns_records(client, &config.api_token, &config.zone_id, &config.host, record_type).await?;

    let record = records
        .iter()
        .find(|r| r.name == config.host)
        .ok_or("Host not found in DNS records")?;

    update_dns_record(client, &config.api_token, &config.zone_id, record, ip).await?;
    println!("IP changed, updated to {}", ip);
    *last_ip = ip.to_string();

    Ok(())
}

async fn run_ddns_update(
    client: &reqwest::Client,
    config: &Config,
    last_ipv4: &mut String,
    last_ipv6: &mut String,
) -> Result<(), String> {
    if config.update_ipv4 {
        let ip = get_public_ip(IPV4_CHECK_URL).await?;
        update_record(client, config, &ip, "A", last_ipv4).await?;
    }

    if config.update_ipv6 {
        let ip = get_public_ip(IPV6_CHECK_URL).await?;
        update_record(client, config, &ip, "AAAA", last_ipv6).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let config = match Config::from_env_and_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client");

    let mut last_ipv4 = String::new();
    let mut last_ipv6 = String::new();

    // Run once immediately
    if let Err(e) = run_ddns_update(&client, &config, &mut last_ipv4, &mut last_ipv6).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // If no interval specified, exit
    if config.interval.is_none() {
        return;
    }

    // Run on interval
    let mut interval_timer = time::interval(config.interval.unwrap());
    interval_timer.tick().await; // First tick completes immediately

    loop {
        interval_timer.tick().await;
        if let Err(e) = run_ddns_update(&client, &config, &mut last_ipv4, &mut last_ipv6).await {
            eprintln!("Error: {}", e);
            // Continue running on error
        }
    }
}
