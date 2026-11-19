import type { ImgHTMLAttributes, CSSProperties } from 'react';
import { fileHref } from './location';
type Props = Omit<ImgHTMLAttributes<HTMLImageElement>, 'src'> & {
  src: string | { src: string; width?: number; height?: number }; fill?: boolean;
  priority?: boolean; preload?: boolean; quality?: unknown; unoptimized?: boolean;
  placeholder?: string; blurDataURL?: string; loader?: unknown;
};
export default function Image({ src, fill, priority, preload, quality: _quality,
  unoptimized: _unoptimized, placeholder: _placeholder, blurDataURL: _blur, loader: _loader,
  style, loading, ...props }: Props) {
  const fillStyle: CSSProperties = fill ? { position: 'absolute', height: '100%', width: '100%',
    left: 0, top: 0, right: 0, bottom: 0, color: 'transparent' } : { color: 'transparent' };
  return <img {...(typeof src === 'object' ? { width: src.width, height: src.height } : {})}
    {...props} src={fileHref(typeof src === 'string' ? src : src.src)}
    loading={loading ?? (priority || preload ? 'eager' : 'lazy')} decoding="async"
    data-nimg={fill ? 'fill' : '1'} style={{ ...fillStyle, ...style }} />;
}
