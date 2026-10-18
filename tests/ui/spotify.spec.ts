import {test,expect,type Page} from '@playwright/test';

/** Tela Spotify da live 0.7.9: conexão com o app próprio do streamer, leitura
 * da música atual e as variáveis que a automação recebe — sem rede real. */

type Now={id:string;name:string;artist:string;album:string;art:string;url:string;duration_ms:number;progress_ms:number;timestamp:number;playing:boolean};
type St={ops:string[];writes:{op:string;args:Record<string,unknown>}[];connected:boolean;track:unknown;getConfig:unknown;failConnect:string};
const profile={id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{}};

async function mock(page:Page){
 await page.addInitScript(arg=>{
  const st:St={ops:[],writes:[],connected:false,track:null,getConfig:{},failConnect:''};
  const w=window as unknown as Record<string,unknown>;
  w.__st=st;
  Object.assign(w,{
   isTauri:true,
   __TAURI_EVENT_PLUGIN_INTERNALS__:{registerListener:()=>{},unregisterListener:()=>{}},
   __TAURI_INTERNALS__:{
    transformCallback:()=>1,
    unregisterCallback:()=>{},
    invoke:async(cmd:string,payload:{op:string;args?:Record<string,unknown>}|null)=>{
     if(cmd!=='rpc')return 1;
     const op=payload?.op||'';const args=(payload?.args||{}) as Record<string,unknown>;
     st.ops.push(op);
     if(op==='snapshot')return {profiles:[arg.profile],logs:[],statuses:{},theme:'dark',apiPort:0,dataDir:'teste'};
     if(op==='flows'||op==='logs'||op==='command.counters')return [];
     if(op==='spotify.get')return {config:st.getConfig,connected:st.connected,now:null,overlay:{token:'tk1',port:9876,url:'http://127.0.0.1:9876/overlay#token=tk1&profile=p1'}};
     if(op==='module.config'){st.writes.push({op,args});return null}
     if(op==='spotify.connect'){if(st.failConnect)throw Error(st.failConnect);st.connected=true;return {authorized:true,user:'Cactus',product:'premium'}};
     if(op==='spotify.disconnect'){st.connected=false;st.track=null;return null};
     if(op==='spotify.poll')return {track:st.track};
     return null;
    }
   }
  });
 },{profile});
 await page.goto('/');
}
const st=async(page:Page)=>page.evaluate(()=>{
 const s=(window as unknown as {__st:St}).__st;
 return {ops:s.ops,writes:s.writes,connected:s.connected};
});
const saves=async(page:Page)=>(await st(page)).writes.filter(w=>w.op==='module.config').length;

