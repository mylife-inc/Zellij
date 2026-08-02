#
# Builds a documentation site from a content folder. Copy this into your own
# repository — it needs nothing from Zellij's source, only the public base
# image.
#
#   docker build -f site.Dockerfile --build-arg CONTENT=docs -t my-docs .
#   docker run --rm -p 3000:3000 my-docs
#
# To build against a base image you built yourself:
#
#   docker build -f site.Dockerfile \
#     --build-arg ZELLIJ_IMAGE=zellij-base --build-arg ZELLIJ_VERSION=test ...
#
# The engine never reaches the image you deploy: the build stage is discarded
# and only the compiled site survives into the runtime stage.

# The base image, overridable so it can be pulled from a mirror, or pointed at
# a locally built one while working on the image itself.
ARG ZELLIJ_IMAGE=ghcr.io/mylife-inc/zellij-base
ARG ZELLIJ_VERSION=latest
ARG CONTENT=content

# ------------------------------------------------------------------- build
FROM ${ZELLIJ_IMAGE}:${ZELLIJ_VERSION} AS build

ARG CONTENT
WORKDIR /app

# Replace the placeholder content wholesale. `rm` first so a file the previous
# content had and yours does not cannot survive into your site.
RUN rm -rf /app/content
COPY ${CONTENT}/ /app/content/

# Fails the build on a broken link or an invalid section, naming the file —
# rather than publishing a site with a dead page in it.
#
# Through the copy npm installed, not the one at /opt/zellij: that one is only
# ever an install source and has no node_modules beside it, so the CLI cannot
# resolve its own dependencies from there.
RUN npx zellij check content

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
