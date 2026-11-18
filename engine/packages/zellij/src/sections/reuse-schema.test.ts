import { expect, it } from 'vitest';
import { composeSchema } from './compose-schema';
import { schemaReferences } from '../content/reference-schema';
import { buildSectionTemplates } from '../cli/section-templates';
import { parse } from 'yaml';
const types = ['feature-grid','pricing-table','stats-band','cta-banner','bento-grid','timeline','team-grid','logo-strip'];
const node = (content: unknown) => ({type:'section',label:'Reusable content',content});
const composition = (child: unknown) => ({type:'compose',version:1,label:'Composed',children:[child]});
const content = {type:'feature-grid',heading:'A feature',items:[{title:'First',body:'Read [more](/guide)',href:'/details'}]};
it('accepts every supported built-in with defaults inside ordinary groups', () => {
  const templates=buildSectionTemplates();
  for(const type of types) {
    const value=parse(templates[type]!);
    for(const key of ['id','motion','background','padding']) delete value[key];
    const result=composeSchema.parse(composition({type:'group',children:[node(value)]}));
    expect(result.children[0]).toMatchObject({children:[{headingLevel:2,content:{type}}]});
  }
});
it('rejects unsupported or recursive types, owned frame fields and oversized collections', () => {
  for(const bad of [
    ...['hero','compose','markdown-prose','custom'].map(type=>({...content,type})),
    ...['id','motion','background','padding','unknown'].map(key=>({...content,[key]:'invalid'})),
    {...content,items:Array(13).fill({title:'Too many'})},
  ]) expect(composeSchema.safeParse(composition(node(bad))).success).toBe(false);
  for(const level of [1,5,2.5]) expect(composeSchema.safeParse(composition({...node(content),headingLevel:level})).success).toBe(false);
  for(const extra of [{place:'center'},{effect:'reveal'}]) expect(composeSchema.safeParse(composition({...node(content),...extra})).success).toBe(false);
});
it('rejects embedding below layers or scroll stages through groups', () => {
  const value=composition({type:'group',children:[node(content)]});
  expect(composeSchema.safeParse({...value,scroll:{from:{},to:{}}}).success).toBe(false);
  expect(composeSchema.safeParse({...value,layout:{kind:'layers',backdrop:{kind:'image',src:'assets/a.svg'}}}).success).toBe(false);
});
it('retains asset, inline Markdown and link references within embedded data', () => {
  const value=composeSchema.parse(composition(node({type:'team-grid',people:[{name:'Author',photo:'assets/author.svg',bio:'Read [more](/guide)',links:[{label:'Details',href:'/details'}]}]})));
  expect(schemaReferences(composeSchema,value)).toEqual(expect.arrayContaining([
    {kind:'asset',field:'children[0].content.people[0].photo',value:'assets/author.svg'},
    {kind:'inline',field:'children[0].content.people[0].bio',value:'Read [more](/guide)'},
    {kind:'href',field:'children[0].content.people[0].links[0].href',value:'/details'},
  ]));
});
