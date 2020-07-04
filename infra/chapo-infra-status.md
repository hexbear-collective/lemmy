# Infrastructure Status (Chapo Deploy)
## Overview
hello its me alltheseteeth pls read 4 info

I'll split it into multiple sections - server, source/CI/CD, network

If I am being vague on some points it is intentional for security! ok ty

Most things that are default admin accounts I have made sure to note the credentials and put it in the keyvault

## Server
The connection info for the server is in the keyvault - I made my own user on the server just for ease of use, but nothing should be tied to that user, either run things as sudo or add a user to the docker usergroup.

Software-wise it's pretty simple: Debian, docker, docker-compose, and an nginx config for reverse proxy

All running code (outside of the nginx reverse proxy config file) lives in the `/opt/app/` folder - I tried to make it so it was one folder per docker-compose/docker-swarm file (so one folder = one stack). The intention of this is of course being you can back up the individual app folder and be able to restore it quite easily.

All the volumes mentioned in the compose files should be underneath the folder. There is a `volumes` folder under most of them.

* **lemmy-dev** - docker-compose in `/docker/chapo-dev`

  The main app stack

* **tools** - docker-compose in `/docker/chapo-tools`

  The intention is to put non-stack-related operational tools here - currently it runs a docker control UI, and watching for new images to automatically restart the `lemmy-dev` main docker

* **monitor** - docker-swarm in `/docker/chapo-monitor`

  This is an all-in-one set of images to run in docker swarm to get metrics into Grafana

There is also another folder under `/opt/` for additional certs needed for Cloudflare - these are referenced in the nginx config.

## Source Control + CI/CD

Source is in GitLab (obvi if ur reading this) which is also how CI/CD is done - we're simply building the main Lemmy dockerfile marked as prod, but with our changes in the `chapo-dev` branch.

When an image is marked appropriately via the gitlab-ci, the dev stack picks it up and restarts itself with the new image.

## Network

DDoS protection and caching is done through Cloudflare. 

## Future Intention

* Prod environment/cloudflare with manual vetted promotions
* Multiple servers
* Dedicated monitoring/logging server
* Hosted secret vault
* Splitting backend and frontend into two images
* question mark