/**
 * A terminal transcript, as text rather than a screenshot.
 *
 * A picture of a terminal is the obvious way to do this and the wrong one: it
 * cannot be selected or copied, it is unreadable to a screen reader, it ships a
 * few hundred kilobytes, and it is wrong the day the output changes. This is a
 * `<pre>`.
 *
 * Prompt and result lines are told apart by their first character, which is the
 * same convention the transcript already uses for a human reader — `$` is a
 * command someone typed, `✓` and `✗` are outcomes. Nothing else is interpreted,
 * so a line starting with neither is left alone. That matters for the panels
 * that hold a directory tree rather than a session: `├──` is not a prompt and
 * is not styled as one.
 *
 * Its own module because two sections draw one: `hero-split` opens with a
 * session, and `steps-panel` shows the shape of a repository beside the steps
 * that produce it. One implementation, so a change to how a command line reads
 * cannot land in one of them and not the other.
 */
export function Terminal({ title, body }: { title: string; body: string }) {
  const lines = body.replace(/\n$/, '').split('\n');

  return (
    <div className="zellij-terminal">
      <div className="zellij-terminal-bar">
        <span className="zellij-terminal-lights" aria-hidden="true">
          <i />
          <i />
          <i />
        </span>
        <span className="zellij-terminal-title">{title}</span>
      </div>

      <pre className="zellij-terminal-body">
        <code>
          {lines.map((line, index) => {
            const kind = line.startsWith('$')
              ? 'command'
              : line.startsWith('✓')
                ? 'ok'
                : line.startsWith('✗')
                  ? 'fail'
                  : undefined;

            return (
              <span
                key={index}
                className="zellij-terminal-line"
                {...(kind ? { 'data-kind': kind } : {})}
              >
                {line || ' '}
                {'\n'}
              </span>
            );
          })}
        </code>
      </pre>
    </div>
  );
}
