import type { SectionNode } from '../reuse-schema';
import type { Site } from '../../content/site';
import type { MotionPreset } from '../../config/schema';
import { FeatureGrid } from './feature-grid';
import { PricingTable } from './pricing-table';
import { StatsBand } from './stats-band';
import { CtaBanner } from './cta-banner';
import { BentoGrid } from './bento-grid';
import { Timeline } from './timeline';
import { TeamGrid } from './team-grid';
import { LogoStrip } from './logo-strip';

/** Only the reviewed, finite schema is dispatched here. Never consult the open
 * section registry: wrappers may register executable or recursively composed types. */
export function SectionContent({ node, site, motion }: { node: SectionNode; site: Site; motion: MotionPreset }) {
  const props = {site, defaultMotion: motion, headingLevel: node.headingLevel as 2 | 3 | 4};
  const frame = {background: 'default' as const, padding: 'none' as const};
  const content = node.content;
  const render = () => {
    switch (content.type) {
      case 'feature-grid': return <FeatureGrid {...props} section={{...content, ...frame}} />;
      case 'pricing-table': return <PricingTable {...props} section={{...content, ...frame}} />;
      case 'stats-band': return <StatsBand {...props} section={{...content, ...frame}} />;
      case 'cta-banner': return <CtaBanner {...props} section={{...content, ...frame}} />;
      case 'bento-grid': return <BentoGrid {...props} section={{...content, ...frame}} />;
      case 'timeline': return <Timeline {...props} section={{...content, ...frame}} />;
      case 'team-grid': return <TeamGrid {...props} section={{...content, ...frame}} />;
      case 'logo-strip': return <LogoStrip {...props} section={{...content, ...frame}} />;
    }
  };
  return <div className="zellij-compose-section" role="group" aria-label={node.label}>{render()}</div>;
}
