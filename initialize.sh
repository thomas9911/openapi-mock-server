#!/usr/bin/env bash
PORT=${PORT:-3000}
curl -s -X POST "http://localhost:${PORT}/_initialize" \
  -H "authorization: $API_KEY" \
  -d @openapi.json | jq .
