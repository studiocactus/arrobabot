import {describe,expect,it,vi} from 'vitest';
import {ACCENTS,applyAccent} from './appearance';

const lum=(hex:string)=>{
 const rgb=[1,3,5].map(i=>parseInt(hex.slice(i,i+2),16)/255).map(v=>v<=.04045?v/12.92:((v+.055)/1.055)**2.4);
 return rgb[0]*.2126+rgb[1]*.7152+rgb[2]*.0722;
};

describe('amostras de cor de destaque',()=>{
 it('são cores válidas, únicas e legíveis sobre os dois temas',()=>{
  const values=ACCENTS.map(a=>a.value);
  expect(new Set(values).size).toBe(values.length);
  expect(ACCENTS.length).toBeGreaterThanOrEqual(6);
  const escuro=lum('#0b0f16');
  for(const a of ACCENTS){
   expect(a.value,`${a.name} precisa ser uma cor hexadecimal`).toMatch(/^#[0-9a-f]{6}$/i);
   expect(a.name.trim().length).toBeGreaterThan(0);
   const l=lum(a.value);
   const sobreBranco=1.05/(l+.05);
   const sobreEscuro=(l+.05)/(escuro+.05);
   expect(Math.min(sobreBranco,sobreEscuro),`${a.name} perde leitura em um dos temas`).toBeGreaterThanOrEqual(3);
  }
 });

 it('aplicar cada amostra define --accent e escolhe texto com contraste',()=>{
  const props:Record<string,string>={};
  const style={setProperty:(k:string,v:string)=>{props[k]=v},removeProperty:(k:string)=>{delete props[k]}};
  vi.stubGlobal('document',{documentElement:{style}});
  try{
   for(const a of ACCENTS){
    applyAccent(a.value);
    expect(props['--accent']).toBe(a.value);
    expect(['#000000','#ffffff']).toContain(props['--accent-text']);
   }
   applyAccent('');
   expect(props['--accent']).toBeUndefined();
   expect(props['--accent-text']).toBeUndefined();
  } finally {vi.unstubAllGlobals()}
 });
});
