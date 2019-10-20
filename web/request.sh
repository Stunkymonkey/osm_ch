#!/usr/bin/env bash
curl -d '{"start":{"latitude": 9.9937,"longitude": 53.5511}, "end":{"latitude": 9.1829,"longitude": 48.7758}}' -H "Content-Type: application/json" -X POST http://localhost:8080/dijkstra
echo ""
