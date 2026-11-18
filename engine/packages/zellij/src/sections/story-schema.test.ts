import { expect, it } from 'vitest';
import { composeSchema } from './compose-schema';
import { schemaReferences } from '../content/reference-schema';
const image = {heading: 'Begin', body: 'See [the guide](/guide)', media: {kind: 'image', src: 'assets/one.svg', alt: 'A beginning'}};
const video = {heading: 'Continue', body: 'The detail unfolds.', media: {kind: 'video', src: 'assets/demo.mp4', poster: 'assets/one.svg', label: 'A folding study', start: 1, end: 2, tracks: [{src: 'assets/en.vtt', srclang: 'en', label: 'English'}], transcript: '/transcript'}};
const story = {type: 'story', label: 'An idea', scenes: [image, video]};
const section = (node: unknown) => ({type: 'compose', version: 1, label: 'Stories', children: [node]});
it('accepts ordered story scenes and default video starts inside ordinary groups', () => {
  const value = composeSchema.parse(section({type: 'group', children: [story]}));
  expect(value.children[0]).toMatchObject({children:[{distance:1, mediaSide:'end', sceneHeadingLevel:3}]});
  expect(composeSchema.safeParse(section({...story, scenes:[image,{...video,media:{...video.media,start:undefined}}]})).success).toBe(true);
});
it('rejects invalid segments, bounds and stages nested through intermediate groups', () => {
  for (const invalid of [
    {...story, scenes:[image]}, {...story, scenes:Array(9).fill(image)}, {...story, sceneHeadingLevel:1}, {...story, distance:3},
    ...[{start:2,end:2},{start:3,end:2},{start:-1},{end:0},{end:3601},{poster:''}].map(media => ({...story,scenes:[image,{...video,media:{...video.media,...media}}]})),
  ]) expect(composeSchema.safeParse(section(invalid)).success).toBe(false);
  const nested = section({type:'group',children:[story]});
  expect(composeSchema.safeParse({...nested,scroll:{from:{},to:{}}}).success).toBe(false);
  expect(composeSchema.safeParse({...nested,layout:{kind:'layers',backdrop:{kind:'image',src:'assets/one.svg'}}}).success).toBe(false);
});
it('retains references in every scene, including video captions and transcripts', () => {
  const value = composeSchema.parse(section(story));
  expect(schemaReferences(composeSchema,value)).toEqual(expect.arrayContaining([
    {kind:'inline',field:'children[0].scenes[0].body',value:image.body},
    {kind:'asset',field:'children[0].scenes[0].media.src',value:'assets/one.svg'},
    {kind:'asset',field:'children[0].scenes[1].media.src',value:'assets/demo.mp4'},
    {kind:'asset',field:'children[0].scenes[1].media.poster',value:'assets/one.svg'},
    {kind:'asset',field:'children[0].scenes[1].media.tracks[0].src',value:'assets/en.vtt'},
    {kind:'href',field:'children[0].scenes[1].media.transcript',value:'/transcript'},
  ]));
});
