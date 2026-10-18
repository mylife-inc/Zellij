import mermaid from 'mermaid';

const diagrams = [...document.querySelectorAll('.mermaid-diagram')];
const root = document.documentElement;
const scheme = matchMedia('(prefers-color-scheme: dark)');
let requested = 0;
let running = false;

function settings() {
  const mode = root.dataset.zellijMode;
  const dark = mode === 'dark' || (mode === 'system' && scheme.matches);
  const css = getComputedStyle(root);
  const token = (name, fallback) => css.getPropertyValue(`--zellij-${name}`).trim() || fallback;
  const fg = token('fg', dark ? '#eeeeee' : '#161616');
  const bg = token('bg', dark ? '#161616' : '#ffffff');
  const surface = token('surface', bg);
  const border = token('border', fg);
  return {
    startOnLoad: false, securityLevel: 'strict', suppressErrorRendering: true,
    maxTextSize: 65536, maxEdges: 500,
    // Author directives must not weaken safety, re-enable external math CSS,
    // or override the active site's colors. All supported layout syntax remains.
    secure: ['secure', 'securityLevel', 'startOnLoad', 'suppressErrorRendering',
      'maxTextSize', 'maxEdges', 'legacyMathML', 'forceLegacyMathML',
      'theme', 'themeVariables', 'themeCSS', 'fontFamily', 'dompurifyConfig'],
    legacyMathML: false, forceLegacyMathML: false,
    theme: 'base', fontFamily: getComputedStyle(document.body).fontFamily,
    themeVariables: {
      darkMode: dark, background: bg, mainBkg: surface,
      primaryColor: surface, primaryTextColor: fg, primaryBorderColor: border,
      secondaryColor: bg, secondaryTextColor: fg, secondaryBorderColor: border,
      tertiaryColor: surface, tertiaryTextColor: fg, tertiaryBorderColor: border,
      textColor: fg, lineColor: fg, nodeBorder: border,
      clusterBkg: surface, clusterBorder: border, edgeLabelBackground: bg,
      actorBkg: surface, actorBorder: border, actorTextColor: fg, actorLineColor: fg,
      signalColor: fg, signalTextColor: fg, sequenceNumberColor: bg,
      noteBkgColor: surface, noteTextColor: fg, noteBorderColor: border,
    },
  };
}

async function redraw() {
  requested += 1;
  if (running) return;
  running = true;
  try {
    await document.fonts.ready;
    await document.fonts.load('16px "Zellij STIX Two Math"');
    let completed = 0;
    while (completed !== requested) {
      const revision = requested;
      const config = settings();
      for (const [index, figure] of diagrams.entries()) {
        if (revision !== requested) break;
        const canvas = figure.querySelector('.mermaid-canvas');
        const source = figure.querySelector('code').textContent;
        const details = figure.querySelector('details');
        const status = figure.querySelector('[role="status"]');
        // Each rendering gets an isolated measurable DOM tree. Mermaid uses
        // global configuration, so render serially and reinitialize per graph.
        const stage = document.createElement('div');
        stage.className = 'mermaid-stage';
        stage.setAttribute('aria-hidden', 'true');
        document.body.append(stage);
        try {
          mermaid.initialize(config);
          const { svg } = await mermaid.render(`_zel-mermaid-${index + 1}-${revision}`, source, stage);
          if (revision !== requested) break;
          const template = document.createElement('template');
          template.innerHTML = svg;
          // Mermaid mindmaps repeat a group's ID on its background path and
          // emit identical gradient definitions. Keep the first target (the
          // target browser fragment references already resolve to).
          const ids = new Set();
          for (const node of template.content.querySelectorAll('[id]')) {
            if (ids.has(node.id)) node.removeAttribute('id');
            else ids.add(node.id);
          }
          canvas.replaceChildren(template.content);
          canvas.hidden = false;
          status.textContent = '';
          if (figure.dataset.status !== 'drawn') details.open = false;
          figure.dataset.status = 'drawn';
          figure.dataset.theme = root.dataset.zellijTheme;
          figure.dataset.mode = config.themeVariables.darkMode ? 'dark' : 'light';
        } catch (error) {
          if (revision !== requested) break;
          canvas.replaceChildren();
          canvas.hidden = true;
          details.open = true;
          figure.dataset.status = 'failed';
          status.textContent = `Diagram could not be drawn. ${String(error?.message || error).slice(0, 1000)}`;
        } finally { stage.remove(); }
      }
      completed = revision;
    }
  } finally { running = false; }
}

if (diagrams.length) {
  const observer = new MutationObserver(() => { void redraw(); });
  observer.observe(root, { attributes: true, attributeFilter: ['data-zellij-theme', 'data-zellij-mode'] });
  scheme.addEventListener('change', () => { void redraw(); });
  void redraw();
}
