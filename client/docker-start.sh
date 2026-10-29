#!/usr/bin/dumb-init /bin/sh

# Integrate environment variables
sed -i "s|__BACKEND__|${BACKEND_HOST}|" \
  /etc/nginx/nginx.conf
sed -i "s|__BASEURL__|${BASE_URL:-/}|g" \
  /var/www/index.htm \
  /var/www/manifest.json
sed -i "s|__ASSISTANT_URL__|${ASSISTANT_URL:-}|g" \
  /var/www/index.htm

# Start server
exec nginx
