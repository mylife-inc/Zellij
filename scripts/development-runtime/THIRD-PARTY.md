# Third-party development dependencies

This optional package contains unmodified third-party dependency files. Their
licenses, copyrights and notices remain beside the files in `node_modules/`.
`DEPENDENCIES.json` records the installed names, versions and declared licenses;
`package-lock.json` records the upstream npm archives and their integrity values.
The package's `UNLICENSED` metadata applies to Shebka's packaging/runner code,
not to these third-party components. Their own licenses continue to apply.

Source for the MPL-2.0 components is available without charge:

- Lightning CSS 1.32.0, including its platform bindings:
  https://github.com/parcel-bundler/lightningcss/tree/v1.32.0
- next-mdx-remote 6.0.0:
  https://github.com/hashicorp/next-mdx-remote/tree/v6.0.0
- Next 16.4.6's bundled `@vercel/og` code and build recipe:
  https://github.com/vercel/next.js/tree/v16.4.6/packages/next

DOMPurify's dual license is used under its Apache-2.0 option. The package retains
Next's bundled component notices and the font/data licenses supplied upstream.
The optional Sharp/libvips image optimizer is excluded: this documentation
development wrapper serves original image files with `images.unoptimized`.
Generated Mermaid/Lucide source maps and the Cytoscape fCoSE demo directory are
omitted to reduce download size. Runtime JavaScript and library licenses remain.

Maintainers: regenerate the exact dependency manifest with `node
scripts/development-runtime/prepare.mjs`, refresh and review the lockfile when
dependencies change, and update these source locations before publishing a new
component version. This package does not contain the separate GPL PDF adapter.
