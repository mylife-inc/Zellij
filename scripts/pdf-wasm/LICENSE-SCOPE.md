# Zellij PDF adapter license

Copyright (c) 2026 Shebka.

SPDX-License-Identifier: GPL-2.0-only

Shebka licenses its original work in **this `scripts/pdf-wasm/` directory only**
under the GNU General Public License, version 2 only. This includes the isolated
JavaScript PDF adapter, its original maintainer tools and tests, and the original
documentation and metadata in this directory. The complete, unmodified license
text is in the source tree's `LICENSE`, packaged as
`notices/ZELLIJ-PDF-ADAPTER-LICENSE.txt`.

Coverage follows copies of these files when packaged under `runner/` or included
in corresponding-source archives. User-authored manuscripts are outside this
source-code grant.

This work is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See LICENSE for the terms and conditions.

Existing third-party material retains its original copyright notices and terms.
In particular, the source tree's `NOTICE` (packaged as
`notices/ZELLIJ-ADAPTER-NOTICE.txt`) preserves the MIT attribution for the WasmTex
controller code used by the adapter. Downloaded compiler, TeX package and font
components keep their respective licenses; this scope statement does not
relicense them or complete their distribution review.

This grant does not cover files outside this directory. The Rust CLI, website
engine, templates and the rest of Zellij retain their existing terms. The root
`package.json` and `cli/Cargo.toml` remain `UNLICENSED`.

The owner approved this specific scope on 24 November 2026. Matching adapter and
compiler source must accompany the corresponding optional runtime releases under
the applicable licenses, including if the rest of Zellij later becomes private
or sealed. Existing recipients retain their granted permissions.
