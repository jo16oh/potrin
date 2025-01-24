#!/bin/bash
rm -f db/dev.db &&
sqlx db create &&
sqlx migrate run --source db/migrations/
