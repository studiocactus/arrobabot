import {describe,expect,it} from 'vitest';
import {TEMPLATES,defaultDesign,designVars,goalPercent,keyTexts,normalizeDesign,overlayTree,type OverlayDesign,type Spec} from './overlayDesign';
import sharedKeys from './overlayDesign.keys.json';

const collect=(n:Spec,pick:(s:Spec)=>boolean,out:Spec[]=[]):Spec[]=>{
 if(pick(n))out.push(n);
 n.ch?.forEach(c=>collect(c,pick,out));
 return out;
};
const keysOf=(n:Spec):string[]=>collect(n,s=>!!s.k).map(s=>s.k as string);

describe('estúdio do overlay: normalização',()=>{
 it('todos os modelos passam pela normalização sem mudar',()=>{
  for(const id of Object.keys(TEMPLATES)){
   expect(normalizeDesign(TEMPLATES[id].design),`template ${id}`).toEqual(TEMPLATES[id].design);
  }
 });

 it('limita faixuras e ignora valores de injeção de CSS',()=>{
  const d=normalizeDesign({
   layout:'nope',align:'sideways',width:900,glow:-40,barHeight:0,fontWeight:555,
   font:'x;}body{background:red',fillStart:'javascript:alert(1)',panel:'red',
   fontSize:900,unknownField:1
  });
  expect(d.layout).toBe('bar');
  expect(d.align).toBe('center');
  expect(d.width).toBe(100);
  expect(d.glow).toBe(0);
  expect(d.barHeight).toBe(4);
  expect(d.fontWeight).toBe(700);
  expect(d.font).toBe(defaultDesign().font);
  expect(d.fillStart).toBe(defaultDesign().fillStart);
  expect(d.panel).toBe(defaultDesign().panel);
  expect(d.fontSize).toBe(48);
  expect('unknownField' in d).toBe(false);
 });

 it('aceita dados ausentes devolvendo o visual padrão',()=>{
  expect(normalizeDesign(null)).toEqual(defaultDesign());
  expect(normalizeDesign('nada')).toEqual(defaultDesign());
 });

 it('tem as mesmas chaves que o higienizador do Rust (paridade com livepix.rs)',()=>{
  expect(Object.keys(normalizeDesign({})).sort()).toEqual([...sharedKeys].sort());
 });
});

describe('estúdio do overlay: textos da meta',()=>{
 it('formata percentual e valores em BRL',()=>{
  expect(goalPercent({title:'',current:31000,target:50000})).toBe(62);
  expect(goalPercent({title:'',current:90000,target:0})).toBe(0);
  expect(goalPercent({title:'',current:999999,target:50000})).toBe(100);
  const t=keyTexts({title:' Meta do mês ',current:31000,target:50000});
  expect(t.title).toBe('Meta do mês');
  expect(t.pct).toBe('62%');
  expect(t.amounts).toContain('310,00');
  expect(t.amounts).toContain('500,00');
  expect(t.current).toContain('310,00');
  expect(t.target).toContain('500,00');
 });

 it('título vazio cai para "Meta"',()=>{
  expect(keyTexts({title:'   ',current:0,target:100}).title).toBe('Meta');
 });
});

describe('estúdio do overlay: árvore por formato',()=>{
 const texts=keyTexts({title:'Meta do mês',current:31000,target:50000});

 it('cada formato monta as chaves de texto certas',()=>{
  const bar=overlayTree(defaultDesign(),texts);
  expect(keysOf(bar).sort()).toEqual(['amounts','pct','title']);
  const row=overlayTree({...defaultDesign(),layout:'row'},texts);
  expect(keysOf(row).sort()).toEqual(['amounts','pct','title']);
  const panel=overlayTree({...defaultDesign(),layout:'panel'},texts);
  expect(keysOf(panel).sort()).toEqual(['current','pct','target','title']);
  const ring=overlayTree({...defaultDesign(),layout:'ring'},texts);
  expect(keysOf(ring).sort()).toEqual(['amounts','pct','title']);
 });

 it('raiz carrega a classe do formato e o anel desenha o svg',()=>{
  const root=overlayTree({...defaultDesign(),layout:'ring'},texts);
  expect(root.c).toContain('bl-ov-layout-ring');
  expect(collect(root,s=>s.tag==='svg').length).toBe(1);
  expect(collect(root,s=>s.c==='bl-ov-ring-fill').length).toBe(1);
  const confetti=collect(root,s=>s.tag==='i');
  expect(confetti.length).toBe(10);
 });

 it('os textos entram nos nós certos',()=>{
  const bar=overlayTree(defaultDesign(),texts);
  const title=collect(bar,s=>s.c==='bl-ov-title')[0];
  expect(title.t).toBe('Meta do mês');
  const pct=collect(bar,s=>s.c==='bl-ov-pct')[0];
  expect(pct.t).toBe('62%');
 });
});

describe('estúdio do overlay: variáveis CSS',()=>{
 it('cita a fonte entre aspas e respeita alinhamento',()=>{
  const quoted=designVars({...defaultDesign(),font:'Press Start 2P'});
  expect(quoted['--bl-font']).toContain('"Press Start 2P"');
  expect(designVars({...defaultDesign(),align:'left'})['--bl-margin']).toBe('0 auto');
  expect(designVars({...defaultDesign(),align:'right'})['--bl-margin']).toBe('auto 0');
  expect(designVars({...defaultDesign(),align:'center'})['--bl-margin']).toBe('auto');
 });

 it('converte faixas em medidas inteiras',()=>{
  const d:OverlayDesign={...defaultDesign(),borderWidth:1.5,tracking:1.5,uppercase:true};
  const v=designVars(d);
  expect(v['--bl-bw']).toBe('1.5px');
  expect(v['--bl-ls']).toBe('1.5px');
  expect(v['--bl-transform']).toBe('uppercase');
 });
});
