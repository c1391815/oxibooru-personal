# Personal server changes

## server-v0.9.1-personal.1

- Add a PostgreSQL-backed display parent with automatic, explicit and root modes.
- Read it with `GET /tag/{name}?fields=parentTag` and set it with `PUT /tag/{name}` by including `parentTag` and the tag version. It uses the existing tag implication edit privilege and preserves the implication list.
- Automatic mode uses the sole implication when both tags share a category. Explicit parents must be in the same category when selected. Category changes can hide a link, and deleting the parent clears the reference without activating automatic mode.
- Server startup applies migration 022. The release image is published to GHCR under the release tag.
