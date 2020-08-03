*(Original Lemmy readme is [here](README-lemmy.md))*

# Lemmy (Chapo edition)

## Overview

The goal is to contribute to the upstream [Lemmy](https://github.com/LemmyNet/lemmy) project while also trying to retain the r/CTH flavor that brought the community together in the first place.

Please make sure to visit the original readme for all of the information.

## Running in Docker

```
cd docker/dev
sudo ./docker_update.sh
sudo docker-compose down

# connect to the docker postgresql
psql -h localhost -p 5432 -U lemmy # password is password
```
