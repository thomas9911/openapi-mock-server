#!/usr/bin/env bash
PORT=${PORT:-3000}
curl -s -X POST "http://localhost:${PORT}/_initialize" \
  -H 'Content-Type: application/json' \
  -d @openapi.json | jq .
