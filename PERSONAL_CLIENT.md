# Personal Oxibooru client

This branch keeps the official Oxibooru server and database unchanged. Only the
client image is customized.

## Runtime setting

Set this environment variable on the `oxibooru-client` container:

```text
ASSISTANT_URL=http://192.168.1.3:8001
```

When the value is empty, the `Statistics` navigation item is hidden.

## Image publication

Every push to a `personal/**` branch publishes an edge image to GitHub Container
Registry. A stable version is published by pushing a tag such as:

```text
client-v0.8.0-personal.1
```

The resulting stable image is:

```text
ghcr.io/c1391815/oxibooru-client:0.8.0-personal.1
```

For TrueNAS, keep the official server image and replace only the client image.
Add `ASSISTANT_URL` to the client environment variables.
