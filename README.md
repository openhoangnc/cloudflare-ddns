# Dynamic DNS (Rust)

> Ultra-lightweight Cloudflare DDNS updater written in Rust - **1.5 MB binary, 1 dependency**

Updates a given DNS record with your current IP address. This minimal Rust implementation provides maximum performance with the smallest possible footprint.

## Quick Start

### Docker Run

Example with Cloudflare:
```bash
docker run \
  -e CLOUDFLARE_APITOKEN=YOUR_API_TOKEN \
  -e CLOUDFLARE_ZONEID=YOUR_ZONE_ID \
  -e CLOUDFLARE_HOST=YOUR_DOMAIN \
  ghcr.io/openhoangnc/cloudflare-ddns:latest
```

Example running as a persistent daemon:
```bash
docker run -d --restart always \
  -e CLOUDFLARE_APITOKEN=YOUR_API_TOKEN \
  -e CLOUDFLARE_ZONEID=YOUR_ZONE_ID \
  -e CLOUDFLARE_HOST=YOUR_DOMAIN \
  ghcr.io/openhoangnc/cloudflare-ddns:latest -duration 2h
```

### Docker Compose

Set environment variables in your shell:
```bash
export CLOUDFLARE_APITOKEN=your_api_token_here
export CLOUDFLARE_ZONEID=your_zone_id_here
export CLOUDFLARE_HOST=subdomain.example.com
```

Start the service:
```bash
docker-compose up -d
```

Example `docker-compose.yml`:
```yaml
version: '3.8'

services:
  cloudflare-ddns:
    image: ghcr.io/openhoangnc/cloudflare-ddns:latest
    container_name: cloudflare-ddns
    restart: unless-stopped
    environment:
      # Pass from shell environment or set directly
      - CLOUDFLARE_APITOKEN
      - CLOUDFLARE_ZONEID
      - CLOUDFLARE_HOST
    command: ["-duration", "2h"]
```

Alternatively, set values directly in docker-compose.yml:
```yaml
    environment:
      - CLOUDFLARE_APITOKEN=your_token_here
      - CLOUDFLARE_ZONEID=your_zone_here
      - CLOUDFLARE_HOST=subdomain.example.com
```

## Public IP Detection

The current public IP is read from Cloudflare's own `/cdn-cgi/trace` endpoint,
addressed by resolver IP literal so no DNS lookup is needed and each check is
pinned to the address family it reports on:

| Record | Endpoint |
| ------ | -------- |
| `A` (IPv4)    | `https://1.1.1.1/cdn-cgi/trace` |
| `AAAA` (IPv6) | `https://[2606:4700:4700::1111]/cdn-cgi/trace` |

No third-party IP-echo service is contacted - the only hosts this tool talks to
are Cloudflare's.

## Performance

This minimal Rust implementation provides:
- **Tiny binary**: 1.5 MB (75% smaller than Go, 67% smaller than async Rust)
- **Minimal memory**: ~2-3 MB RAM usage (83% less than Go)
- **Single dependency**: Only ureq for HTTP (no async runtime overhead)
- **Fast startup**: <5ms initialization
- **Synchronous**: Simple blocking I/O, perfect for infrequent DDNS updates
- **Manual JSON**: No serde overhead, custom parsing for Cloudflare API
- **Size-optimized**: Built with `opt-level = "z"` for smallest binary

## Building

### With Cargo
```sh
cargo build --release
```

### With Docker
```sh
docker build -t cloudflare-ddns .
```

## CLI

| Parameter             | Description                                                                                                                                                                | Example           | Required |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-------------------|----------|
| `-duration`           | Runs program perpetually and recheck after specified interval; parses time strings such as `5m`, `15m`, `2h`. If not specified, run once and exit. | 2h                | `false`  |
| `-ipv4`             | Enable updates for IPv4 records. Default is `true`        | `false` | `false`  |
| `-ipv6`             | Enable updates for IPv6 records. Default is `false`        | `true` | `false`  |


## Environment Variables

| Environment Variable               | Description                                                                                                                                                | Example                 | Required |
| ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- | -------- |
| `CLOUDFLARE_APITOKEN`              | An [API Token](https://support.cloudflare.com/hc/en-us/articles/200167836-Managing-API-Tokens-and-Keys), with permission to edit DNS records for your zone | `12345`                 | `true`   |
| `CLOUDFLARE_ZONEID`                | The Zone ID of your domain in Cloudflare (you can find this in the "Overview" tab at the bottom of the page)                                               | `dd255baaaaad2e8...`    | `true`   |
| `CLOUDFLARE_HOST`                  | The record you want to update                                                                                                                              | `subdomain.example.com` | `true`   |

# License

MIT, see [LICENSE](./LICENSE).
