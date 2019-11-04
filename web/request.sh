#!/usr/bin/env bash
curl -d '{"start":{"latitude": 53.5511,"longitude": 9.9937}, "end":{"latitude": 48.7758,"longitude": 9.1829},"use_car":true,"by_distance":false}' -H "Content-Type: application/json" -X POST http://localhost:8080/dijkstra
echo ""
