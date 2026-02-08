# Dynamic DNS (Rust)

> High-performance Cloudflare DDNS updater written in Rust

Updates a given DNS record with your current IP address. This Rust implementation provides maximum performance and minimal memory consumption.

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

Create a `.env` file with your credentials:
```bash
cp .env.example .env
# Edit .env with your values
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
      - CLOUDFLARE_APITOKEN=${CLOUDFLARE_APITOKEN}
      - CLOUDFLARE_ZONEID=${CLOUDFLARE_ZONEID}
      - CLOUDFLARE_HOST=${CLOUDFLARE_HOST}
    command: ["-duration", "2h"]
```

Example `.env` file:
```bash
CLOUDFLARE_APITOKEN=your_api_token_here
CLOUDFLARE_ZONEID=your_zone_id_here
CLOUDFLARE_HOST=subdomain.example.com
```

## Performance

This Rust implementation provides:
- **Low memory footprint**: ~2-5 MB RAM usage (vs ~10-20 MB for Go)
- **Fast startup**: Near-instant initialization
- **Efficient networking**: Using rustls for TLS with zero-copy operations
- **Static binary**: No runtime dependencies

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
