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
 await page.getByRole('button',{name:'Meta',exact:true}).click();
 await expect(page.getByRole('heading',{name:'Estúdio do overlay'})).toBeVisible();

 // O endereço da camada sai do "Gerando…" porque as ops do livepix recebem
 // profileId (regressão do bug travado na 0.7.0 — mock falha sem o id).
 const address=page.locator('code').filter({hasText:/^http/});
 await expect(address).toBeVisible();
 await expect(address).toHaveText(/mode=goal$/);
 await expect(page.getByText('Gerando o endereço do overlay…')).toHaveCount(0);

 // trocar modelo e simular doação mexem na prévia na hora
 await page.getByRole('button',{name:/^Neon/}).click();
 await page.getByRole('button',{name:'Simular doação'}).click();
 await expect(page.locator('.bl-ov-pct')).not.toHaveText('62%');

 // Sons da Meta: escolher da biblioteca no grupo Efeitos e trocar de modelo
 // não pode perder a escolha (0.7.1).
 await page.getByLabel('Som de doação').scrollIntoViewIfNeeded();
 await page.getByLabel('Som de doação').selectOption('snd1');
 await expect(page.getByLabel('Som de doação')).toHaveValue('snd1');
 await expect(page.getByLabel('Som de celebração')).toHaveValue('');
 await page.getByRole('button',{name:/^Clean/}).click();
 await expect(page.getByLabel('Som de doação')).toHaveValue('snd1');
 // Espera o autosave assentar e volta a olhar o grupo Efeitos para o registro.
 await expect(page.locator('.lp-save')).toHaveText('Salvo');
 await page.getByLabel('Som de doação').scrollIntoViewIfNeeded();
 await page.waitForTimeout(250);
 await page.screenshot({path:'artifacts/estudio-livepix-sons.png'});

 await page.setViewportSize({width:1100,height:800});
 await page.locator('.lp-studio').scrollIntoViewIfNeeded();
 await page.waitForTimeout(250);
 const narrow=await page.evaluate(auditPage);
 expect(fmt(narrow),`${narrow.length} achado(s) em 1100px`).toEqual([]);
 // Padrão Estúdio: a prévia fica fixa enquanto os ajustes rolam ao lado.
 await expect(page.locator('.lp-preview')).toHaveCSS('position','sticky');
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

test('camada de chat no OBS: cor da Twitch, evento inline e modo código com escopo',async({page})=>{
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
 await page.goto(overlayUrl+'&mode=chat');
 // o motor só despacha depois do handshake (mesma espera dos testes vizinhos)
 await page.waitForFunction(()=>(window as unknown as {__sent:string[]}).__sent.length>0);

 const emit=async(msg:{type:string;payload:Record<string,unknown>})=>{
  await page.evaluate(m=>{
   const f=(window as unknown as {__made:FakeWS[]}).__made[0];
   f.onmessage&&f.onmessage({data:JSON.stringify(m)});
  },msg);
 };

 // Coluna ligada com eventos e CSS do modo código.
 const config={enabled:true,eventsOn:true,code:{html:'',css:'.bl-chat-msg{background:#001122}',js:''}};
 await emit({type:'chat.config',payload:{profileId:'p1',config}});

 // Mensagem real: prefixo antes das badges e a cor real da Twitch no nome.
 await emit({type:'chat',payload:{profileId:'p1',id:'m1',user:'Ana',userId:'ana',role:'everyone',text:'oi tudo bem',
  badges:['moderator'],frags:[{t:'t',v:'oi tudo bem'}],first:false,bot:false,color:'#9146ff'}});
 const col=page.locator('.bl-chat-col');
 await expect(col).toContainText('Ana');
 await expect(col.locator('.bl-chat-prefix')).toHaveText(/^>/);
 await expect(col.locator('.bl-chat-name')).toHaveCSS('color','rgb(145, 70, 255)');
 await expect(col.locator('.bl-chat-msg').first()).toHaveCSS('background-color','rgb(0, 17, 34)');
 // No overlay o escopo do CSS é o body inteiro (mesma regra com prefixo da prévia).
 await expect.poll(async()=>(await page.locator('#bl-code-chat').textContent())||'')
  .toContain('body .bl-chat-msg');

 // Evento inline: caixa com a cor própria do evento, mesma coluna.
 await emit({type:'chat.event',payload:{profileId:'p1',kind:'raid',text:'RAID / ana / 9 VIEWERS',color:'#ff8a3d'}});
 const box=col.locator('.bl-chat-ev');
 await expect(box).toHaveAttribute('data-ev','raid');
 await expect(box).toContainText('RAID / ana / 9 VIEWERS');
 expect(await box.evaluate(el=>el.style.getPropertyValue('--ec'))).toBe('#ff8a3d');
 // Base escura sob a cor: o texto branco da caixa lê em qualquer cena.
 await expect.poll(()=>box.locator('.bl-evbox').evaluate(el=>getComputedStyle(el).backgroundImage))
  .toContain('rgba(0, 0, 0, 0.6)');
 // Captura depois da animação de entrada, para o registro sair assentado.
 await page.waitForTimeout(600);
 await page.screenshot({path:'artifacts/overlay-chat-camada.png'});

 // Tipo fora do catálogo não desenha nada, mesmo com o mestre ligado.
 await emit({type:'chat.event',payload:{profileId:'p1',kind:'tip',text:'não pode aparecer',color:'#ffffff'}});
 await expect(col.locator('.bl-chat-ev')).toHaveCount(1);
});

