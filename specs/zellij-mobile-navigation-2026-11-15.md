# Mobile navigation with section views

Date: 2026-11-15. Baseline: `259567e` on main; source rc.9, public rc.8.
The user approved the proposed mobile interaction before rc.9 publication.

## Behavior

The hamburger opens the main section names. A group opens a dedicated view
with a heading and its links; existing mega-menu headings, descriptions and
icons stay in that view rather than adding unnecessary levels. Back returns
to the root menu with its previous scroll position and focused section. Close
and Escape exit the entire menu. Direct links navigate immediately and close;
external/new-tab metadata is preserved. Each opening starts at the main menu.

The existing menu model supplies both desktop and mobile navigation. Phones
(up to 600px) use a full-width panel; tablets up to the existing 900px desktop
breakpoint use a 30rem side panel and retain backdrop dismissal. Back/Close
controls remain outside the independently scrolling menu, with safe-area
padding and 44px targets. Root CTA/GitHub links remain in a bounded scrolling
footer. Light/dark themes use existing tokens; no dependency/font is added.
Animations honor reduced motion and `motion: none`.

One portal dialog replaces its active content; no layered dialogs or duplicate
hidden focus targets are added. Focus moves to the section heading on entry,
to its originating button on Back, and follows the existing modal cleanup on
close. The mounted open component owns view state so closing/reopening resets
it, including across responsive breakpoint changes. Header scroll updates do
not restart the trap or reset the active view.

The old mobile leaf/root renderers omitted `target`/`rel` metadata. This change
also restores the already-configured new-tab behavior in both menu levels.

## Verification

Existing drawer and header focus/resize tests stay active. New hosted browser
cases cover root/section/Back/Close, direct and nested navigation, real external
popup behavior through an intercepted destination, root scroll restoration in
a short viewport, document scroll on Escape, resize reset, light/dark layouts
at 320/390/768px, reduced motion, motion none, section keyboard trapping and
tablet backdrop dismissal. Chromium and WebKit run the new suite in phone and
desktop keyboard contexts. Screenshots capture root and section views in both
palettes. Actual CLI website exports also open/close their direct-link mobile
menus and retain screenshots.

Native portable books retain their independent sidebar navigation. Public rc.8,
production hosting and deferred live-provider checks are unchanged. Final source
CI, visual review, merge and development-image evidence will follow verification.
