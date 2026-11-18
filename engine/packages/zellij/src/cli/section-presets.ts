import { stringify } from 'yaml';
import { validateSection } from '../content/model-schema';
import '../sections/schemas';

/** Presets expand once into ordinary authored YAML; rendering has no preset lookup. */
const presets: Record<string, { description: string; section: Record<string, unknown> }> = {
  'product-overview': { description: 'Real feature cards, statistics, pricing and an install action in one editable composition.', section: {
    type: 'compose', version: 1, label: 'Product overview', layout: {kind: 'stack', gap: 'large'}, children: [
      {type: 'heading', text: 'Built around your next idea.', scale: 'display'},
      {type: 'section', label: 'Product features', content: {type: 'feature-grid', columns: 2, heading: 'Thoughtful by design.', items: [
        {title: 'A clear beginning', body: 'Explain the simplest way to begin.', icon: 'sparkles', href: '#'},
        {title: 'Room to grow', body: 'Describe how your product grows with its reader.', icon: 'layers', href: '#'},
      ]}},
      {type: 'section', label: 'At a glance', content: {type: 'stats-band', stats: [
        {value: '3', label: 'Replace with a meaningful measure'}, {value: '2', label: 'Describe what this number means'}, {value: '1', label: 'Give your measure context'},
      ]}},
      {type: 'section', label: 'Choose a plan', content: {type: 'pricing-table', heading: 'A plan for your next chapter.', tiers: [
        {name: 'Personal', price: 'Free', features: ['Describe what is included'], cta: {label: 'Get started', href: '#'}},
        {name: 'Studio', price: 'Contact us', features: ['Describe the studio offering'], highlighted: true, cta: {label: 'Talk to us', href: '#'}},
      ]}},
      {type: 'section', label: 'Begin creating', content: {type: 'cta-banner', headline: 'Make something worth sharing.', body: 'Replace the command and links with your next step.', command: 'zel build', cta: {label: 'Read the guide', href: '#'}}},
    ],
  } },
  'studio-profile': { description: 'A studio introduction with real bento tiles, people, milestones and a logo strip.', section: {
    type: 'compose', version: 1, label: 'Studio profile', layout: {kind: 'stack', gap: 'large'}, children: [
      {type: 'heading', text: 'People. Purpose. Possibility.', scale: 'display'},
      {type: 'section', label: 'The studio', content: {type: 'bento-grid', items: [
        {title: 'A shared point of view.', body: 'Describe what brings your work together.', image: 'assets/replace-me.svg', span: 'wide'},
        {title: 'Made with care.', body: 'Explain a detail that matters to your studio.'},
      ]}},
      {type: 'group', layout: {kind: 'split', gap: 'large'}, children: [
        {type: 'section', label: 'People', content: {type: 'team-grid', heading: 'Meet the people.', people: [
          {name: 'Your name', role: 'Your role', photo: 'assets/replace-me.svg', bio: 'Tell readers a little about your work.', links: [{label: 'Website', href: '#'}]},
        ]}},
        {type: 'section', label: 'Milestones', content: {type: 'timeline', heading: 'The story so far.', items: [
          {date: 'Beginning', title: 'One clear idea.', body: 'Describe the first milestone.'},
          {date: 'Today', title: 'A new perspective.', body: 'Describe where that idea has taken you.'},
        ]}},
      ]},
      {type: 'section', label: 'Studio marks', content: {type: 'logo-strip', heading: 'A shared identity', logos: ['assets/replace-me.svg']}},
    ],
  } },
  'scroll-story': { description: 'Ordered image scenes with optional pinning and complete reading fallbacks.', section: {
    type: 'compose', version: 1, label: 'A story in chapters', motion: 'expressive', children: [
      {type: 'heading', text: 'Follow the idea.'},
      {type: 'story', label: 'From idea to detail', scenes: [
        {heading: 'Start with a clear idea.', body: 'Introduce the thought that shaped your product.', media: {kind: 'image', src: 'assets/replace-me.svg', alt: 'Describe the starting idea'}},
        {heading: 'Bring the details into focus.', body: 'Show how that idea becomes a useful detail.', media: {kind: 'image', src: 'assets/replace-me.svg', alt: 'Describe the finished detail'}, cta: {label: 'Explore more', href: '#'}},
      ]},
    ],
  } },
  'product-selector': { description: 'Labelled product finishes with synchronized illustrations, facts and actions.', section: {
    type: 'compose', version: 1, label: 'Choose your finish', children: [
      {type: 'heading', text: 'Find your point of view.'},
      {type: 'selector', label: 'Choose a finish', presentation: 'swatches', initial: 'copper', items: [
        {id: 'silver', label: 'Silver', swatch: '#c3c5c7', heading: 'Quiet clarity.', body: 'Describe the qualities of your silver finish.', media: {image: 'assets/replace-me.svg', alt: 'Replace with a description of the silver product'}, facts: [{label: 'Finish', value: 'Silver'}], cta: {label: 'Explore silver', href: '#'}},
        {id: 'copper', label: 'Copper', swatch: '#b97650', heading: 'A warmer perspective.', body: 'Describe what makes your copper finish distinctive.', media: {image: 'assets/replace-me.svg', alt: 'Replace with a description of the copper product'}, facts: [{label: 'Finish', value: 'Copper'}], cta: {label: 'Explore copper', href: '#'}},
      ]},
    ],
  } },
  'feature-explorer': { description: 'Keyboard-operable feature tabs with complete reading and print fallbacks.', section: {
    type: 'compose', version: 1, label: 'Explore the details', children: [
      {type: 'heading', text: 'Discover what makes the difference.'},
      {type: 'explorer', label: 'Explore the features', items: [
        {id: 'design', label: 'Design', heading: 'Every detail, considered.', body: 'Tell the story behind the design.', media: {image: 'assets/replace-me.svg', alt: 'Describe the design illustration'}},
        {id: 'performance', label: 'Performance', heading: 'Ready for your next idea.', body: 'Explain what the performance means in everyday use.', facts: [{label: 'Benefit', value: 'Replace with a meaningful measure'}], media: {image: 'assets/replace-me.svg', alt: 'Describe the performance illustration'}},
        {id: 'care', label: 'Care', heading: 'Made to stay with you.', body: 'Describe the care behind the experience.', cta: {label: 'Learn more', href: '#'}},
      ]},
    ],
  } },
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
