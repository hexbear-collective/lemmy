<div align="center"> 

<p align="center">
    <a href="https://www.hexbear.net" rel="noopener">
    <img width=300px height=300px src="https://www.hexbear.net/static/android-chrome-512x512.png"></a>
</p>

**Main Branch:** [![Build Status](https://drone.chapo.chat/api/badges/hexbear-collective/lemmy-hexbear/status.svg?ref=refs/heads/main)](https://drone.chapo.chat/hexbear-collective/lemmy-hexbear)
**Production:** [![Build Status](https://drone.chapo.chat/api/badges/hexbear-collective/lemmy-hexbear/status.svg?ref=refs/heads/prod)](https://drone.chapo.chat/hexbear-collective/lemmy-hexbear) 

[Original Lemmy README](README-lemmy.md)

</div>

# Hexbear

Hexbear is the engine that powers [chapo.chat](https://www.chapo.chat). It is a customization of the [Lemmy](https://github.com/LemmyNet/lemmy) project.

> **Why Hexbear?**
>
> The geometric beauty of Hexbear; all edges and angles equal. She guides our solidarity.

## Overview

The goal is to contribute to the upstream [Lemmy](https://github.com/LemmyNet/lemmy) project while also trying to retain the r/CTH flavor that brought the community together in the first place.

Please make sure to visit the original readme for all of the information about Lemmy.

## Running in Docker

```
cd docker/dev
sudo ./docker_update.sh
sudo docker-compose down

# connect to the docker postgresql
psql -h localhost -p 5432 -U lemmy # password is password
```
