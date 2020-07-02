# Docker Development

## Setup

> **NOTE**: Workflow is designed for Mac/Linux - it will work on Windows with WSL, but you'll need to do some extra steps.

* **Mac** - Install [Docker Desktop for Mac](https://hub.docker.com/editions/community/docker-ce-desktop-mac) - this creates the VM that will build images, as well as installs other tools needed such as docker-compose. Once installed, go to settings and bump up the amount of RAM from 2gb to at least 4gb, or the build will fail. (8GB recommended)

* **Linux** - You'll need to install Docker and docker-compose - there is a different installation for Docker per distro, but here is the [Ubuntu/Debian](https://docs.docker.com/engine/install/ubuntu/) instructions. Once done, go to the [docker-compose install directions](https://docs.docker.com/compose/install/) and pick the Linux instructions.

## Running

```bash
git clone https://github.com/LemmyNet/lemmy
cd lemmy/docker/dev
sudo docker-compose up --no-deps --build
```

Upon running this, it will take a while to build, especially on slower systems. Later builds will be faster, based on how Docker creates images (using point in time snapshots).

Also, trying to upload images will be broken until you set the permissions on your /volumes/pictrs folder - you need to run `chmod -R 991:991 /volumes/pictrs` from the root of the project, and will likely need to restart the docker-compose after that.

Once everything is up and running, you can go to http://localhost:8536 to visit your local instance.

## Additional Steps

To speed up the Docker compile, add the following to `/etc/docker/daemon.json` and restart Docker.
```
{
  "features": {
    "buildkit": true
  }
}
```

*(You can edit the daemon.json file on Docker Desktop under Settings > Docker Engine)*

If the build is still too slow, you will have to use a
[local build](contributing_local_development.md) instead.
