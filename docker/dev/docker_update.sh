#!/bin/sh
set -e

docker build ../../ --file ../dev/Dockerfile -t hexbear-dev:latest
docker-compose up -d
