#!/usr/bin/env bash

docker compose -f docker/compose.test.yml down 
docker compose -f docker/compose.test.yml up -d 
pnpm pg-migrations apply --directory db/migrations --database postgresql://postgres:proxy_password@localhost:65433/postgres 
pnpm exec electric-sql generate --proxy postgresql://postgres:proxy_password@localhost:65433/postgres -s http://localhost:5134
