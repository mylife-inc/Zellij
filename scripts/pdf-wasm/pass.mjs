// One disposable worker per LuaHBTeX pass. The guest receives MEMFS and a fixed
// resource map, with no host filesystem, process environment or network bridge.
// This is a feasibility runner; production isolation qualification is still open.
import { createContext, runInContext } from 'node:vm';
import { parentPort, workerData } from 'node:worker_threads';
import path from 'node:path';

const { bundle, files } = workerData;
const heapLimit = 512 * 1024 * 1024;
const fileLimit = 128 * 1024 * 1024;
const fsLimit = 256 * 1024 * 1024;
// Tests may lower the console budget; callers can never raise it.
const requestedLogLimit = workerData.logBytes ?? 8 * 1024 * 1024;
if (!Number.isSafeInteger(requestedLogLimit) || requestedLogLimit < 1024 || requestedLogLimit > 8 * 1024 * 1024) {
  throw new Error('Invalid PDF probe console budget');
}
const logLimit = requestedLogLimit;
let log = '';
let logBytes = 0;
let failed = false;
const misses = new Set();
let finishInit;
const initialized = new Promise(resolve => { finishInit = resolve; });
const context = createContext({
  WebAssembly, TextDecoder, TextEncoder, Uint8Array, ArrayBuffer, performance,
}, { codeGeneration: { strings: false, wasm: true } });

function violation(message) {
  // libc may translate a JS exception from a stream into EIO and keep running.
  // Notify the supervising thread first so it terminates the guest immediately.
  if (!failed) {
    failed = true;
    parentPort.postMessage({ error: message, console: log.slice(-16 * 1024) });
  }
  return new Error(message);
}

function put(file, bytes) {
  context.FS.mkdirTree(path.posix.dirname(file));
  context.FS.writeFile(file, bytes);
}

function resolveResource(name, format) {
  // Generated inputs are opened inside MEMFS before this fallback. A requested
  // resource name can never turn into a host path or an HTTP request.
  if (!/^[A-Za-z0-9_.+ -]{1,200}$/.test(name)) return undefined;
  const extension = path.posix.extname(name).toLowerCase();
  const kinds = { '.lua': 51, '.otf': 47, '.ttf': 36, '.ttc': 36, '.pfb': 32, '.afm': 4, '.tfm': 3 };
  const kind = kinds[extension] ?? format;
  const suffixes = { 3: '.tfm', 4: '.afm', 26: '.tex', 32: '.pfb', 36: '.ttf', 47: '.otf', 51: '.lua' };
  const filename = extension ? name : name + (suffixes[kind] || '');
  const keys = [`${kind}/${filename}`];
  // The pinned mirror stores TFM/VF resources with extensionless names.
  if ([3, 33].includes(format) && filename.includes('.')) keys.push(`${kind}/${filename.replace(/\.[^.]+$/, '')}`);
  for (const key of keys) {
    const bytes = bundle.resources.get(key);
    if (!bytes) continue;
    const suffix = filename.includes('.') ? '' : ({ 3: '.tfm', 33: '.vf' }[kind] || '');
    const file = `/tex/${kind}/${filename}${suffix}`;
    if (!context.FS.analyzePath(file).exists) put(file, bytes);
    return file;
  }
  if (misses.size < 4096) misses.add(`${format}/${name}`);
}

context.kpse_find_file_impl = (pointer, format) => {
  const file = resolveResource(context.UTF8ToString(pointer), format);
  if (!file) return 0;
  const bytes = new TextEncoder().encode(file + '\0');
  const result = context._malloc(bytes.length);
  context.HEAPU8.set(bytes, result);
  return result;
};
// Supported Zellij fonts are OpenType/TrueType; no bitmap-font subprocess.
context.kpse_find_pk_impl = () => 0;

function print(line) {
  logBytes += Buffer.byteLength(line) + 1;
  if (logBytes > logLimit) throw violation('PDF probe log budget exceeded');
  log += line + '\n';
}

