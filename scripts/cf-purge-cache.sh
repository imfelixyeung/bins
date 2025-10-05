#!/usr/bin/env bash

# sets the api token from password-store to variable
API_TOKEN=$(pass show projects/bins/cloudflare-token)
ZONE_ID=$(pass show projects/bins/cloudflare-zone-id)

curl "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/purge_cache" \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer '"$API_TOKEN"'' \
  --data '{"hosts":["bins.felixyeung.com"]}'

echo ""
echo "Cache purged"