test('conexão, leitura da música e variáveis — autosave inclui o design completo',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await mock(page);
 await page.getByRole('button',{name:'Spotify',exact:true}).click();
 await expect(page.locator('.nav-item.active')).toContainText('Spotify');

 // cartões de conexão, música e variáveis
 await expect(page.getByRole('heading',{name:'Conexão com o Spotify',exact:true})).toBeVisible();
 await expect(page.getByRole('heading',{name:'Música no overlay',exact:true})).toBeVisible();
 await expect(page.getByRole('heading',{name:'Nas automações',exact:true})).toBeVisible();
 await expect(page.getByText('Não autorizado',{exact:true})).toBeVisible();

 // sem Client ID não há autorização — e a URL de retorno sai com um clique
 const id=page.getByLabel('Client ID do seu aplicativo');
 await expect(page.getByRole('button',{name:'Autorizar conta',exact:true})).toBeDisabled();
 await expect(page.locator('.lp-copyrow input')).toHaveValue('http://127.0.0.1:43828/callback');

 // escrever o Client ID salva sozinho, com o design completo (validação do backend)
 await id.fill('app-id-123');
 await expect.poll(()=>saves(page),{timeout:3000}).toBe(1);
 await expect(page.getByText('Salvo',{exact:true}).first()).toBeVisible();
 const saved=(await st(page)).writes.at(-1)!.args.value as {clientId:string;enabled:boolean;design:{layout:string;barColor:string}};
 expect(saved.clientId).toBe('app-id-123');
 expect(saved.design.layout,'o design completo vai junto no salvamento').toBe('card');
 expect(saved.design.barColor).toBe('#1ed760');

 // autorizar: mock devolve a conta e a tela mostra quem conectou
 await page.getByRole('button',{name:'Autorizar conta',exact:true}).click();
 await expect(page.getByText('Conectado — Cactus',{exact:false})).toBeVisible();

 // testar a leitura: a faixa (snake_case, como o backend envia) aparece na linha
 await page.evaluate(()=>{(window as unknown as {__st:St}).__st.track={
  id:'t1',name:'Tocando Agora',artist:'Cactus',album:'Ao Vivo',art:'',url:'',
  duration_ms:201000,progress_ms:12000,timestamp:0,playing:false}});
 await page.getByRole('button',{name:'Testar conexão',exact:true}).click();
 await expect(page.locator('.lp-nowplaying strong')).toContainText('Tocando Agora — Cactus (0:12/3:21) · pausado');

 // interruptor mestre salva como as demais peças
 await page.getByRole('switch',{name:'Ler a música que está tocando'}).click();
 await expect.poll(()=>saves(page),{timeout:3000}).toBe(2);
 expect(((await st(page)).writes.at(-1)!.args.value as {enabled:boolean}).enabled,'interruptor salvo').toBe(true);

 // variáveis: as seis aparecem prontas para copiar
 for(const key of ['nowplaying.musica','nowplaying.artista','nowplaying.album','nowplaying.tempo','nowplaying.duracao','nowplaying.link']){
  await expect(page.locator('.lp-varlist code').filter({hasText:`{{${key}}}`})).toBeVisible();
 }

 // estreito: sem rolagem horizontal
 await page.setViewportSize({width:1100,height:900});
 const overflow=await page.evaluate(()=>document.documentElement.scrollWidth>document.documentElement.clientWidth);
 expect(overflow,'sem rolagem horizontal a 1100px').toBe(false);
 await page.screenshot({path:'artifacts/spotify-1100.png',fullPage:true});
 await page.setViewportSize({width:1440,height:1000});
 await page.screenshot({path:'artifacts/spotify-wide.png',fullPage:true});

 // desconectar limpa a linha e a etiqueta
 await page.getByRole('button',{name:'Desconectar',exact:true}).click();
 await expect(page.getByText('Nada tocando agora.',{exact:true})).toBeVisible();
 await expect(page.getByText('Não autorizado',{exact:true})).toBeVisible();
 const s=await st(page);
 expect(s.ops,'fluxo completo de ops').toContain('spotify.connect');
 expect(s.ops).toContain('spotify.poll');
 expect(s.ops).toContain('spotify.disconnect');
 expect(errors).toEqual([]);
});

test('Client ID salvo volta ao abrir a tela e erro de autorização aparece sem quebrar',async({page})=>{
 await mock(page);
 await page.getByRole('button',{name:'Spotify',exact:true}).click();

 // o que ficou salvo volta preenchido — abrir a tela de novo é o carregamento real
 await page.evaluate(()=>{const s=(window as unknown as {__st:St}).__st;s.getConfig={enabled:true,clientId:'voltei-ao-abrir'};s.connected=true});
 await page.getByRole('button',{name:'Visão geral',exact:true}).click();
 await page.getByRole('button',{name:'Spotify',exact:true}).click();
 await expect(page.getByLabel('Client ID do seu aplicativo')).toHaveValue('voltei-ao-abrir');
 await expect(page.getByText('Conectado',{exact:false})).toBeVisible();

 // autorização recusada pelo backend: o aviso vem em português e a tela segue de pé
 await page.evaluate(()=>{(window as unknown as {__st:St}).__st.failConnect='O Spotify recusou a autorização. Confira o Client ID e a URL de retorno no aplicativo.'});
 await page.getByRole('button',{name:'Autorizar conta',exact:true}).click();
 await expect(page.getByText('O Spotify recusou a autorização',{exact:false})).toBeVisible();
 await expect(page.getByRole('heading',{name:'Conexão com o Spotify',exact:true})).toBeVisible();
});

