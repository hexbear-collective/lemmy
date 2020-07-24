#!/bin/bash

set -e

docker exec -t postgres-dev /bin/bash -c 'pg_dump lemmy -U lemmy' | gzip >/opt/backup/db/lemmy-dev/lemmy-dev-$(date +%Y-%m-%d-%H-%m).gz