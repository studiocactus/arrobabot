import {test,expect} from '@playwright/test';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {auditPage,fmt,mock} from './ux-helpers';

/**
 * Overlay servido pelo BotLive (mesmo HTML de src-tauri/overlay.html e
 * examples/goal-overlay.html): renderiza o design, a meta e a celebração
 * sem recarregar a fonte do OBS. O WebSocket é simulado para o teste não
 * depender do aplicativo desktop.
 */
type FakeWS={
 url:string;
 onopen:((e:unknown)=>void)|null;
 onmessage:((e:unknown)=>void)|null;
 onclose:((e:unknown)=>void)|null;
 onerror:((e:unknown)=>void)|null;
 send:(d:string)=>void;
 close:()=>void;
};

const overlayUrl=pathToFileURL(resolve('examples/goal-overlay.html')).href+'#token=chave-secreta&profile=p1';

test('estúdio do overlay: prévia e controles em janela estreita e ampla',async({page})=>{
 await mock(page);
 await page.goto('/');
 await page.getByRole('button',{name:'LivePix',exact:true}).click();
 await expect(page.getByRole('heading',{name:'Estúdio do overlay'})).toBeVisible();

 // trocar modelo e simular doação mexem na prévia na hora
 await page.getByRole('button',{name:/^Neon/}).click();
 await page.getByRole('button',{name:'Simular doação'}).click();
 await expect(page.locator('.bl-ov-pct')).not.toHaveText('62%');

 await page.setViewportSize({width:1100,height:800});
 await page.locator('.lp-studio').scrollIntoViewIfNeeded();
 await page.waitForTimeout(250);
 const narrow=await page.evaluate(auditPage);
 expect(fmt(narrow),`${narrow.length} achado(s) em 1100px`).toEqual([]);
 await page.screenshot({path:'artifacts/estudio-livepix-1100.png'});

 await page.setViewportSize({width:1680,height:1000});
 await page.waitForTimeout(250);
 const wide=await page.evaluate(auditPage);
 expect(fmt(wide),`${wide.length} achado(s) em 1680px`).toEqual([]);
 await page.screenshot({path:'artifacts/estudio-livepix-largo.png'});

 // nenhuma das larguras produz rolagem horizontal
 const overflow=await page.evaluate(()=>document.documentElement.scrollWidth-document.documentElement.clientWidth);
 expect(overflow).toBeLessThanOrEqual(1);
});

test('overlay desenha design, meta, perfil, celebração e formato anel',async({page})=>{
 await page.addInitScript(()=>{
  const w=window as unknown as {__made:FakeWS[];__sent:string[]};
  w.__made=[];w.__sent=[];
  const made=w.__made,sent=w.__sent;
  class Fake implements FakeWS{
   url:string;
   onopen:((e:unknown)=>void)|null=null;
   onmessage:((e:unknown)=>void)|null=null;
   onclose:((e:unknown)=>void)|null=null;
   onerror:((e:unknown)=>void)|null=null;
   constructor(u:string){this.url=u;made.push(this);setTimeout(()=>this.onopen&&this.onopen({}),0)}
   send(d:string){sent.push(d)}
   close(){this.onclose&&this.onclose({})}
  }
  (window as unknown as {WebSocket:unknown}).WebSocket=Fake;
 });
 await page.goto(overlayUrl);

 const ov=page.locator('.bl-ov');
 // sem meta: invisível (sem "flash" de barra zerada no OBS)
 await expect(ov).toHaveCount(1);
 await expect(ov).toHaveClass(/is-off/);

 // autenticação sai com a chave do fragmento, sem reenviar dados
 await page.waitForFunction(()=>(window as unknown as {__sent:string[]}).__sent.length>0);
 const sent=await page.evaluate(()=>(window as unknown as {__sent:string[]}).__sent);
 expect(JSON.parse(sent[0])).toEqual({token:'chave-secreta'});

 const emit=async(msg:{type:string;payload:Record<string,unknown>})=>{
  await page.evaluate(m=>{
   const f=(window as unknown as {__made:FakeWS[]}).__made[0];
   f.onmessage&&f.onmessage({data:JSON.stringify(m)});
  },msg);
 };

 // design chega primeiro e troca as variáveis CSS, mas não liga a barra
 await emit({type:'design',payload:{profileId:'p1',design:{
  template:'neon',layout:'bar',align:'left',valign:'bottom',width:55,
  fillStart:'#ff2fd6',fillEnd:'#00e5ff',panel:'#0a0a12e6',uppercase:true,font:'Arial',fontSize:17
 }}});
 const fillA=await ov.evaluate(el=>el.style.getPropertyValue('--bl-fill-a'));
 expect(fillA).toBe('#ff2fd6');
 await expect(ov).toHaveClass(/is-off/);

 // meta visível: textos, percentual e largura da barra
 await emit({type:'goal',payload:{profileId:'p1',title:'Meta de teste',current:31000,target:50000,celebrate:false,hidden:false}});
 await expect(ov).not.toHaveClass(/is-off/);
 await expect(page.locator('.bl-ov-pct')).toHaveText('62%');
 await expect(page.locator('.bl-ov-title')).toHaveText('Meta de teste');
 await page.waitForTimeout(900); // transição de largura
 const ratio=await page.evaluate(()=>{
  const f=document.querySelector('.bl-ov-fill') as HTMLElement;
  const t=f.parentElement as HTMLElement;
  return f.getBoundingClientRect().width/t.getBoundingClientRect().width;
 });
 expect(ratio).toBeGreaterThan(0.58);
 expect(ratio).toBeLessThan(0.66);

 // meta de outro perfil é ignorada (a chave só enxerga o próprio overlay)
 await emit({type:'goal',payload:{profileId:'OUTRO',title:'Não pode aparecer',current:1,target:2,hidden:false}});
 await expect(page.locator('.bl-ov-title')).toHaveText('Meta de teste');

 // celebração ao cruzar 100%
 await emit({type:'goal',payload:{profileId:'p1',title:'Meta de teste',current:50000,target:50000,celebrate:true,hidden:false}});
 await expect(ov).toHaveClass(/is-celebrate/);
 await expect(page.locator('.bl-ov-pct')).toHaveText('100%');
 await page.screenshot({path:'artifacts/overlay-01-animacao.png'});

 // desligar a meta esconde na hora
 await emit({type:'goal',payload:{profileId:'p1',hidden:true}});
 await expect(ov).toHaveClass(/is-off/);

 // trocar para o formato anel redesenha sem recarregar a fonte
 await emit({type:'design',payload:{profileId:'p1',design:{layout:'ring'}}});
 await expect(ov).toHaveClass(/bl-ov-layout-ring/);
 await expect(page.locator('.bl-ov-ring-fill')).toHaveCount(1);
 const dash=await page.locator('.bl-ov-ring-fill').evaluate(el=>getComputedStyle(el).strokeDasharray);
 expect(dash).not.toBe('none');
 expect(dash).not.toBe('');
 await page.screenshot({path:'artifacts/overlay-02-anel.png'});
});
