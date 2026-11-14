# Shared book statements — 2026-11-13

Source baseline: `ed23611` on `main`. Public rc.6 remains unchanged. This
increment extends shared equation identity with mathematical statement blocks.
Implementation and acceptance are in progress; no hosted result is claimed yet.

## Authoring contract

In a portable book, a top-level Markdown blockquote starts with a single plain
text metadata paragraph and a blank quoted line before its body:

```md
> [!THEOREM] {#thm:energy} Energy and equality
>
> The energy is nonnegative.

> [!PROOF] {#proof:energy}
>
> Apply $\autoref{lem:square}$ to prove $\autoref{thm:energy}$.
```

Kinds are uppercase `THEOREM`, `LEMMA`, `PROPOSITION`, `COROLLARY`, `DEFINITION`,
`EXAMPLE`, `REMARK` and `PROOF`. Every block needs a stable `{#id}`, including
proofs; a following plain text title is optional. Bodies support normal rich
Markdown, math, lists, code, ordinary nested quotes and footnote references.
Semantic blocks must be top-level, not inside lists, quotes or footnotes. Bodies
cannot contain headings or footnote definitions. Put definitions outside and
reference them from the body. Empty bodies, malformed recognized markers, rich
metadata headings, unknown labelled kinds and invalid labels fail validation.

All numbered kinds share one counter per reading entry. Chapters use `1.1`,
`1.2`; appendices `A.1`; front matter `F1.1`; an authored title page `T.1`.
Proofs have no number, do not consume a number and end with a proof mark.
Equations keep their independent counter. Labels share the book-wide equation
namespace and ASCII identifier rules. A label's anchor `_zel-stmt-thm:energy`
stays unchanged when earlier statements are inserted or the reading order moves.
Renaming its label or source changes its target.

With `markdown.math: true`, standalone inline `$\ref{thm:energy}$` displays the
number; `$\autoref{thm:energy}$` displays `Theorem 1.1`. `\autoref` also supports
equations (`Equation 1.1`) and proofs (`Proof`). `\eqref` remains equation-only;
`\ref` rejects unnumbered proofs with guidance to use `\autoref`. References
cannot occur in headings, links or larger mathematical expressions. Custom
Markdown links can target stable anchors without enabling math. Plain statement
blocks need no math feature. Non-book Markdown/GFM alerts and manually numbered
MDX `<Theorem>` components retain their existing behavior.

## Model and adapters

`statements.rs` resolves the book-wide registry after equations and before
reference/link validation or output writes. The shared header parser is used by
all three adapters. Publication schema version 1 gains an optional `statements`
array, omitted when empty. Each item exports `id`, lowercase `kind`, optional
`number`/`title`, `entry`, `source`, `anchor` and encoded root-relative `href`.
Internal page indexes stay private. Existing revision identity includes the
original manuscript and manifest, so inserted blocks/reordering change it.

References lower to validated link events. HTML and Next emit inert, escaped
semantic containers with the same heading/number/body/proof mark; Next only adds
an allowed `ariaHidden` string enum (true/false) for the decorative mark. Print emits
escaped titles and stable hyperlink destinations with the resolved numbers;
LaTeX does not run another counter or execute authored metadata. Blocks can flow
across pages; headings reserve following space and proof marks stay with the
preceding content. Existing PDF layouts are unaffected by absent statements.

## Verification and retained findings

Native regressions cover every kind, all reading-entry prefixes, shared counters,
forward/backward/equation/proof references, stable targets after insertion and
CRLF manifest reorder, rich bodies, ordinary nested quotes, raw-text escaping,
math-disabled blocks, non-book compatibility and last-export preservation on
invalid input. Engine tests exercise actual inert-tree rendering. The shared
fixture is `scripts/fixtures/statements`; the original equation fixture stays
unchanged for regression comparison.

Hosted acceptance covers relocated `file://` exports in Chromium/WebKit with
JavaScript on/off on Linux/macOS/Windows, keyboard references, narrow layouts,
prefixed Next images and real LuaLaTeX on Docker amd64/arm64 and rootless Podman.
The Next equation image now carries the combined statement/equation manuscript,
so existing equation browser assertions still run without another image build.
PDF acceptance adds the combined manuscript while retaining every prior PDF.

