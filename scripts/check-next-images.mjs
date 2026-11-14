// The actual CLI adapts portable Markdown; real container mode also exercises
// hosted-prefix Next servers and publication to a loopback registry.
// --local uses an isolated wrapper and no daemon.
import assert from 'node:assert/strict';
import { checkEquations } from './check-equations.mjs';
import { checkStatements } from './check-statements.mjs';
import { execFileSync, spawn } from 'node:child_process';
import { chmod, cp, mkdir, mkdtemp, readFile, rm, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import net from 'node:net';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { localRegistry, pullPublishedImage } from './fixtures/local-registry.mjs';

const repo = fileURLToPath(new URL('../', import.meta.url));
const zel = path.resolve(process.argv[2] ?? 'cli/target/debug/zel');
const local = process.argv.includes('--local');
const root = await mkdtemp(path.join(tmpdir(), 'zellij-next-images-'));
const suffix = `${process.pid}-${Date.now()}`;
const images = [], containers = [], servers = [];
const engineCli = path.join(repo,'engine/packages/zellij/bin/zellij.mjs');
const env = {...process.env};
const docker = args => execFileSync('docker',args,{encoding:'utf8',timeout:120000}).trim();
const run = (args, extra = {}) => execFileSync(zel,args,{cwd:root,env:{...env,...extra},stdio:'inherit',timeout:600000});
async function write(relative, body) {
  const file = path.join(root,relative);
  await mkdir(path.dirname(file),{recursive:true});
  await writeFile(file,body);
}
async function ready(origin, prefix) {
  for (let n=0;n<120;n++) {
    try { if ((await fetch(`${origin}${prefix}/`,{signal:AbortSignal.timeout(1000)})).ok) return; } catch { /* startup */ }
    await new Promise(resolve => setTimeout(resolve,250));
  }
  throw new Error(`Server did not become ready at ${origin}${prefix}/`);
}
async function browserChecks(origin, prefix, portable) {
  await ready(origin,prefix);
  assert.equal((await fetch(`${origin}${prefix}/unknown-route`)).status,404);
  const browser = await chromium.launch();
  try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror',err=>errors.push(err.message));
    page.on('response',res=>{if(res.status()>=400) errors.push(`${res.status()} ${res.url()}`);});
    await page.goto(`${origin}${prefix}/`);
    if (portable === 'equations') {
      const record = await (await fetch(`${origin}${prefix}/_zellij/assets/book.json`)).json();
      assert.deepEqual(record.publication.equations.map(e => e.number), ['F1.1','1.1','1.2','2.1','A.1']);
      await checkEquations(page, `${origin}${prefix}/`, path.join(repo, 'test-results/next-equations'), 'next');
      assert.deepEqual(record.publication.statements.map(s => s.number ?? null), ['T.1','F1.1','1.1','1.2','1.3',null,'1.4','2.1','2.2','A.1']);
      await checkStatements(page, `${origin}${prefix}/`, path.join(repo, 'test-results/next-equations'), 'next-statements');
    } else if (portable === 'book') {
      await page.getByRole('heading', { name: 'An atlas of quiet ideas', exact: true }).waitFor();
      assert.equal(await page.locator('html').getAttribute('lang'), 'en');
      assert.equal(await page.locator('html').getAttribute('dir'), 'ltr');
      const cover = page.locator('article img');
      assert(await cover.evaluate(img => img.complete && img.naturalWidth > 0));
      const record = await (await fetch(`${origin}/_zellij/assets/book.json`)).json();
      assert.equal(record.publication.entries[3].id, 'questions');
      assert.equal(record.routes[0].route, '/');
      const sidebar = page.locator('.zellij-guide-sidebar');
      await sidebar.getByRole('link', { name: '1. The art of noticing', exact: true }).click();
      await page.getByRole('heading', { name: 'The art of noticing', exact: true }).waitFor();
      assert((await page.locator('article').innerText()).includes('Chapter 1'));
      await sidebar.getByRole('link', { name: '2. Better questions', exact: true }).click();
      await page.getByRole('heading', { name: 'A question worth keeping', exact: true }).waitFor();
      await page.reload();
      await page.getByRole('heading', { name: 'A question worth keeping', exact: true }).waitFor();
      const search = await (await fetch(`${origin}/_zellij/search-index.json`)).json();
      assert(search.some(e => e.text.includes('observatory')));
    } else if (portable) {
      await page.getByRole('heading',{name:'Authored title',exact:true}).waitFor();
      assert.equal(await page.locator('html').getAttribute('data-zellij-theme'),'forest');
      assert.equal(await page.getByRole('link',{name:'Company',exact:true}).getAttribute('href'),'https://company.example');
      assert.equal(await page.locator('.zellij-guide-sidebar').getByRole('link',{name:'Help elsewhere'}).getAttribute('href'),'https://company.example/help');
      assert.equal(await page.locator('article script').count(),0);
      assert((await page.locator('article').innerText()).includes('<script>{process.exit(9)}</script>'));
      assert((await page.locator('article').innerText()).includes('export const value = 1'));
      assert.equal(await page.locator('article table tbody tr').count(),1);
      assert.equal(await page.locator('article input[type=checkbox]:checked').count(),1);
      await page.locator('mjx-container svg').first().waitFor();
      await page.locator('.zellij-mindmap svg').waitFor();
      await page.locator('.zellij-mermaid svg').waitFor();
      const image = page.locator('article img[alt="Diagram"]');
      assert(await image.evaluate(img=>img.complete && img.naturalWidth>0));
      await page.locator('article').getByRole('link',{name:'Read details',exact:true}).click();
      await page.waitForURL(`${origin}/nested/page.html?from=home#exact-id`);
      assert.equal(await page.locator('#exact-id').textContent().then(t=>t.replace('#','').trim()),'Details');
      await page.reload();
      await page.getByRole('heading',{name:'Second page',exact:true}).waitFor();
      // An omitted sidebar entry still has a route, title and search entry.
      await page.goto(`${origin}/unlisted.html`);
      await page.getByRole('heading',{name:'Unlisted document',exact:true}).waitFor();
      const search = await (await fetch(`${origin}/_zellij/search-index.json`)).json();
      assert(search.some(e=>e.route==='/unlisted.html' && e.text.includes('saffron')));
      assert(search.some(e=>e.route==='/nested/page.html#exact-id'));
      assert.equal(await (await fetch(`${origin}/_zellij/assets/portable/report.pdf`)).text(),'%PDF-1.4\nfixture\n');
      for (const url of ['/index.md','/zel.yaml','/unused.txt']) assert.equal((await fetch(`${origin}${url}`)).status,404);
    } else {
      await page.waitForURL(`${origin}${prefix}/manual`);
      await page.locator('mjx-container svg').first().waitFor();
      await page.locator('.zellij-mermaid svg').waitFor();
      await page.getByRole('link',{name:'Next page',exact:true}).first().click();
      await page.waitForURL(`${origin}${prefix}/manual/next`);
      await page.reload();
      await page.getByRole('heading',{name:'More documentation',exact:true}).waitFor();
      assert.equal((await fetch(`${origin}/manual`)).status,404);
    }
    assert.deepEqual(errors,[],'Browser errors or failed resources');
  } finally { await browser.close(); }
}
async function container(tag, prefix, portable, publicOrigin) {
  const name = `zel-next-${typeof portable === 'string' ? portable : portable ? 'portable' : 'hosted'}-${suffix}`;
  containers.push(name);
  docker(['run','-d','--name',name,'-p','127.0.0.1::3000',tag]);
  const info = JSON.parse(docker(['inspect',name]))[0];
  assert.equal(info.Config.User,'nextjs');
  // Check the actual shipped filesystem, not just recipe text. Node and the
  // shell/healthcheck remain; build-only package managers and headers do not.
  docker(['exec',name,'sh','-ec','node --version; ! command -v npm; ! command -v yarn; test ! -d /usr/local/include/node; test ! -d /usr/local/lib/node_modules/npm']);
  const origin = `http://127.0.0.1:${info.NetworkSettings.Ports['3000/tcp'][0].HostPort}`;
  await ready(origin,prefix);
  if (publicOrigin) {
    const html = await (await fetch(`${origin}${prefix}/manual`)).text();
    assert(html.includes(`href="${publicOrigin}${prefix}/manual"`));
    assert((await (await fetch(`${origin}${prefix}/sitemap.xml`)).text()).includes(`<loc>${publicOrigin}${prefix}/manual</loc>`));
  }
  docker(['exec',name,'sh','-c',info.Config.Healthcheck.Test[1]]);
  await browserChecks(origin,prefix,portable);
}
try {
  await write('docs/zellij/zel.yaml', 'version: 1\nrenderer: portable\nname: Native Next\ntheme: forest\nmarkdown: {math: true, mermaid: true}\nbrand:\n  logo: {src: "logo #1%.svg", alt: Company, href: https://company.example}\nnav:\n  - {page: index.md, label: Welcome}\n  - label: Guide\n    children:\n      - {href: https://company.example/help, label: Help elsewhere}\n      - {page: nested/page.md, label: Details page}\n');
  await write('docs/zellij/content/index.md', '# Authored title {#custom-home}\n\n[Read details](nested/page.md?from=home#exact-id)\n\n![Diagram](logo%20%231%25.svg) [PDF](report.pdf)\n\n<script>{process.exit(9)}</script>\n\nexport const value = 1\n\n| Left | Right |\n| :--- | ---: |\n| A | B |\n\n- [x] Finished\n\nNote[^note].\n\n[^note]: Kept footnote.\n\n$x^2$\n\n```mindmap\nRoot: [A, B]\n```\n\n```mermaid\ngraph TD; A-->B\n```\n\n```custom-language\n{literal} <literal>\n```\n');
  await write('docs/zellij/content/nested/page.md','# Second page\n\n## Details {#exact-id}\n\n[Home](../index.md#custom-home)\n');
  await write('docs/zellij/content/unlisted.md','# Unlisted document\n\nThe saffron phrase is searchable.\n');
  await write('docs/zellij/content/logo #1%.svg','<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><circle cx="10" cy="10" r="9"/></svg>');
  await write('docs/zellij/content/report.pdf','%PDF-1.4\nfixture\n');
  await write('docs/zellij/content/unused.txt','private');
  await write('docs/zellij/site/index.html','previous output');
  const original = await readFile(path.join(root,'docs/zellij/content/index.md'),'utf8');
  if (local) {
    // Capture exactly the generated context using a test-only runtime shim.
    await write('bin/docker',`#!${process.execPath}\nconst fs=require('fs'); const a=process.argv.slice(2); if(a[0]==='info') process.exit(0); if(a[0]==='--version') {console.log('fixture');process.exit(0)}; if(a[0]!=='build') process.exit(99); fs.readFileSync(0); fs.cpSync(a.at(-1),process.env.CAPTURE,{recursive:true});\n`);
    await chmod(path.join(root,'bin/docker'),0o755);
    const adapted = path.join(root,'adapted');
    run(['ni'],{PATH:`${path.join(root,'bin')}${path.delimiter}${env.PATH}`,DOCKER_HOST:'unix:///fixture.sock',DOCKER_CONTEXT:'',CAPTURE:adapted});
    execFileSync(process.execPath,[engineCli,'check',adapted,'--deep'],{stdio:'inherit'});
    const app = path.join(root,'app');
    execFileSync(process.execPath,[engineCli,'init',app,'--name','next-acceptance','--features','all'],{stdio:'inherit'});
    await rm(path.join(app,'content'),{recursive:true,force:true});
    await cp(adapted,path.join(app,'content'),{recursive:true});
    await symlink(path.join(repo,'node_modules'),path.join(app,'node_modules'),'dir');
    execFileSync(process.execPath,[engineCli,'prepare'],{cwd:app,stdio:'inherit'});
    execFileSync(process.execPath,[path.join(repo,'node_modules/next/dist/bin/next'),'build','--webpack'],{cwd:app,env:{...env,ZELLIJ_OUTPUT:''},stdio:'inherit',timeout:600000});
    const listener=net.createServer();
    await new Promise(resolve=>listener.listen(0,'127.0.0.1',resolve));
    const port=listener.address().port; await new Promise(resolve=>listener.close(resolve));
    const child=spawn(process.execPath,[path.join(repo,'node_modules/next/dist/bin/next'),'start','--port',String(port),'--hostname','127.0.0.1'],{cwd:app,env:{...env,ZELLIJ_OUTPUT:''},stdio:'inherit'});
    servers.push(child);
    await browserChecks(`http://127.0.0.1:${port}`,'',true);
  } else {
    const registry = await localRegistry(docker, containers, suffix);
    const portable = `${registry}/acceptance/next:${suffix}`; images.push(portable);
    const config = await readFile(path.join(root,'docs/zellij/zel.yaml'),'utf8');
    await write('docs/zellij/zel.yaml',`${config.replace('math: true, mermaid: true', 'math: false, mermaid: false')}image: {name: '${portable}'}\npublish: {action: push, image: next}\n`);
    const savedConfig = await readFile(path.join(root,'docs/zellij/zel.yaml'),'utf8');
    run(['pub', '--features', 'all']);
    assert.equal(await readFile(path.join(root,'docs/zellij/zel.yaml'),'utf8'), savedConfig);
    pullPublishedImage(docker, portable);
    await container(portable,'',true);
    await write('hosted/zel.yaml','version: 1\nrenderer: hosted\n');
    const hostedSettings = 'name: Hosted Next\nhome: manual\nbasePath: /authored\nsiteUrl: https://authored.example\n';
    await write('hosted/content/site.yaml',hostedSettings);
    await write('hosted/content/manual/_sidebar.yaml','title: Manual\nnav: [index, next]\n');
    await write('hosted/content/manual/index.mdx','# Hosted Next\n\n[Next page](/manual/next)\n\n$x^2$\n\n```mermaid\ngraph TD; A-->B;\n```\n');
    await write('hosted/content/manual/next.mdx','# More documentation\n\nHosted prefix.\n');
    const hosted = `zellij-next-hosted:${suffix}`; images.push(hosted);
    run(['image','next','hosted','--tag',hosted,'--features','all','--base-path','/products/.docs','--site-url','https://next.example','--no-check']);
    assert.equal(await readFile(path.join(root,'hosted/content/site.yaml'),'utf8'),hostedSettings);
    await container(hosted,'/products/.docs',false,'https://next.example');
    await cp(path.join(repo, 'scripts/fixtures/book'), path.join(root, 'book'), { recursive: true });
    const book = `zellij-next-book:${suffix}`; images.push(book);
    run(['image', 'next', 'book', '--tag', book, '--features', 'none']);
    await container(book, '', 'book');
    await cp(path.join(repo, 'scripts/fixtures/statements'), path.join(root, 'equations'), { recursive: true });
    const equations = `zellij-next-equations:${suffix}`; images.push(equations);
    run(['image', 'next', 'equations', '--tag', equations, '--features', 'maths', '--base-path', '/books/equations']);
    await container(equations, '/books/equations', 'equations');
  }
  assert.equal(await readFile(path.join(root,'docs/zellij/site/index.html'),'utf8'),'previous output');
  assert.equal(await readFile(path.join(root,'docs/zellij/content/index.md'),'utf8'),original);
  console.log('Next images: content fidelity, navigation, search, assets, diagrams, safe text and existing-output preservation passed.');
} finally {
  for (const child of servers) {
    if(child.exitCode===null) { child.kill('SIGTERM'); await new Promise(resolve=>child.once('exit',resolve)); }
  }
  for (const name of containers) { try { docker(['rm','-f',name]); } catch {} }
  for (const tag of images) { try { docker(['image','rm',tag]); } catch {} }
  await rm(root,{recursive:true,force:true});
}
