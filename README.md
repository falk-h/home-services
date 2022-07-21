# Home services

## Setup

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
