#!/usr/bin/env bash
curl -d '{"start":{"latitude": 53.5511,"longitude": 9.9937,"rank":0}, "end":{"latitude": 48.7758,"longitude": 9.1829,"rank":0}}' -H "Content-Type: application/json" -X POST http://localhost:8080/dijkstra
echo ""
