cd sanzang
curl http://127.0.0.1:8080/api-docs/openapi.json | jq '.' > ./src/lib/openapi.json
npx openapi-typescript http://127.0.0.1:8080/api-docs/openapi.json -o ./src/lib/wukong.d.ts