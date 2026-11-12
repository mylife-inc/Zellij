import { stringify } from 'yaml';
import { validateSection } from '../content/model-schema';
import '../sections/schemas';

/** Presets expand once into ordinary authored YAML; rendering has no preset lookup. */
const presets: Record<string, { description: string; section: Record<string, unknown> }> = {
  'product-hero': { description: 'A generous product introduction with a gradient title and call to action.', section: {
    type: 'compose', version: 1, label: 'Introduction', padding: 'loose',
    layout: { kind: 'stack', align: 'center', gap: 'large' }, children: [
      { type: 'heading', level: 1, scale: 'display', text: 'Make room for your next idea.', gradient: { colors: ['#b94c25', '#df9b55'], effect: 'reveal' } },
      { type: 'text', size: 'lead', text: 'A clear introduction to what you make, who it helps, and why it matters.' },
      { type: 'button', label: 'Explore the story', href: '#' },
    ],
  } },
  'split-story': { description: 'An editorial image and a compact story, with a reversible split layout.', section: {
    type: 'compose', version: 1, label: 'The story', layout: { kind: 'split', ratio: 'wide-start', align: 'center', gap: 'large' }, children: [
      { type: 'image', image: 'assets/replace-me.svg', alt: 'Describe the story illustrated by your image', caption: 'Add context to the illustration.' },
      { type: 'group', children: [
        { type: 'heading', text: 'Every detail has a reason.' },
        { type: 'text', size: 'lead', text: 'Tell the story behind one thoughtful choice. Keep the focus on what it means for your reader.' },
        { type: 'button', label: 'Read more', href: '#', style: 'ghost' },
      ] },
    ],
  } },
  'feature-cards': { description: 'Three equal-height feature cards with theme-aware surfaces.', section: {
    type: 'compose', version: 1, label: 'Details', children: [
      { type: 'heading', text: 'Small details. Lasting differences.' },
      { type: 'group', layout: { kind: 'grid', columns: 3, align: 'stretch' }, children: [
        ['A clear beginning.', 'Explain how your reader gets started.'],
        ['Room to grow.', 'Describe how the experience adapts to new needs.'],
        ['Thoughtfully finished.', 'Show the care behind one useful detail.'],
      ].map(([text, body]) => ({ type: 'group', surface: 'tinted', padding: 'medium', radius: 'large', children: [
        { type: 'heading', level: 3, scale: 'subtitle', text }, { type: 'text', text: body },
      ] })) },
    ],
  } },
  'media-accordion': { description: 'Selectable feature stories with a synchronized illustration and sequential fallbacks.', section: {
    type: 'compose', version: 1, label: 'A closer look', children: [
      { type: 'heading', text: 'A closer look at the details.' },
      { type: 'media-accordion', label: 'Explore the details', items: [
        { title: 'Start with the essentials.', body: 'Explain one useful feature in **plain language**.', image: 'assets/replace-me.svg', alt: 'Describe the first feature illustration' },
        { title: 'Find a new perspective.', body: 'Give the reader another reason to explore.', image: 'assets/replace-me.svg', alt: 'Describe the second feature illustration' },
        { title: 'Make it your own.', body: 'Show how this detail fits the reader’s needs.', image: 'assets/replace-me.svg', alt: 'Describe the third feature illustration' },
      ] },
    ],
  } },
};

export function buildSectionPresets(): Record<string, { sectionType: string; description: string; yaml: string }> {
  return Object.fromEntries(Object.entries(presets).map(([name, { description, section }]) => {
    validateSection(`preset:${name}`, section, warning => { throw new Error(warning); });
    let yaml = `# Composition preset: ${name}. Edit this file freely.\n${stringify(section, { lineWidth: 0 })}`;
    if (yaml.includes('assets/replace-me.')) yaml += '\n# Replace the placeholder images with your own content/assets files before building.\n';
    return [name, { sectionType: String(section.type), description, yaml }];
  }));
}
