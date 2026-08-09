import type { SectionProps } from '../types';
import type { MatrixSection } from '../schemas';
import { InlineMarkdown, SectionHeader, SectionShell } from '../shared';
import { Reveal } from '../../motion/reveal';

/**
 * Two axes crossed.
 *
 * A real `<table>`, with the column names as `<th scope="col">` and each row's
 * name as `<th scope="row">`. That pairing is not markup pedantry: it is what
 * makes a screen reader announce "passphrase, portal-key" before reading a
 * cell, and the cell means nothing without it. A grid of divs looks identical
 * and says, to anyone not looking at it, eight unrelated sentences.
 *
 * Below the breakpoint the same table restacks into one block per row, with
 * each cell labelled by its column. The markup does not change — only the CSS —
 * so the reading order and the header associations survive the layout.
 */
export function Matrix({ section, defaultMotion }: SectionProps<MatrixSection>) {
  const motion = section.motion ?? defaultMotion;

  return (
    <SectionShell
      type="matrix"
      background={section.background}
      padding={section.padding}
      {...(section.id ? { id: section.id } : {})}
      motion={motion}
      className="zellij-matrix"
    >
      <SectionHeader
        {...(section.eyebrow ? { eyebrow: section.eyebrow } : {})}
        {...(section.heading ? { heading: section.heading } : {})}
        {...(section.subhead ? { subhead: section.subhead } : {})}
        motion={motion}
        align="start"
        {...(section.titleGradient ? { titleGradient: section.titleGradient } : {})}
      />

      <Reveal motion={motion} className="zellij-matrix-scroll">
        <table className="zellij-matrix-table">
          <thead>
            <tr>
              <th scope="col" className="zellij-matrix-corner">
                {section.rowAxis || section.columnAxis ? (
                  <>
                    {section.rowAxis ? <span>{section.rowAxis} ↓</span> : null}
                    {section.columnAxis ? <span>{section.columnAxis} →</span> : null}
                  </>
                ) : null}
              </th>
              {section.columns.map((column) => (
                <th scope="col" key={column.label} className="zellij-matrix-column">
                  {column.label}
                </th>
              ))}
            </tr>
          </thead>

          <tbody>
            {section.rows.map((row) => (
              <tr key={row.label}>
                <th scope="row" className="zellij-matrix-row">
                  <span className="zellij-matrix-row-label">{row.label}</span>
                  {row.description ? (
                    <span className="zellij-matrix-row-description">
                      <InlineMarkdown text={row.description} />
                    </span>
                  ) : null}
                </th>

                {row.cells.map((cell, index) => (
                  <td
                    key={section.columns[index]?.label ?? index}
                    /* Repeated on every cell so the stacked layout can label it
                       with its column via CSS, without a second copy of the
                       column names in the markup. */
                    data-column={section.columns[index]?.label}
                  >
                    <InlineMarkdown text={cell} />
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </Reveal>

      {section.footnote ? (
        <Reveal motion={motion} as="p" className="zellij-matrix-footnote" index={1}>
          <InlineMarkdown text={section.footnote} />
        </Reveal>
      ) : null}
    </SectionShell>
  );
}
