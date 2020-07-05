#!/bin/sh

set -e

export LEMMYDEV_POSTGRES_PW="$(cat secrets/postgres_pw)"
export LEMMYDEV_JWT_SECRET="$(cat secrets/jwt_secret)"

./docker-compose.sh up -d