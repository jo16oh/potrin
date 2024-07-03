#!/usr/bin/env bash
# This script nukes databases in development environment. 
# Since ElectricSQL only accepts additive schema change, Resetting the entire databases is necessary during early development.

# check whether macOS or others
if [[ "$(uname)" != "Darwin" ]]; then
  echo 'This script only works with macOS'
  exit 1
fi

docker compose -f docker/compose.dev.yml down 
docker volume rm potrin_dev_pg_data 
docker compose -f docker/compose.dev.yml up -d 

# wait until the containers are ready
while true; do 
  log=$(docker compose -f docker/compose.dev.yml logs --tail=1)
  search_string='LOG:  logical replication apply worker for subscription "postgres_1" has started'
  if echo "$log" | grep -q "$search_string"; then
    break
  fi
  sleep 1
done

pnpm pg-migrations apply --directory db/migrations --database postgresql://postgres:proxy_password@localhost:65432/postgres

pnpm exec electric-sql generate

# cleanup app data
rm -rf ~/Library/"Application Support"/com.potrin.dev/*
