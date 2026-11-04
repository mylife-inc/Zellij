import { afterEach, expect, it, vi } from 'vitest';
import { watchDevelopmentRevision } from './development-reload';

const stops: Array<() => void> = [];
afterEach(() => {
  stops.splice(0).forEach((stop) => stop());
  vi.useRealTimers();
  vi.unstubAllGlobals();
});
const response = (revision: string) => new Response(JSON.stringify({ version: 1, revision }));

it('reloads once for a new prepared revision, including a change before the first poll', async () => {
  vi.useFakeTimers();
  const fetch = vi.fn().mockResolvedValueOnce(response('initial')).mockResolvedValue(response('changed'));
  vi.stubGlobal('fetch', fetch);
  const reload = vi.fn();
  stops.push(watchDevelopmentRevision('/docs/_zellij/dev-revision.json', 'initial', reload));
  await vi.advanceTimersByTimeAsync(0);
  expect(reload).not.toHaveBeenCalled();
  expect(fetch).toHaveBeenCalledWith('/docs/_zellij/dev-revision.json', expect.objectContaining({ cache: 'no-store' }));
  await vi.advanceTimersByTimeAsync(1000);
  expect(reload).toHaveBeenCalledTimes(1);
  await vi.advanceTimersByTimeAsync(10000);
  expect(fetch).toHaveBeenCalledTimes(2);

  const earlyChange = vi.fn();
  fetch.mockResolvedValue(response('already-changed'));
  stops.push(watchDevelopmentRevision('/_zellij/dev-revision.json', 'initial', earlyChange));
  await vi.advanceTimersByTimeAsync(0);
  expect(earlyChange).toHaveBeenCalledTimes(1);
});

it('retains the page through missing, invalid and failed responses, then recovers', async () => {
  vi.useFakeTimers();
  const fetch = vi.fn()
    .mockRejectedValueOnce(new Error('server restarting'))
    .mockResolvedValueOnce(new Response('', { status: 404 }))
    .mockResolvedValueOnce(new Response('not JSON'))
    .mockResolvedValueOnce(new Response('{"version":2,"revision":"future"}'))
    .mockResolvedValueOnce(response(''))
    .mockResolvedValueOnce(response('initial'))
    .mockResolvedValueOnce(response('recovered'));
  vi.stubGlobal('fetch', fetch);
  const reload = vi.fn();
  stops.push(watchDevelopmentRevision('/_zellij/dev-revision.json', 'initial', reload));
  await vi.advanceTimersByTimeAsync(5000);
  expect(reload).not.toHaveBeenCalled();
  await vi.advanceTimersByTimeAsync(1000);
  expect(reload).toHaveBeenCalledTimes(1);
});

it('bounds stalled requests, never overlaps polls and cancels on unmount', async () => {
  vi.useFakeTimers();
  const signals: AbortSignal[] = [];
  const fetch = vi.fn((_url: string, options: RequestInit) => new Promise((_resolve, reject) => {
    const signal = options.signal!;
    signals.push(signal);
    signal.addEventListener('abort', () => reject(new Error('aborted')), { once: true });
  }));
  vi.stubGlobal('fetch', fetch);
  const reload = vi.fn();
  const stop = watchDevelopmentRevision('/_zellij/dev-revision.json', 'initial', reload);
  stops.push(stop);
  await vi.advanceTimersByTimeAsync(4999);
  expect(fetch).toHaveBeenCalledTimes(1);
  expect(signals[0]!.aborted).toBe(false);
  await vi.advanceTimersByTimeAsync(1001);
  expect(signals[0]!.aborted).toBe(true);
  expect(fetch).toHaveBeenCalledTimes(2);
  stop();
  await vi.advanceTimersByTimeAsync(10000);
  expect(signals[1]!.aborted).toBe(true);
  expect(fetch).toHaveBeenCalledTimes(2);
  expect(reload).not.toHaveBeenCalled();
  expect(vi.getTimerCount()).toBe(0);
});
