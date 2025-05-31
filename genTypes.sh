#!/bin/bash

# GENERATE API TYPES
cd sanzang
curl http://127.0.0.1:8080/api-docs/openapi.json | jq '.' > ./src/lib/openapi.json
npx openapi-typescript http://127.0.0.1:8080/api-docs/openapi.json -o ../sanzang/src/lib/wukong.d.ts

# GENERATE WEBSOCKET TYPES
cd ../wukong
rm -rf bindings
rm -rf ../sanzang/src/lib/bindings
cargo test export_bindings
cp -r bindings ../sanzang/src/lib/bindings