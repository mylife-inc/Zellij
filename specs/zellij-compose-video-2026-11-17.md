# Composer video and container responsiveness — 2026-11-17

The user asked whether the long-requested universal section was complete and
could express every possible section. `compose` is a released bounded first
version, not the whole proposed system. Its existing guide describes the tree,
four presets, limits, authoring commands and gradient reveal. The guide now also
has an explicit capability/status matrix. Solid-to-gradient headings already
work in rc.10. Layers, sticky scroll stages, independent media selectors,
feature explorers, block Markdown, icons and embedding arbitrary section types
remain unfinished. Arbitrary scripts/CSS are outside the declarative contract.

## Source rc.11 increment

- Add `type: video` as a composer leaf at every supported group depth. Require
  a label and source; support a poster, inline caption, transcript link and up
  to eight WebVTT caption/subtitle tracks. Existing tree limits are unchanged.
- Share media rendering with `video-embed`; its visible caption stays plain
  text for compatibility. Both gain tracks and transcripts. Asset annotations
  cover sources, posters and tracks; href annotations cover transcripts. The
  generated native schema/context and authoring tests exercise the same fields.
- Video remains user initiated. Fix the existing inert no-JavaScript play
  button with a native self-hosted player using `preload="none"`. Provider
  videos keep a direct source link and load no iframe until requested. After
  activation, focus moves from the disappearing button to the player.
- Print hides player controls and retains the source/transcript links and
  visible caption. Caption text is not a substitute for synchronized speech
  captions. Provider subtitles are managed by the provider; local tracks apply
  to self-hosted media. Cross-origin media/tracks need anonymous CORS support.
- Provider detection now parses an exact hostname and path rather than matching
  `youtube.com`/`vimeo.com` anywhere in a URL. Query parameter order and YouTube
  short/watch/embed URLs are supported. Embeds use the no-cookie YouTube host.
- Nested grids and splits use their own 760px containing width instead of the
  browser width. Groups supply a container around their inner layout. Narrow
  cards stay readable on desktop; source order is unchanged. Unsupported
  container queries and print retain a single column.

No dependency is added. Composer format remains version 1; existing documents
are valid. These additive nodes/fields require a matching rc.11 CLI/engine.
Published rc.10 artifacts, existing template archives and production hosting
are untouched. Docker's default source version follows Cargo's rc.11 version.

## Original fixture

`compose-motion.mp4` is an original 4.5 KB, two-second silent H.264 clip generated
for this task with FFmpeg: 640×360 at 12 fps, warm paper with three rectangles
(copper/slate/gold), fading in over 0.8 seconds. It contains no third-party
footage. Both showcase and documentation include the same bytes and a local
English WebVTT track. The showcase supplies a readable animation transcript.

## Verification

Local: engine types/build and 504 unit tests pass. The first run caught a test
assertion using lowercase `srclang` against React's server markup `srcLang`;
the assertion now uses React's emitted attribute spelling. Browser HTML remains
case-insensitive. Native tests, generated-schema parity, deep checking and
hosted browser/visual acceptance are still in progress.

Hosted coverage adds real local-video decoding/playback, lazy loading, caption
loading, focus after activation, no-JavaScript native controls, print links,
reduced-motion behavior and nested container sizing on desktop, Android-sized
Chromium and iPhone-sized WebKit. Existing composer screenshots cover the
additional section in light/dark themes. No local browser or container is run.

The separate development-only picomatch advisory remains open under its dated
exception. No zero-bug, unlimited-composition or production-publication claim
is made. The next composer slice should define layered layouts/placement before
adding further scroll behavior; it must preserve a meaningful mobile/print
reading order and avoid covering interactive content.
