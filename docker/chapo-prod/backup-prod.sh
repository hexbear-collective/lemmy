#!/bin/bash

set -e

docker exec -t postgres-prod /bin/bash -c 'pg_dump lemmy -U lemmy -p 5433' | gzip >/opt/backup/db/lemmy-prod/lemmy-prod-$(date +%Y-%m-%d-%H-%m).gz