test('Estúdio da música: prévia igual ao OBS, peles, cores e endereço mode=spotify',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.context().grantPermissions(['clipboard-write','clipboard-read']);
 await mock(page);
 await page.getByRole('button',{name:'Spotify',exact:true}).click();
 await expect(page.getByRole('heading',{name:'Estúdio da música',exact:true})).toBeVisible();
 await expect(page.getByRole('heading',{name:'No OBS',exact:true})).toBeVisible();

 // A prévia desenha a amostra com o mesmo módulo que a fonte do OBS usa.
 const card=page.locator('.lp-preview .bl-spy');
 await expect(card).toBeVisible();
 await expect(page.locator('.lp-preview .bl-spy-title')).toHaveText('Tocando Agora');
 await expect(page.locator('.lp-stagebar .lp-chip')).toHaveText('Amostra');

 // Pele troca o desenho na hora e o salvamento leva o design novo.
 await page.getByRole('button',{name:'Empilhado',exact:true}).click();
 await expect(card).toHaveAttribute('data-layout','stacked');
 await expect.poll(()=>saves(page),{timeout:3000}).toBe(1);
 const saved=(await st(page)).writes.at(-1)!.args.value as {design:{layout:string;position:string;cover:string;exitTransition:string;pauseDelay:number;hideOnPause:boolean}};
 expect(saved.design.layout).toBe('stacked');
 expect(saved.design.position,'as peças novas da Fase B vão junto no salvamento').toBe('bottom-left');
 expect(saved.design.cover).toBe('square');
 expect(saved.design.exitTransition).toBe('fade');
 expect(saved.design.pauseDelay).toBe(0);
 expect(saved.design.hideOnPause).toBe(true);

 // Posição, formato da capa, tema e destaque — cada controle mexe no cartão.
 await page.getByRole('button',{name:'Baixo à direita',exact:true}).click();
 await expect(page.locator('.lp-preview .bl-spy-slot')).toHaveAttribute('data-position','bottom-right');
 await page.getByRole('button',{name:'Redonda',exact:true}).click();
 await expect(card).toHaveAttribute('data-cover','circle');
 await page.getByRole('button',{name:'Claro',exact:true}).click();
 await expect.poll(()=>card.evaluate(el=>el.style.getPropertyValue('--sp-bg'))).toBe('#f4f6fa');
 await page.getByRole('button',{name:'Destaque #38b6ff',exact:true}).click();
 await expect.poll(()=>card.evaluate(el=>el.style.getPropertyValue('--sp-bar'))).toBe('#38b6ff');

 // Sem música: o estado parado que o OBS mostraria, na própria prévia.
 await page.getByRole('button',{name:'Sem música',exact:true}).click();
 await expect(card).toHaveAttribute('data-state','empty');
 await expect(page.locator('.lp-preview .bl-spy-title')).toHaveText('Nada tocando');
 await expect(page.locator('.lp-stagebar .lp-chip')).toHaveText('Parado');
 await page.getByRole('button',{name:'Com música',exact:true}).click();
 await expect(card).toHaveAttribute('data-state','music');

 // No OBS: endereço da camada própria e cópia com um clique.
 await expect(page.locator('.lp-url code')).toHaveText(/mode=spotify$/);
 await page.getByRole('button',{name:/Copiar endereço/}).click();
 await expect(page.getByText('Endereço da camada de música copiado.')).toBeVisible();

 // Padrão Estúdio: prévia grudada a 1100px, sem rolagem horizontal.
 await page.setViewportSize({width:1100,height:900});
 await page.locator('.lp-studio').scrollIntoViewIfNeeded();
 await expect(page.locator('.lp-preview')).toHaveCSS('position','sticky');
 const overflow=await page.evaluate(()=>document.documentElement.scrollWidth>document.documentElement.clientWidth);
 expect(overflow,'sem rolagem horizontal a 1100px').toBe(false);
 await page.screenshot({path:'artifacts/spotify-estudio-1100.png'});
 await page.setViewportSize({width:1680,height:1000});
 await page.waitForTimeout(250);
 await page.screenshot({path:'artifacts/spotify-estudio-largo.png'});
 expect(errors).toEqual([]);
});

