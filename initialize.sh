#!/usr/bin/env bash
curl -s -X POST http://localhost:3000/_initialize \
  -H 'Content-Type: application/json' \
  -d @openapi.json | jq .