test('explosão de emotes: cada camada mostra só a sua e o chat entra quando liberado',async({page})=>{
 // A CDN da Twitch responde com um pixel: o teste não depende da rede.
 const png=Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==','base64');
 await page.route('**/emoticons/**',route=>route.fulfill({status:200,contentType:'image/png',body:png}));
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
 const emit=async(msg:{type:string;payload:Record<string,unknown>})=>{
  await page.evaluate(m=>{
   const f=(window as unknown as {__made:FakeWS[]}).__made[0];
   f.onmessage&&f.onmessage({data:JSON.stringify(m)});
  },msg);
 };
 const handshake=()=>page.waitForFunction(()=>(window as unknown as {__sent:string[]}).__sent.length>0);
 const fx=()=>page.locator('.bl-eb');
 // Trocar só o fragmento não recarrega o documento (e o modo antigo ficaria);
 // o reload força a página nova com o mode pedido.
 const goto=async(url:string)=>{await page.goto(url);await page.reload()};

 // Camada de alertas: explosão de sub vira uma imagem por id, na CDN da Twitch.
 await goto(overlayUrl+'&mode=alerts');
 await handshake();
 await emit({type:'emote.burst',payload:{profileId:'p1',ids:['25','88','25'],size:48,via:'sub',alsoChat:false}});
 await expect(fx().locator('img')).toHaveCount(3);
 await expect(fx().locator('img').first()).toHaveAttribute('src',/static-cdn\.jtvnw\.net\/emoticons\/v2\/25\//);
 await expect(fx().locator('img').first()).toHaveAttribute('width','48');
 // A explosão de outro perfil não entra nesta fonte.
 await emit({type:'emote.burst',payload:{profileId:'OUTRO',ids:['9','9','9','9'],size:48,via:'sub',alsoChat:true}});
 await expect(fx().locator('img')).toHaveCount(3);

 // Camada de meta: só a meta completada explode aqui.
 await goto(overlayUrl+'&mode=goal');
 await handshake();
 await emit({type:'emote.burst',payload:{profileId:'p1',ids:['25'],size:48,via:'sub',alsoChat:false}});
 await expect(fx()).toHaveCount(0);
 await emit({type:'emote.burst',payload:{profileId:'p1',ids:['25','88'],size:48,via:'meta',alsoChat:false}});
 await expect(fx().locator('img')).toHaveCount(2);

 // Camada de chat: explode só quando a tela de chat libera (alsoChat).
 await goto(overlayUrl+'&mode=chat');
 await handshake();
 await emit({type:'emote.burst',payload:{profileId:'p1',ids:['25'],size:48,via:'sub',alsoChat:false}});
 await expect(fx()).toHaveCount(0);
 await emit({type:'emote.burst',payload:{profileId:'p1',ids:['25'],size:48,via:'meta',alsoChat:true}});
 await expect(fx().locator('img')).toHaveCount(1);

 // Registro final: a explosão caindo sobre o painel da meta, com o design da tela.
 await goto(overlayUrl);
 await handshake();
 await emit({type:'design',payload:{profileId:'p1',design:{
  template:'neon',layout:'bar',align:'left',valign:'bottom',width:55,
  fillStart:'#ff2fd6',fillEnd:'#00e5ff',panel:'#0a0a12e6',uppercase:true,font:'Arial',fontSize:17
 }}});
 await emit({type:'goal',payload:{profileId:'p1',title:'Meta de teste',current:31000,target:50000,celebrate:false,hidden:false}});
 await emit({type:'emote.burst',payload:{profileId:'p1',ids:['25','88','11483664','25','88','25','11483664','88','25','88','25','88'],size:56,via:'meta',alsoChat:false}});
 await expect(fx().locator('img')).toHaveCount(12);
 await page.waitForTimeout(700);
 await page.screenshot({path:'artifacts/overlay-explosao-emotes.png'});
});
