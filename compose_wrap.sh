#!/bin/bash
set -euo pipefail

# Wrapper for `docker compose` that sets up environment variables that are used
# in `docker-compose.yaml`.

cd "$(dirname "${BASH_SOURCE[0]}")"

if ! [[ -f .env ]]; then
    echo "You need to create a .env file! See env.template" >&2
    exit 1
fi

# Needed for pihole
LOCAL_IP=$(ip -4 -json address \
    | jq -r '.[] | select(.ifname | test("enp.*")) | .addr_info[0].local')
export LOCAL_IP

docker compose "$@"
