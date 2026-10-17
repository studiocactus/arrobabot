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
     if(op==='spotify.get')return {config:st.getConfig,connected:st.connected,now:null};
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
 await expect(page.getByText('Salvo',{exact:true})).toBeVisible();
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