test('zoom da prévia de 10% a 100% fora do design, com roda, teclado e persistência',async({page})=>{
 await mock(page);
 await page.getByRole('button',{name:'Spotify',exact:true}).click();
 await expect(page.getByRole('heading',{name:'Estúdio da música',exact:true})).toBeVisible();

 const pct=page.locator('.lp-zoom-pct');
 const zoomBox=page.locator('.lp-zoombar');
 await expect(zoomBox).toBeVisible();

 // Ajustar à área é o estado inicial e a faixa acompanha o tamanho real do quadro.
 await expect(page.getByRole('button',{name:'Ajustar à área',exact:true})).toHaveAttribute('aria-pressed','true');
 const fitPct=Number((await pct.textContent()).replace('%',''));
 expect(fitPct,'escala de ajuste é menor que 100%').toBeGreaterThan(0);
 expect(fitPct).toBeLessThan(100);

 // Teclado: a faixa responde às setas e amplia sem sair do limite.
 await page.getByRole('slider',{name:/Zoom da prévia/}).focus();
 await page.keyboard.press('ArrowRight');
 const afterKey=Number((await pct.textContent()).replace('%',''));
 expect(afterKey,'seta direita amplia um ponto').toBe(fitPct+1);
 await expect(page.getByRole('button',{name:'Ajustar à área',exact:true})).toHaveAttribute('aria-pressed','false');

 // 100% e limites dos botões: no teto o “+” some do alcance (fica desabilitado)
 // e o “−” desce o passo de 10 em 10 — previsível, sem salto.
 await page.getByRole('button',{name:'100%',exact:true}).click();
 await expect(pct).toHaveText('100%');
 await expect(page.getByRole('button',{name:'Aumentar zoom',exact:true}),'no teto o + desabilita').toBeDisabled();
 await page.getByRole('button',{name:'Diminuir zoom',exact:true}).click();
 await expect(pct,'botão − desce o passo de 10').toHaveText('90%');

 // Roda do mouse sobre a prévia amplia e a composição fica navegável.
 await page.locator('.lp-frame').hover();
 await page.mouse.wheel(0,240);
 await expect(pct,'roda para baixo afasta').not.toHaveText('90%');
 const zoomed=Number((await pct.textContent()).replace('%',''));
 expect(zoomed,'roda segue o limite de 10% a 100%').toBeGreaterThan(9);
 expect(zoomed).toBeLessThan(100);
 await expect(page.locator('.lp-frame'),'composição ampliada vira área rolável').toHaveClass(/zoomable/);
 await expect(page.getByRole('button',{name:'Ajustar à área',exact:true}),'roda sai do ajuste automático').toHaveAttribute('aria-pressed','false');

 // O zoom nunca vaza para o design salvo (o OBS recebe o quadro inteiro sempre).
 const saved=(await st(page)).writes.filter(w=>w.op==='module.config').at(-1);
 if(saved){
  const design=(saved.args.value as {design:Record<string,unknown>}).design;
  expect('zoom' in design,'zoom não é campo do design').toBe(false);
 }
 const view=await page.evaluate(()=>localStorage.getItem('botlive.spotify.view'));
 expect(view,'preferência de prévia fica fora do overlay').toBeTruthy();
 expect(view).toContain('"fit":false');
});

test('presets do mínimo ao completo e seções do Estúdio',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await mock(page);
 await page.getByRole('button',{name:'Spotify',exact:true}).click();
 await expect(page.getByRole('heading',{name:'Modelo do cartão',exact:true})).toBeVisible();
 for(const s of ['Conteúdo','Layout','Aparência','Comportamento','Ajustes avançados'])
  await expect(page.getByRole('heading',{name:s,exact:true})).toBeVisible();

 const card=page.locator('.lp-preview .bl-spy');

 // “Só a música”: só o título, sem capa, barra ou tempos — e o chip nomeia o modelo.
 // (o nome acessível do botão junta o título com a descrição; casa por trecho)
 await page.getByRole('button',{name:/Só a música/}).click();
 await expect(page.locator('.lp-model-chip')).toHaveText('Só a música');
 await expect(card).toHaveAttribute('data-layout','minimal');
 await expect(card).toHaveAttribute('data-showart','off');
 await expect(card).toHaveAttribute('data-progress','off');
 await expect(card).toHaveAttribute('data-times','off');
 await expect(card).toHaveAttribute('data-title','on');

 // “Completo” devolve capa, álbum, barra e tempos de uma vez.
 await page.getByRole('button',{name:/Completo/}).click();
 await expect(page.locator('.lp-model-chip')).toHaveText('Completo');
 await expect(card).toHaveAttribute('data-showart','on');
 await expect(card).toHaveAttribute('data-album','on');
 await expect(card).toHaveAttribute('data-progress','on');
 await expect(card).toHaveAttribute('data-times','on');

 // Alavanca independente: desligar o artista some a linha e vira “Personalizado”.
 await page.getByRole('checkbox',{name:'Artista',exact:true}).uncheck();
 await expect(card).toHaveAttribute('data-artist','off');
 await expect(page.locator('.lp-model-chip')).toHaveText('Personalizado');

 // Ajustes avançados começam recolhidos (nada montado) e abrem no comando.
 await expect(page.getByRole('switch',{name:'Capa desfocada ao fundo',exact:true})).toHaveCount(0);
 await page.getByRole('button',{name:'Abrir',exact:true}).click();
 await expect(page.getByRole('switch',{name:'Capa desfocada ao fundo',exact:true})).toBeVisible();

 expect(errors).toEqual([]);
});
