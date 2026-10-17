import { z } from 'zod';
import { parse, stringify } from 'yaml';
import { getSection, registeredSectionTypes } from '../sections/registry';
import '../sections/schemas';

type JsonSchema = {
  type?: string;
  properties?: Record<string, JsonSchema>;
  required?: string[];
  items?: JsonSchema;
  anyOf?: JsonSchema[];
  oneOf?: JsonSchema[];
  enum?: unknown[];
  const?: unknown;
  default?: unknown;
  minItems?: number;
  maxItems?: number;
  minimum?: number;
};

const shared = new Set(['type', 'id', 'motion', 'background', 'padding', 'titleGradient']);

/** JSON Schema supplies shapes and bounds; Zod below enforces refinements too. */
function example(schema: JsonSchema, name: string): unknown {
  if (schema.const !== undefined) return schema.const;
  // Show the entry shape even when an optional list defaults to [].
  if (schema.type === 'array' && schema.items) {
    const count = Math.max(schema.minItems ?? 0, schema.maxItems === 0 ? 0 : 1);
    return Array.from({ length: count }, () => example(schema.items!, name));
  }
  if (schema.default !== undefined) return structuredClone(schema.default);
  if (schema.enum) return schema.enum[0];
  const options = schema.anyOf ?? schema.oneOf;
  if (options) return example(options.find((option) => option.type !== 'null')!, name);
  if (schema.type === 'object') {
    return Object.fromEntries((schema.required ?? []).map((key) => [key, example(schema.properties![key]!, key)]));
  }
  if (schema.type === 'boolean') return false;
  if (schema.type === 'number' || schema.type === 'integer') return schema.minimum ?? 0;
  if (schema.type === 'string') {
    if (['image', 'visual', 'logos', 'avatar', 'photo', 'poster'].includes(name)) return 'assets/replace-me.svg';
    if (name === 'video' || name === 'src') return 'assets/replace-me.mp4';
    if (name === 'href') return '#';
    if (name === 'diagram') return 'flowchart LR\n  Start --> Finish';
    return 'TODO';
  }
  throw new Error(`Cannot generate an example for ${name}; add an explicit starter.`);
}

export function buildSectionTemplates(): Record<string, string> {
  return Object.fromEntries(registeredSectionTypes().map((type) => {
    const definition = getSection(type)!;
    const schema = definition.schema as z.ZodType;
    const json = z.toJSONSchema(schema, { io: 'input', unrepresentable: 'any' }) as JsonSchema;
    const live = example(json, type) as Record<string, unknown>;
    let note = '';

    // These choices cannot be expressed by the JSON Schema exported from Zod.
    if (type === 'hero-split' || type === 'steps-panel') {
      live.terminal = { body: '$ zel --help' };
      note = `# Keep exactly one of: ${type === 'steps-panel' ? 'visual, terminal, diagram' : 'visual, terminal'}.\n`;
    } else if (type === 'media-panel') {
      live.image = 'assets/replace-me.svg';
      note = '# Keep exactly one of: image, video.\n';
    } else if (type === 'matrix') {
      const columns = live.columns as unknown[];
      for (const row of live.rows as Array<{ cells: string[] }>) {
        row.cells = columns.map(() => 'TODO');
      }
    }

    let body = stringify(live, { lineWidth: 0 });
    if (body.includes('assets/replace-me.')) {
      body += '\n# Media paths are placeholders. Add your own files under content/assets\n# and update these paths before publishing.\n';
    }
    body += `\n${note}# Uncomment what you need.\n`;
    for (const [name, field] of Object.entries(json.properties ?? {})) {
      if (shared.has(name) || Object.hasOwn(live, name)) continue;
      if (field.enum) body += `# ${field.enum.join(' | ')}\n`;
      const yaml = stringify({ [name]: example(field, name) }, { lineWidth: 0 });
      body += yaml.trimEnd().split('\n').map((line) => `# ${line}\n`).join('');
    }

    // Validate the actual serialized artifact, not just a parallel JS fixture.
    const result = schema.safeParse(parse(body));
    if (!result.success) throw new Error(`Invalid ${type} starter: ${result.error.message}`);
    return [type, body];
  }));
}