Initial local checks found an imprecise missing-label error, corrected to request
`{#id}` explicitly. The new no-math fixture initially omitted required book
authors/chapters; the fixture was corrected without weakening manifest validation.
The engine typecheck also required HAST ariaHidden values as strings rather than
booleans; the emitter and strict schema now agree. A rendering assertion was
corrected to allow harmless HTML attribute ordering. Search text includes the
resolved headings/references, without marker or label syntax. Local verification passes 782 Rust tests (five hosted-only tests remain ignored
locally), 490 engine tests, workspace typechecks, warnings-denied Clippy,
formatting, engine packaging, Actionlint and strict/deep documentation checks
(59 routes, 25 examples). The actual Rust-emitted five-page manuscript passes
the engine deep/strict validator. Search changes subsequently pass all 72 native
docs tests. Hosted results will be recorded with their exact source commits.

Remaining advanced publishing includes bibliography, index/glossary,
multilingual/RTL print, Next-image PDFs and richer technical/study/custom
print templates. This increment does not establish zero bugs or a complete 1.0.
The separate recorded WebKit overflow and documentation hydration timings remain
unexplained; live providers and production-domain migration remain deferred.

The first hosted run `43563739586` at `4203d65` passes real PDF on all three
runtimes, prefixed Next, Linux/Windows book browsers and the engine/showcase
checks. macOS WebKit with JavaScript disabled loses the new helper's programmatic
focus immediately after returning to a fragment in another document (job
`132352906017`). The helper had no explicit load/font/target readiness check
after that last return. The follow-up waits for those native navigation
preconditions before taking focus and adds failure-only state/screenshot capture.
No product focus behavior is changed. The exact lost-focus timing was not
captured in the initial run; retain that limit when interpreting a later pass.

Visual review of the first seven statement PDF pages exposes an existing print
layout defect: authored home content starts directly below the table of contents.
The adapter now starts that content on its own page before emitting its stable
target. Native source assertions and real PDF page-text checks enforce the
separation. This affects books with an authored home; all 105 existing baseline
captures matched before this change and will be compared again after it.

The separated home-page capture at `2ddd17a` also reveals an inherited `CONTENTS`
running header. The follow-up resets both running marks to the escaped home title
and uses the plain first-page style, including a case-insensitive PDF regression
for the old contents heading. No other publication layout is changed.

Bounded hosted comparison `43565934456` at probe source `a8f7355` runs 20 attempts
each, alternating the original and revised helper on the same native manuscript,
macOS WebKit, with JavaScript disabled and no retries. The original fails its
focus assertion 4/20 times (rounds 7, 8, 9, 18); the revised helper passes 20/20.
This supports the readiness correction. The shared failure sampler originally
waited for an in-page timer, which cannot run with JavaScript disabled; those
four state captures time out and are retained. Failure capture now has a bounded
no-frame mode that returns native DOM/focus state without page timers. The new
helper selects that mode; other existing callers retain frame sampling.

The no-frame follow-up `43566630220` at probe source `048008e` repeats ten
attempts per variant without retries. Original readiness fails 2/10; revised
readiness passes 10/10. Both captured failures have the document focused, fonts
loaded, the theorem fragment in the URL and `BODY` as the active element after
the timeout. These snapshots are after the failure, not a trace of the exact
focus transition. Across the two bounded runs, original readiness fails 6/30
and revised readiness passes 30/30 on the same product output. This is evidence
for the test-readiness correction, not a product-script focus fix.

Final-source PDF jobs at `6b76504` in `43566612273` pass on all three runtimes:
18 PDFs and 129 captures. The 105 earlier layouts are byte-identical PNGs; all
24 new statement captures match the eight reviewed pages across runtimes.
The final home-page capture has its own page, no inherited contents header and
correct roman folio. Other source gates are still running at this checkpoint.

Run `43566612273` then exposes the same missing navigation-readiness precondition
in the older equation helper on Linux (job `132361846213`): the `Equation 1.1`
self-link is inactive after the return from another document. The helper now
uses the same URL/load/font/target readiness before focus and the bounded
no-script failure capture. Both helpers capture full-page layouts from the top
only after their keyboard assertions. Product code is unchanged by this
follow-up; the final combined-source CI must pass before merge.
