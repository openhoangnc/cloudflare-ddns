# Dynamic DNS (Rust)

> High-performance Cloudflare DDNS updater written in Rust

Updates a given DNS record with your current IP address. This Rust implementation provides maximum performance and minimal memory consumption.

## Quick Start

Example with Cloudflare:
```
docker run \
  -e CLOUDFLARE_APITOKEN=YOUR_API_TOKEN \
  -e CLOUDFLARE_ZONEID=YOUR_ZONE_ID \
  -e CLOUDFLARE_HOST=YOUR_DOMAIN \
  openhoangnc/cloudflare-ddns:3.0.0
```

Example running as a persistent daemon:
```
docker run -d --restart always \
  -e CLOUDFLARE_APITOKEN=YOUR_API_TOKEN \
  -e CLOUDFLARE_ZONEID=YOUR_ZONE_ID \
  -e CLOUDFLARE_HOST=YOUR_DOMAIN \
  openhoangnc/cloudflare-ddns:3.0.0 -duration 2h
```

You can load environment variables through a config file of key/value pairs:

```sh
echo "CLOUDFLARE_APITOKEN=YOUR_API_TOKEN" > config.env
docker run \
  -v $PWD/config.env:/tmp/config.env \
  openhoangnc/cloudflare-ddns:3.0.0 -config /tmp/config.env
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
| `-config`             | Loads environment variables from a given file. Variables should be specified as lines of `key=value` pairs. No variables will be loaded if a file is not specified.        | `/tmp/config.env` | `false`  |
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
