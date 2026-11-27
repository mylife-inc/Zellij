// Keep the dependency-free inspector bound to the YAML used by the generator.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

const sourceHash = bytes => createHash('sha256').update(bytes).digest('hex');
export function imageInspectionMetadata(entry, config, workflow, bytes, regenerate = false) {
  if (config.publish?.action !== 'push') {
    assert(!entry.image && !entry.configSha256, 'Non-image fixture has inspection metadata');
    return null;
  }
  assert(['static', 'next'].includes(entry.name));
  assert.equal(config.publish.image, entry.name, 'Publication kind differs from inspection');
  const image = config.image?.name;
  const version = config.ci?.['zel-version']?.replace(/^v/, '');
  assert(version && /^\d+\.\d+\.\d+(?:-[\w.-]+)?$/.test(version), 'Expected exact fixture CLI version');
  assert.equal(image, `ghcr.io/mylife-inc/zellij/acceptance-${entry.name}:${version}`, 'Publication image differs from fixture version');
  const tags = workflow.jobs.publish?.steps?.filter(step => step.env?.IMAGE_TAG).map(step => step.env.IMAGE_TAG);
  assert.deepEqual(tags, [image], 'Generated publication target differs from fixture image');
  const metadata = { image, configSha256: sourceHash(bytes) };
  if (!regenerate) assert.deepEqual({ image: entry.image, configSha256: entry.configSha256 }, metadata, `${entry.workflow}: inspection target is stale; regenerate with --write`);
  return metadata;
}

export function inspectionImage(entry, bytes) {
  assert(['static', 'next'].includes(entry?.name), 'Expected image fixture');
  const prefix = `ghcr.io/mylife-inc/zellij/acceptance-${entry.name}:`;
  assert(typeof entry.image === 'string' && entry.image.startsWith(prefix) && /^[A-Za-z0-9_][A-Za-z0-9_.-]*$/.test(entry.image.slice(prefix.length)), 'Unexpected inspection image');
  assert.equal(entry.configSha256, sourceHash(bytes), 'Inspection metadata is stale; run prepare-delivery-acceptance.mjs --write');
  return entry.image;
}
