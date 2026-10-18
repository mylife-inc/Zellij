import { beforeAll, describe, expect, it, vi } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import path from 'node:path';
import { FolderContentSource, loadSite, type Site } from '../../content';
import { videoEmbedSchema } from '../schemas';
import { VideoEmbed } from './video-embed';
import { VideoPlayer } from './video-player';

vi.mock('./video-player', () => ({ VideoPlayer: vi.fn(() => null) }));
let site: Site;
beforeAll(async () => {
  site = await loadSite(new FolderContentSource({
    contentDir: path.resolve('../../../examples/showcase/content'),
    basePath: '/products/acme/docs',
  }));
});

describe('video media resolution', () => {
  it.each([
    ['assets/demo.mp4', 'assets/poster.png', { fileUrl: '/products/acme/docs/_zellij/assets/demo.mp4', poster: '/products/acme/docs/_zellij/assets/poster.png' }],
    ['https://media.example.com/demo.mp4', 'https://media.example.com/poster.png', { fileUrl: 'https://media.example.com/demo.mp4', poster: 'https://media.example.com/poster.png' }],
    ['https://www.youtube.com/watch?v=abcdefghijk', 'assets/poster.png', { embedUrl: 'https://www.youtube-nocookie.com/embed/abcdefghijk', poster: '/products/acme/docs/_zellij/assets/poster.png' }],
    ['https://vimeo.com/12345678', undefined, { embedUrl: 'https://player.vimeo.com/video/12345678' }],
  ])('resolves %s without losing or adding an external prefix', (src, poster, expected) => {
    vi.mocked(VideoPlayer).mockClear();
    const section = videoEmbedSchema.parse({ type: 'video-embed', src, poster, heading: 'Watch the demo' });
    renderToStaticMarkup(<VideoEmbed site={site} section={section} defaultMotion="none" />);
    expect(vi.mocked(VideoPlayer).mock.calls[0]?.[0]).toEqual({ ...expected, label: 'Watch the demo' });
  });
});
