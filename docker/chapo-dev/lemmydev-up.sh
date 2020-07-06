#!/bin/sh

set -e

export LEMMYDEV_POSTGRES_PW="$(cat secrets/postgres_pw)"
export LEMMYDEV_JWT_SECRET="$(cat secrets/jwt_secret)"
export LEMMYDEV_SMTP_PASSWORD="$(cat secrets/smtp_password)"

./docker-compose.sh up -d