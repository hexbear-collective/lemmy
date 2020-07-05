#!/bin/sh

set -e

export LEMMYPROD_POSTGRES_PW="$(cat secrets/postgres_pw)"
export LEMMYPROD_JWT_SECRET="$(cat secrets/jwt_secret)"

./docker-compose.sh up -d