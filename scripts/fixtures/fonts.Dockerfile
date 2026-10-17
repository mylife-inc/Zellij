# The image already contains the engine/font snapshot from this checkout.
FROM zellij-ci
RUN npm install --install-links
# A clean default Turbopack build, without a font cache or external network.
# Loopback remains available for Next's compiler workers.
RUN --network=none npm run build
