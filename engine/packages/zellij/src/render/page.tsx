import type { Metadata } from 'next';
import { notFound, redirect } from 'next/navigation';

import { loadZellijRuntime, type ZellijRuntimeOptions } from '../config/runtime';
import { assetUrl, routeToSegments, segmentsToRoute, withBasePath } from '../content/paths';
import type { GuidePage, SectionsPage } from '../content/types';
import { flattenSidebar } from '../guide/navigation';
import { SectionsPageView } from './sections-page';
import { GuidePageView } from './guide-page';

export interface ZellijPageProps {
  params: Promise<{ slug?: string[] }>;
}

type Resolved =
  | { kind: 'sections'; page: SectionsPage }
  | { kind: 'guide'; page: GuidePage }
  | { kind: 'redirect'; to: string }
  | { kind: 'none' };

/**
 * Wires Zellij into a wrapper app's catch-all route. The wrapper needs no logic
 * of its own beyond re-exporting what this returns (spec §9).
 */
export function createZellijPages(options: ZellijRuntimeOptions = {}) {
  async function resolve(slug: string[] | undefined): Promise<Resolved> {
    const { site } = await loadZellijRuntime(options);
    const route = segmentsToRoute(slug);

    const page = site.pages.find((p) => p.route === route);
    if (page) return { kind: 'sections', page };

    const guidePage = site.guidePages.find((p) => p.route === route);
    if (guidePage) return { kind: 'guide', page: guidePage };

    // "/" when the home destination lives at its own route (design §4).
    if (route === '/' && site.homeRedirect) return { kind: 'redirect', to: site.homeRedirect };

    // A guide folder with no index.mdx: its root sends the reader to the first
    // page in its navigation rather than rendering an empty shell.
    const tree = site.guides.find((guide) => guide.route === route);
    const first = tree ? flattenSidebar(tree.sidebar)[0] : undefined;
    if (first) return { kind: 'redirect', to: first.route };

    return { kind: 'none' };
  }

  async function generateStaticParams(): Promise<Array<{ slug: string[] }>> {
    const { site } = await loadZellijRuntime(options);
    return [...site.routes].map((route) => ({ slug: routeToSegments(route) }));
  }

  async function generateMetadata({ params }: ZellijPageProps): Promise<Metadata> {
    const { site } = await loadZellijRuntime(options);
    const { slug } = await params;
    const resolved = await resolve(slug);

    if (resolved.kind === 'none' || resolved.kind === 'redirect') {
      return { title: site.config.name };
    }

    const { page } = resolved;
    const description =
      resolved.kind === 'sections' ? page.description : (page.description ?? site.config.tagline);

    const title =
      resolved.kind === 'sections' ? page.title : `${page.title} — ${site.config.name}`;

    const canonical = withBasePath(page.route, site.config.basePath);

    const metadata: Metadata = {
      title,
      ...(description ? { description } : {}),
      openGraph: {
        title,
        ...(description ? { description } : {}),
        siteName: site.config.name,
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
    if (resolved.kind === 'redirect') redirect(resolved.to);

    if (resolved.kind === 'sections') {
      return <SectionsPageView site={site} page={resolved.page} />;
    }

    return <GuidePageView site={site} page={resolved.page} />;
  }

  // `Page` is returned by name rather than as `default`: destructuring a key
  // called `default` does not produce a module default export, which the App
  // Router requires. The wrapper does `export default Page` (spec §9).
  return { generateStaticParams, generateMetadata, Page };
}
