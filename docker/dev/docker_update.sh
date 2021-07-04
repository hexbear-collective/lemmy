#!/bin/sh
set -e

docker build ../../ --file ../dev/Dockerfile -t lemmy-dev:latest
docker-compose up -d
