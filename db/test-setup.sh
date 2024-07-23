#!/usr/bin/env bash

docker compose -f docker/compose.test.yml down --volume
docker compose -f docker/compose.test.yml up -d 

# wait until the containers are ready
while true; do 
  log=$(docker compose -f docker/compose.test.yml logs --tail=1)
  search_string='LOG:  logical replication apply worker for subscription "postgres_1" has started'
  if echo "$log" | grep -q "$search_string"; then
    break
  fi
  sleep 1
done

pnpm pg-migrations apply --directory db/migrations --database postgresql://postgres:proxy_password@localhost:65433/postgres 
pnpm exec electric-sql generate --proxy postgresql://postgres:proxy_password@localhost:65433/postgres -s http://localhost:5134
