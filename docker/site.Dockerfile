#
# Builds a documentation site from a content folder. Copy this into your own
# repository — it needs nothing from Zellij's source, only the public base
# image.
#
#   docker build -f site.Dockerfile --build-arg CONTENT=docs -t my-docs .
#   docker run --rm -p 3000:3000 my-docs
#
# The engine never reaches the image you deploy: the build stage is discarded
# and only the compiled site survives into the runtime stage.

ARG ZELLIJ_VERSION=latest
ARG CONTENT=content

# ------------------------------------------------------------------- build
FROM ghcr.io/mylife-inc/zellij-base:${ZELLIJ_VERSION} AS build

ARG CONTENT
WORKDIR /app

# Replace the placeholder content wholesale. `rm` first so a file the previous
# content had and yours does not cannot survive into your site.
RUN rm -rf /app/content
COPY ${CONTENT}/ /app/content/

# Fails the build on a broken link or an invalid section, naming the file —
# rather than publishing a site with a dead page in it.
RUN node /opt/zellij/bin/zellij.mjs check /app/content

RUN npm run build


# ----------------------------------------------------------------- runtime
FROM node:24-alpine AS runtime

ENV NODE_ENV=production \
    NEXT_TELEMETRY_DISABLED=1 \
    PORT=3000 \
    HOSTNAME=0.0.0.0

# Not root. The site serves static files and needs nothing a user cannot do.
RUN addgroup -g 1001 -S nodejs && adduser -S -u 1001 -G nodejs nextjs

WORKDIR /app

# `standalone` carries its own traced dependencies, so there is no npm install
# here and no node_modules to ship.
COPY --from=build --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=build --chown=nextjs:nodejs /app/.next/static ./.next/static
COPY --from=build --chown=nextjs:nodejs /app/public ./public

USER nextjs
EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s \
  CMD node -e "fetch('http://127.0.0.1:'+(process.env.PORT||3000)+'/').then(r=>process.exit(r.ok?0:1)).catch(()=>process.exit(1))"

CMD ["node", "server.js"]
