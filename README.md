# Home services

These are the services running on my home server.

## Setup

### Requirements

- Reasonably recent Docker
- `jq`

### Secrets

Fill in [`env.template`](./env.template) and copy it to `.env`.

Fill in GoDNS's [`config.yaml.template`](./godns/config/config.yaml.template)
and copy it to `godns/config/config.yaml`.

### GoDNS

GoDNS can't create new subdomains on Cloudflare. Make sure to create all
subdomains that are configured in `godns/config/config.yaml` in the [Cloudflare
dashboard](https://dash.cloudflare.com).

### Pi-hole

*See also [the official docs for
Ubuntu](https://github.com/pi-hole/docker-pi-hole/#installing-on-ubuntu).*

Disable `systemd-resolved`'s local stub DNS resolver since it binds to port 53.

```shell
sudo sed -E -i 's/#?DNSStubListener=yes/DNSStubListener=no/' /etc/systemd/resolved.conf
```

Change the `/etc/resolv.conf` symlink to not point to the stub resolver.

```
sudo rm /etc/resolv.conf
sudo ln -s /run/systemd/resolve/resolv.conf /etc/resolv.conf
```

Restart `systemd-resolved`.

```
sudo systemctl restart systemd-resolved
```

Set the server as the primary DNS server in the router's settings.
