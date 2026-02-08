use std::env;
use std::thread;
use std::time::Duration;

const IPV4_CHECK_URL: &str = "https://checkip.amazonaws.com";
const IPV6_CHECK_URL: &str = "https://v6.ident.me";
const CLOUDFLARE_API_BASE: &str = "https://api.cloudflare.com/client/v4";

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

fn http_get(url: &str) -> Result<String, String> {
    let mut response = ureq::get(url)
        .call()
        .map_err(|e| format!("HTTP GET failed: {}", e))?;
    
    response.body_mut()
        .read_to_string()
        .map_err(|e| format!("Failed to read response: {}", e))
}

fn get_public_ip(url: &str) -> Result<String, String> {
    println!("Getting IP from {}", url);
    http_get(url).map(|s| s.trim().to_string())
}

// Minimal JSON parsing - just extract what we need from Cloudflare API
fn parse_json_array(json: &str, key: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut in_object = false;
    let mut current_obj = String::new();
    let mut brace_count = 0;
    
    // Find the key's array
    if let Some(start) = json.find(&format!("\"{}\"", key)) {
        let rest = &json[start..];
        if let Some(array_start) = rest.find('[') {
            let chars: Vec<char> = rest[array_start..].chars().collect();
            let mut i = 0;
            
            while i < chars.len() {
                match chars[i] {
                    '[' => {}, // Start of array
                    '{' if !in_object => {
                        in_object = true;
                        brace_count = 1;
                        current_obj.clear();
                        current_obj.push('{');
                    },
                    '}' if in_object => {
                        brace_count -= 1;
                        current_obj.push('}');
                        if brace_count == 0 {
                            results.push(current_obj.clone());
                            in_object = false;
                        }
                    },
                    '{' if in_object => {
                        brace_count += 1;
                        current_obj.push('{');
                    },
                    ']' => break,
                    c if in_object => current_obj.push(c),
                    _ => {},
                }
                i += 1;
            }
        }
    }
    
    results
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\"", key);
    if let Some(start) = json.find(&search) {
        let rest = &json[start + search.len()..];
        if let Some(colon) = rest.find(':') {
            let after_colon = &rest[colon + 1..].trim_start();
            if after_colon.starts_with('"') {
                if let Some(end) = after_colon[1..].find('"') {
                    return Some(after_colon[1..=end].to_string());
                }
            }
        }
    }
    None
}

fn extract_json_bool(json: &str, key: &str) -> Option<bool> {
    let search = format!("\"{}\"", key);
    if let Some(start) = json.find(&search) {
        let rest = &json[start + search.len()..];
        if let Some(colon) = rest.find(':') {
            let after_colon = &rest[colon + 1..].trim_start();
            if after_colon.starts_with("true") {
                return Some(true);
            } else if after_colon.starts_with("false") {
                return Some(false);
            }
        }
    }
    None
}

fn get_dns_records(
    api_token: &str,
    zone_id: &str,
    host: &str,
    record_type: &str,
) -> Result<Vec<(String, String, String, bool)>, String> {
    let url = format!(
        "{}/zones/{}/dns_records?type={}&name={}",
        CLOUDFLARE_API_BASE, zone_id, record_type, host
    );

    let mut response = ureq::get(&url)
        .header("Authorization", &format!("Bearer {}", api_token))
        .call()
        .map_err(|e| format!("Failed to get DNS records: {}", e))?;
    
    let body = response.body_mut()
        .read_to_string()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let mut records = Vec::new();
    for obj in parse_json_array(&body, "result") {
        if let (Some(id), Some(name), Some(content), Some(record_type)) = (
            extract_json_string(&obj, "id"),
            extract_json_string(&obj, "name"),
            extract_json_string(&obj, "content"),
            extract_json_string(&obj, "type"),
        ) {
            let proxied = extract_json_bool(&obj, "proxied").unwrap_or(false);
            if name == host {
                records.push((id, record_type, content, proxied));
            }
        }
    }

    Ok(records)
}

fn update_dns_record(
    api_token: &str,
    zone_id: &str,
    record_id: &str,
    record_type: &str,
    name: &str,
    content: &str,
    proxied: bool,
) -> Result<(), String> {
    let url = format!(
        "{}/zones/{}/dns_records/{}",
        CLOUDFLARE_API_BASE, zone_id, record_id
    );

    // Build minimal JSON manually
    let json = format!(
        r#"{{"type":"{}","name":"{}","content":"{}","proxied":{}}}"#,
        record_type, name, content, proxied
    );

    ureq::put(&url)
        .header("Authorization", &format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .send(json.as_bytes())
        .map_err(|e| format!("Failed to update DNS record: {}", e))?;

    Ok(())
}

fn update_record(
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

    let records = get_dns_records(&config.api_token, &config.zone_id, &config.host, record_type)?;

    if records.is_empty() {
        return Err("Host not found in DNS records".to_string());
    }

    let (record_id, rec_type, _, proxied) = &records[0];
    
    update_dns_record(
        &config.api_token,
        &config.zone_id,
        record_id,
        rec_type,
        &config.host,
        ip,
        *proxied,
    )?;
    
    println!("IP changed, updated to {}", ip);
    *last_ip = ip.to_string();

    Ok(())
}

fn run_ddns_update(
    config: &Config,
    last_ipv4: &mut String,
    last_ipv6: &mut String,
) -> Result<(), String> {
    if config.update_ipv4 {
        let ip = get_public_ip(IPV4_CHECK_URL)?;
        update_record(config, &ip, "A", last_ipv4)?;
    }

    if config.update_ipv6 {
        let ip = get_public_ip(IPV6_CHECK_URL)?;
        update_record(config, &ip, "AAAA", last_ipv6)?;
    }

    Ok(())
}

fn main() {
    let config = match Config::from_env_and_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    let mut last_ipv4 = String::new();
    let mut last_ipv6 = String::new();

    // Run once immediately
    if let Err(e) = run_ddns_update(&config, &mut last_ipv4, &mut last_ipv6) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // If no interval specified, exit
    if config.interval.is_none() {
        return;
    }

    // Run on interval
    let interval = config.interval.unwrap();
    loop {
        thread::sleep(interval);
        if let Err(e) = run_ddns_update(&config, &mut last_ipv4, &mut last_ipv6) {
            eprintln!("Error: {}", e);
            // Continue running on error
        }
    }
}
