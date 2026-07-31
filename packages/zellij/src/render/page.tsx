import type { Metadata } from 'next';
import { notFound } from 'next/navigation';

import { loadZellijRuntime, type ZellijRuntimeOptions } from '../config/runtime';
import { assetUrl, routeToSegments, segmentsToRoute, withBasePath } from '../content/paths';
import type { GuidePage, MarketingPage } from '../content/types';
import { MarketingPageView } from './marketing-page';
import { GuidePageView } from './guide-page';

export interface ZellijPageProps {
  params: Promise<{ slug?: string[] }>;
}

type Resolved =
  | { kind: 'marketing'; page: MarketingPage }
  | { kind: 'guide'; page: GuidePage }
  | { kind: 'none' };

/**
 * Wires Zellij into a wrapper app's catch-all route. The wrapper needs no logic
 * of its own beyond re-exporting what this returns (spec §9).
 */
export function createZellijPages(options: ZellijRuntimeOptions = {}) {
  async function resolve(slug: string[] | undefined): Promise<Resolved> {
    const { site } = await loadZellijRuntime(options);
    const route = segmentsToRoute(slug);

    const marketing = site.marketingPages.find((p) => p.route === route);
    if (marketing) return { kind: 'marketing', page: marketing };

    const guide = site.guidePages.find((p) => p.route === route);
    if (guide) return { kind: 'guide', page: guide };

    return { kind: 'none' };
  }

  async function generateStaticParams(): Promise<Array<{ slug: string[] }>> {
    const { site } = await loadZellijRuntime(options);
    return [...site.marketingPages, ...site.guidePages].map((page) => ({
      slug: routeToSegments(page.route),
    }));
  }

  async function generateMetadata({ params }: ZellijPageProps): Promise<Metadata> {
    const { site } = await loadZellijRuntime(options);
    const { slug } = await params;
    const resolved = await resolve(slug);

    if (resolved.kind === 'none') return { title: site.config.product.name };

    const { page } = resolved;
    const description =
      resolved.kind === 'marketing' ? page.description : (page.description ?? site.config.product.tagline);

    const title =
      resolved.kind === 'marketing' ? page.title : `${page.title} — ${site.config.product.name}`;

    const canonical = withBasePath(page.route, site.config.basePath);

    const metadata: Metadata = {
      title,
      ...(description ? { description } : {}),
      openGraph: {
        title,
        ...(description ? { description } : {}),
        siteName: site.config.product.name,
        type: 'website',
        url: canonical,
        ...(site.config.ogImage
          ? { images: [{ url: assetUrl(site.config.ogImage), alt: title }] }
          : {}),
      },
      twitter: {
        card: 'summary_large_image',
        title,
        ...(description ? { description } : {}),
        ...(site.config.ogImage ? { images: [assetUrl(site.config.ogImage)] } : {}),
      },
      robots: { index: true, follow: true },
    };

    // metadataBase is what turns the relative URLs above into absolute ones,
    // which OG and canonical tags require.
    if (site.config.siteUrl) {
      metadata.metadataBase = new URL(site.config.siteUrl);
      metadata.alternates = { canonical };
    }

    return metadata;
  }

  async function Page({ params }: ZellijPageProps) {
    const { site } = await loadZellijRuntime(options);
    const { slug } = await params;
    const resolved = await resolve(slug);

    if (resolved.kind === 'none') notFound();

    if (resolved.kind === 'marketing') {
      return <MarketingPageView site={site} page={resolved.page} />;
    }

    return <GuidePageView site={site} page={resolved.page} />;
  }

  // `Page` is returned by name rather than as `default`: destructuring a key
  // called `default` does not produce a module default export, which the App
  // Router requires. The wrapper does `export default Page` (spec §9).
  return { generateStaticParams, generateMetadata, Page };
}
