import { expect, it } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { storyNodeSchema } from '../story-schema';
import { StoryContent } from './story-content';
it('renders every scene as readable content with native video controls before enhancement', () => {
  const node = storyNodeSchema.parse({type:'story',label:'The story',scenes:[
    {heading:'First chapter',body:'A **clear** beginning.',media:{kind:'image',src:'assets/start.svg',alt:'A paper study'}},
    {heading:'Second chapter',body:'The fold opens.',media:{kind:'video',src:'assets/fold.mp4',poster:'assets/start.svg',label:'The fold',start:1,end:3,tracks:[{src:'assets/fold.vtt',srclang:'en',label:'English'}],transcript:'/transcript'}},
  ]});
  const html = renderToStaticMarkup(<StoryContent node={node} basePath="/product" motion="expressive" />);
  expect(html).toContain('data-story-active="false"');
  expect(html).toContain('hidden="" aria-disabled="false"');
  expect(html.indexOf('First chapter')).toBeLessThan(html.indexOf('Second chapter'));
  expect(html).toContain('<strong>clear</strong>');
  expect(html).toContain('controls=""');
  expect(html).toContain('preload="none"');
  expect(html).not.toMatch(/autoplay|autoPlay|position:sticky/);
  expect(html).toContain('/product/_zellij/assets/fold.mp4');
  expect(html).toContain('/product/_zellij/assets/fold.vtt');
  expect(html).toContain('href="/transcript"');
  expect(html).toContain('<h3');
  expect(html).not.toContain('<h1');
});
