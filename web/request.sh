#!/usr/bin/env bash
curl -d '{"type":"FeatureCollection","features":[{"type":"Feature","geometry":{"type":"Point","coordinates":[6.767578125000001,49.459198634468564]}},{"type":"Feature","geometry":{"type":"Point","coordinates":[7.182312011718751,49.31438004800689]}}]}' -H "Content-Type: application/json" -X POST http://localhost:8080/dijkstra
echo ""
