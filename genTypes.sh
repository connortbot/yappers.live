cd sanzang
curl http://localhost:8080/api-docs/openapi.json | jq '.' > ./src/lib/openapi.json
npx openapi-typescript http://localhost:8080/api-docs/openapi.json -o ./src/lib/wukong.d.ts