function setup() {
  // Bound virtual file growth, including writes/truncates through C stdio.
  // Count capacity rather than used length; unlinking does not reset the budget.
  const capacities = new Map();
  let total = 0;
  for (const name of ['expandFileStorage', 'resizeFileStorage']) {
    const original = context.MEMFS[name];
    context.MEMFS[name] = (node, size) => {
      const old = capacities.get(node) || 0;
      // Emscripten expands geometrically. Reserve twice the requested storage.
      const reserved = Math.max(old, size * 2);
      if (!Number.isSafeInteger(size) || size < 0 || size > fileLimit || total + reserved - old > fsLimit) {
        throw violation('PDF probe virtual filesystem budget exceeded');
      }
      total += reserved - old;
      capacities.set(node, reserved);
      return original(node, size);
    };
  }
  context.FS.mkdir('/work');
  context.FS.mkdir('/tex');
  context.FS.chdir('/work');
  put('/work/luahbtex', '');
  put('/work/wasmtex-luatex.fmt', bundle.format);
  for (const [name, bytes] of files) put('/work/' + name, bytes);
  const search = ['TEXINPUTS', 'LUAINPUTS', 'CLUAINPUTS', 'TEXFORMATS', 'TEXFONTMAPS', 'ENCFONTS',
    'OPENTYPEFONTS', 'TTFONTS', 'T1FONTS', 'TFMFONTS', 'VFFONTS', 'TEXPSHEADERS'];
  put('/work/texmf.cnf', [
    'TEXMFCNF = .;/work', ...search.map(key => `${key} = .;/tex//`),
    'TEXMFVAR = /tex/texmf-var', 'TEXMFCACHE = /tex/texmf-var',
    'main_memory = 12000000', 'font_mem_size = 8000000', 'pool_size = 10000000',
    'buf_size = 5000000', 'hash_extra = 2000000', 'save_size = 200000', 'stack_size = 50000',
    'trie_size = 1500000', 'hyph_size = 32767', 'max_strings = 1000000', 'nest_size = 2000', 'param_size = 20000', '',
  ].join('\n'));
  put('/work/luaotfload.conf', '[db]\nupdate-live = false\n');
  put('/tex/texmf-var/luatex-cache/generic/names/luaotfload-names.lua', bundle.resources.get('51/luaotfload-names.lua'));
}

context.Module = {
  noInitialRun: true, thisProgram: '/work/luahbtex',
  print, printErr: print, preRun: setup, onRuntimeInitialized: finishInit,
  instantiateWasm(imports, receive) {
    // Minified import names are tied to the locked engine hash. Fail on absent
    // hooks; a future engine requires an explicit review of this mapping.
    const deny = ['n', 'y', 'X', 'W', 'V', 'U', 'T', 'S', 'R', 'Q', 'P', 'o', 'w', 'D', 'm'];
    for (const key of [...deny, 'Y', '$']) {
      if (typeof imports.a?.[key] !== 'function') throw new Error('PDF probe engine import mismatch');
    }
    // system(), socket operations and name resolution never reach host services.
    for (const key of deny) imports.a[key] = () => { context.setErrNo(52); return -1; };
    const resize = imports.a.Y;
    imports.a.Y = size => size <= heapLimit ? resize(size) : 0;
    imports.a.$ = () => heapLimit;
    const module = new WebAssembly.Module(bundle.wasm);
    const instance = new WebAssembly.Instance(module, imports);
    receive(instance, module);
    return instance.exports;
  },
};

try {
  runInContext(bundle.code, context, { filename: 'verified-luahbtex.js' });
  await initialized;
  context.callMain(['-no-shell-escape', '-interaction=nonstopmode', '-halt-on-error',
    '-file-line-error', '-recorder', '--fmt=wasmtex-luatex', 'book.tex']);
  if (context.EXITSTATUS !== 0) throw new Error(`LuaHBTeX exited ${context.EXITSTATUS}`);
  const read = name => {
    const stat = context.FS.lstat('/work/' + name);
    if (!context.FS.isFile(stat.mode) || stat.size > (name.endsWith('.pdf') ? fileLimit : logLimit)) {
      throw new Error('Invalid PDF probe output');
    }
    return context.FS.readFile('/work/' + name);
  };
  const aux = new Map();
  for (const name of ['book.aux', 'book.toc', 'book.out']) {
    if (context.FS.analyzePath('/work/' + name).exists) aux.set(name, read(name));
  }
  parentPort.postMessage({ pdf: read('book.pdf'), log: read('book.log'), aux,
    heapBytes: context.HEAPU8.length, misses: [...misses] });
} catch (error) {
  parentPort.postMessage({ error: error.message, console: log.slice(-16 * 1024) });
}
