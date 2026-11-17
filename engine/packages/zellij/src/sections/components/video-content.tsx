import type { ReactNode } from 'react';
import Link from 'next/link';
import type { VideoTrack } from '../video-schema';
import { assetHref } from '../../content/asset-href';
import { VideoPlayer } from './video-player';

/** Match provider hosts, never a provider name embedded in an unrelated URL. */
export function videoEmbedUrl(src: string): string | undefined {
  let url: URL;
  try { url = new URL(src); } catch { return undefined; }
  if (!['http:', 'https:'].includes(url.protocol)) return undefined;
  const host = url.hostname.toLowerCase();
  let id: string | null | undefined;
  if (host === 'youtu.be') id = url.pathname.slice(1);
  if (['youtube.com', 'www.youtube.com', 'm.youtube.com', 'youtube-nocookie.com', 'www.youtube-nocookie.com'].includes(host)) {
    id = url.pathname === '/watch' ? url.searchParams.get('v') : /^\/(?:embed|shorts)\/([^/]+)\/?$/.exec(url.pathname)?.[1];
  }
  if (id && /^[\w-]{11}$/.test(id)) return `https://www.youtube-nocookie.com/embed/${id}`;
  if (['vimeo.com', 'www.vimeo.com', 'player.vimeo.com'].includes(host)) {
    const match = /^\/(?:video\/)?(\d+)\/?$/.exec(url.pathname);
    if (match) return `https://player.vimeo.com/video/${match[1]}`;
  }
  return undefined;
}

export function VideoContent({ src, poster, label, tracks, transcript, caption, basePath, composed = false }: {
  src: string; poster?: string; label: string; tracks?: VideoTrack[]; transcript?: string;
  caption?: ReactNode; basePath: string; composed?: boolean;
}) {
  const embedUrl = videoEmbedUrl(src);
  const sourceUrl = assetHref(src, basePath);
  return <figure className={composed ? 'zellij-video-figure zellij-compose-video' : 'zellij-video-figure'}>
    <VideoPlayer
      {...(embedUrl ? { embedUrl } : { fileUrl: sourceUrl })}
      {...(poster ? { poster: assetHref(poster, basePath) } : {})}
      {...(tracks?.length ? { tracks: tracks.map(track => ({ ...track, src: assetHref(track.src, basePath) })) } : {})}
      label={label}
    />
    <figcaption>
      {caption ? <div className="zellij-video-caption">{caption}</div> : null}
      <p className="zellij-video-links">
        <a href={sourceUrl}>Open video: {label}</a>
        {transcript ? <> · <Link href={transcript}>Read transcript</Link></> : null}
      </p>
    </figcaption>
  </figure>;
}
