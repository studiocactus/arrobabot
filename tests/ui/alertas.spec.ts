import {test,expect,type Page} from '@playwright/test';
import {auditPage,fmt} from './ux-helpers';

const profile={id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{}};

/** Mock desktop: `alerts.get` devolve sons e endereço; `module.config` grava em session. */
async function setup(page:Page){
 await page.addInitScript(p=>{
  Object.assign(window,{isTauri:true,
   __TAURI_EVENT_PLUGIN_INTERNALS__:{registerListener:()=>{},unregisterListener:()=>{}},
   __TAURI_INTERNALS__:{transformCallback:()=>1,unregisterCallback:()=>{},
    invoke:async(cmd:string,payload:{op:string;args?:Record<string,unknown>})=>{
     if(cmd!=='rpc')return 1;
     const op=payload.op;const args=payload.args||{};
     if(op==='snapshot')return {profiles:[p],flows:[],notes:{},presets:[],theme:'dark',logs:[],statuses:{},apiPort:9876,dataDir:'teste'};
     if(op==='flows')return [];
     if(op==='command.counters')return {};
     if(op==='settings.get')return {theme:'dark',apiPort:9876,apiToken:'',updateEndpoint:'',updatePublicKey:'',autoUpdate:false};
     if(op==='alerts.get')return {config:null,sounds:[{id:'s1',name:'jingle.wav'}],overlay:{token:'tok',port:9876,url:'http://127.0.0.1:9876/overlay#token=tok&profile=p1'}};
     if(op==='alerts.test'){
      sessionStorage.setItem('alertsTest',String(args.kind));
      const kind=String(args.kind);
      // Mesmos interruptores do backend: com algum desligado, volta o motivo.
      const cfg=JSON.parse(sessionStorage.getItem('cfg:alerts')||'null');
      if(cfg?.enabled!==true)return {fired:false,reason:'Ligue os alertas no overlay: em live o alerta não tocaria.'};
      if(cfg?.events?.[kind]?.on!==true){
       const labels:Record<string,string>={follow:'seguidor',subscription:'novo sub',resub:'renovação',gift:'sub de presente',giftbulk:'presentes em lote',cheer:'bits',raid:'raid'};
       return {fired:false,reason:`Ligue o alerta de ${labels[kind]||kind} no overlay: em live ele não tocaria.`};
      }
      // Vars fiéis à tabela do backend (rehearsal_event + vars()).
      const samples:Record<string,Record<string,unknown>>={
       follow:{name:'NovoSeguidor',amount:1,months:1,tier:'TIER 1',message:''},
       subscription:{name:'UsuarioSub',amount:1,months:1,tier:'TIER 1',message:''},
       resub:{name:'UsuarioFiel',amount:1,months:8,tier:'TIER 1',message:'Melhor stream!'},
       gift:{name:'Presenteado123',amount:1,months:1,tier:'TIER 1',message:''},
       giftbulk:{name:'Generoso123',amount:10,months:1,tier:'TIER 1',message:''},
       cheer:{name:'AnaBits',amount:500,months:1,tier:'TIER 1',message:'GG, que jogo!'},
       raid:{name:'RaidMan',amount:37,months:1,tier:'TIER 1',message:''},
      };
      return {fired:true,kind,vars:samples[kind]||samples.follow};
     }
     if(op==='emoteBurst.test')throw new Error('Sem emotes carregados: conecte o canal ao Twitch e tente de novo.');
     if(op==='module.config.get'){
      const raw=sessionStorage.getItem('cfg:'+String(args.key));
      return raw?JSON.parse(raw):null;
     }
     if(op==='module.config'){
      const k=String(args.key);
      sessionStorage.setItem('cfg:'+k,JSON.stringify(args.value));
      sessionStorage.setItem('alertsSave',JSON.stringify({key:k,value:args.value}));
      return null;
     }
     return null;
    }}});
  sessionStorage.removeItem('alertsSave');
  sessionStorage.removeItem('alertsTest');
 },profile);
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Alertas',exact:true}).click();
}

const saved=async(page:Page)=>JSON.parse(await page.evaluate(()=>sessionStorage.getItem('alertsSave')||'null'));

test('estúdio lista os sete eventos, prévia e salvamento automático por evento',async({page})=>{
 await setup(page);

 await expect(page.getByRole('heading',{name:'Alertas da live'})).toBeVisible();
 const pills=page.locator('.al-pills');
 for(const label of ['Seguidor','Novo sub','Renovação','Sub de presente','Presentes em lote','Bits','Raid']){
  await expect(pills.getByRole('button',{name:label,exact:true})).toBeVisible();
 }

 // Prévia parada com o padrão do evento selecionado.
 await expect(page.locator('.lp-frame')).toContainText('BEM-VINDO!');
 await expect(page.locator('.lp-frame')).toContainText('SEGUIDOR');
 // Tarja padrão na cor global do painel, com texto escuro legível por luminância.
 await expect(page.locator('.bl-alert.is-static .al-tag')).toHaveCSS('background-color','rgb(198, 255, 0)');
 await expect(page.locator('.bl-alert.is-static .al-tag')).toHaveCSS('color','rgb(15, 15, 15)');

 // Trocar de evento muda a prévia e o formulário.
 await pills.getByRole('button',{name:'Bits',exact:true}).click();
 await expect(page.locator('.lp-frame')).toContainText('500 BITS');
 await expect(page.getByLabel('Linha grande',{exact:true})).toHaveValue('{{amount}} BITS');
 await expect(page.getByLabel('Etiqueta',{exact:true})).toHaveValue('BITS');

 // Editar texto grava a configuração inteira do perfil com o valor novo.
 await page.getByLabel('Linha grande',{exact:true}).fill('{{amount}} BITS DO CORAÇÃO');
 await expect.poll(async()=>(await saved(page))?.value?.events?.cheer?.big,{timeout:3000}).toBe('{{amount}} BITS DO CORAÇÃO');

 // Som do evento vem da lista de sons do perfil.
 await expect(page.getByLabel('Som do evento').locator('option[value="s1"]')).toHaveText('jingle.wav');

 // Interruptor mestre também salva.
 const master=page.getByRole('switch',{name:'Ligar alertas no overlay'});
 await expect(master).toHaveAttribute('aria-checked','false');
 await master.click();
 await expect.poll(async()=>(await saved(page))?.value?.enabled,{timeout:3000}).toBe(true);
});

test('botão de teste anima a prévia pelo caminho do evento real e termina limpo',async({page})=>{
 await setup(page);
 await pills_select_bits(page);

 // A chave mestre liga (e salva) antes do disparo — é o que a live exige.
 const master=page.getByRole('switch',{name:'Ligar alertas no overlay'});
 await master.click();
 await expect.poll(async()=>(await saved(page))?.value?.enabled,{timeout:3000}).toBe(true);

 await page.getByRole('button',{name:/Testar bits/}).click();
 await expect.poll(async()=>await page.evaluate(()=>sessionStorage.getItem('alertsTest')||''),{timeout:3000}).toBe('cheer');

 // Selo ENSAIO na prévia durante o disparo…
 await expect(page.locator('.lp-ensaio')).toBeVisible();
 // …e o painel animado (não o estático) mostra os dados de exemplo e some sozinho.
 await expect(page.locator('.bl-alert:not(.is-static)')).toContainText('AnaBits',{timeout:5000});
 await expect(page.locator('.bl-alert:not(.is-static)')).toContainText('500 BITS',{timeout:5000});
 await expect(page.locator('.bl-alert:not(.is-static)')).toHaveCount(0,{timeout:25000});
 // O selo some sozinho e a prévia volta pronta para a próxima edição.
 await expect(page.locator('.lp-ensaio')).toHaveCount(0,{timeout:8000});
 await expect(page.locator('.bl-alert.is-static')).toContainText('500 BITS');
});

test('avisa qual interruptor está desligado em vez de disparar pela metade',async({page})=>{
 await setup(page);
 await pills_select_bits(page);

 // Sem nada ligado: o motivo aponta a chave mestre e nada anima.
 await page.getByRole('button',{name:/Testar bits/}).click();
 await expect(page.locator('.toast')).toContainText('Ligue os alertas no overlay');
 await expect(page.locator('.bl-alert:not(.is-static)')).toHaveCount(0);
 await expect(page.locator('.lp-ensaio')).toHaveCount(0);

 // Mestre ligada, mas o evento de bits desligado: o motivo muda para o evento.
 const master=page.getByRole('switch',{name:'Ligar alertas no overlay'});
 await master.click();
 await expect.poll(async()=>(await saved(page))?.value?.enabled,{timeout:3000}).toBe(true);
 await page.getByRole('switch',{name:'Exibir bits'}).click();
 await expect.poll(async()=>(await saved(page))?.value?.events?.cheer?.on,{timeout:3000}).toBe(false);

 await page.getByRole('button',{name:/Testar bits/}).click();
 await expect(page.locator('.toast')).toContainText('Ligue o alerta de bits no overlay');
 await expect(page.locator('.bl-alert:not(.is-static)')).toHaveCount(0);
 await expect(page.locator('.lp-ensaio')).toHaveCount(0);
});

test('estúdio de alertas cabe em janela de 1100px e ampla sem espremer texto',async({page})=>{
 await page.setViewportSize({width:1100,height:950});
 await setup(page);
 await expect(page.locator('.bl-alert.is-static')).toBeVisible();
 // Padrão Estúdio: a prévia fica fixa enquanto os ajustes rolam ao lado.
 await expect(page.locator('.lp-preview')).toHaveCSS('position','sticky');

 // Dispara um teste ligado (a mestre é o que a live exige) para o selo ENSAIO
 // constar da captura estreita — espera o quadro assentar (entrada, glitch e
 // tremor saem nos ~2,2s da coreografia) dentro da janela dos 5s do selo.
 const master=page.getByRole('switch',{name:'Ligar alertas no overlay'});
 await master.click();
 await expect.poll(async()=>(await saved(page))?.value?.enabled,{timeout:3000}).toBe(true);
 await page.getByRole('button',{name:/Testar seguidor/}).click();
 await expect(page.locator('.lp-ensaio')).toBeVisible();
 await page.waitForTimeout(2400);

 const narrow=await page.evaluate(auditPage);
 expect(fmt(narrow),`${narrow.length} achado(s) em 1100px`).toEqual([]);
 await page.screenshot({path:'artifacts/estudio-alertas-1100.png'});

 // A larga captura o layout já ajustado e outro disparo para o selo constar.
 await page.setViewportSize({width:1680,height:1000});
 await page.waitForTimeout(250);
 const wide=await page.evaluate(auditPage);
 expect(fmt(wide),`${wide.length} achado(s) em 1680px`).toEqual([]);
 await expect(page.getByRole('button',{name:/Testar seguidor/})).toBeEnabled({timeout:20000});
 await page.getByRole('button',{name:/Testar seguidor/}).click();
 await expect(page.locator('.lp-ensaio')).toBeVisible();
 await page.waitForTimeout(2400);
 await expect(page.locator('.lp-ensaio')).toBeVisible();
 await page.screenshot({path:'artifacts/estudio-alertas-largo.png'});

 // Nenhuma largura produz rolagem horizontal.
 const overflow=await page.evaluate(()=>document.documentElement.scrollWidth-document.documentElement.clientWidth);
 expect(overflow).toBeLessThanOrEqual(1);
});

test('configurações ficam em acordeão ao lado da prévia, no padrão do estúdio',async({page})=>{
 await setup(page);

 // Prévia à esquerda, configurações à direita (ordem do DOM).
 const previewFirst=await page.evaluate(()=>
  !!(document.querySelector('.lp-preview')?.compareDocumentPosition(document.querySelector('.lp-controls'))&Node.DOCUMENT_POSITION_FOLLOWING));
 expect(previewFirst).toBe(true);

 // "Este evento" aberto por padrão; os demais grupos fechados economizam altura.
 await expect(page.getByLabel('Etiqueta',{exact:true})).toBeVisible();
 const body=page.locator('#item-body-al-panel');
 await expect(body).toHaveAttribute('inert');
 await page.locator('.item-toggle').filter({hasText:'Painel'}).click();
 await expect(body).not.toHaveAttribute('inert');
 await expect(page.getByLabel('Duração',{exact:true})).toBeVisible();
 // Registro do layout: abas do modo código acima do acordeão, grupo aberto.
 await body.scrollIntoViewIfNeeded();
 await page.locator('.cd-tabbar').scrollIntoViewIfNeeded();
 await page.screenshot({path:'artifacts/estudio-alertas-acordeao.png'});
 // Abrir não mexe no interruptor mestre dos alertas.
 await expect(page.getByRole('switch',{name:'Ligar alertas no overlay'})).toHaveAttribute('aria-checked','false');
});

test('cor por evento pinta só a tarja e o botão devolve a cor global',async({page})=>{
 await setup(page);
 await page.getByLabel('Cor da tarja').fill('#ff3b30');

 await expect.poll(async()=>(await saved(page))?.value?.events?.follow?.color,{timeout:3000}).toBe('#ff3b30');
 await expect(page.locator('.bl-alert.is-static .al-tag')).toHaveCSS('background-color','rgb(255, 59, 48)');
 // Contraste: fundo escuro vira texto branco (não o preto fixo antigo).
 await expect(page.locator('.bl-alert.is-static .al-tag')).toHaveCSS('color','rgb(255, 255, 255)');
 // As outras linhas continuam na cor global do painel.
 await expect(page.locator('.bl-alert.is-static .al-name')).toHaveCSS('color','rgb(198, 255, 0)');

 await page.getByRole('button',{name:'Usar cor global'}).click();
 await expect.poll(async()=>(await saved(page))?.value?.events?.follow?.color,{timeout:3000}).toBe('');
});

test('modo código dos alertas aplica CSS no palco e a aba Data recusa o que não é objeto',async({page})=>{
 await setup(page);

 // CSS prefixado com o escopo do palco (não pinta o app) e efeito imediato.
 await page.getByRole('tab',{name:'CSS'}).click();
 await page.locator('.cd-panel .cd-code').fill('.al-tag{word-spacing:7px}');
 await expect.poll(async()=>(await page.locator('#bl-code-alerts').textContent())||'')
  .toContain('.al-stage .al-tag');
 await expect(page.locator('.bl-alert.is-static .al-tag')).toHaveCSS('word-spacing','7px');

 // Restaurar padrão limpa o bloco e a prévia volta ao normal.
 await page.getByRole('button',{name:'Restaurar padrão'}).click();
 await expect(page.locator('#bl-code-alerts')).toHaveCount(0);
 await expect(page.locator('.bl-alert.is-static .al-tag')).toHaveCSS('word-spacing','0px');

 // Data: JSON inválido e não-objeto barrados com a mensagem certa.
 await page.getByRole('tab',{name:'Data'}).click();
 const data=page.locator('.cd-panel .cd-code');
 await expect(data).toHaveValue(/"panelWidth"/);
 const original=await data.inputValue();
 await data.fill('oi');
 await page.getByRole('button',{name:'Aplicar'}).click();
 await expect(page.getByRole('alert')).toContainText('JSON inválido');
 await data.fill('42');
 await page.getByRole('button',{name:'Aplicar'}).click();
 await expect(page.getByRole('alert')).toContainText('Esperado um objeto');

 // Objeto válido normaliza pelo formulário e devolve à aba Fields.
 const obj=JSON.parse(original) as Record<string,unknown>;
 obj.duration=6;
 await data.fill(JSON.stringify(obj));
 await page.getByRole('button',{name:'Aplicar'}).click();
 await expect.poll(async()=>(await saved(page))?.value?.duration,{timeout:3000}).toBe(6);
 await expect(page.getByLabel('Etiqueta',{exact:true})).toBeVisible();
});

async function pills_select_bits(page:Page){
 await page.locator('.al-pills').getByRole('button',{name:'Bits',exact:true}).click();
}

test('explosão de emotes: interruptores por evento, origem, teste e persistência',async({page})=>{
 await setup(page);
 const value=async()=>(await saved(page))?.value;

 // A mestre nasce desligada; ligar salva a configuração com os padrões da tela.
 const master=page.getByRole('switch',{name:'Ligar explosão de emotes'});
 await expect(master).toHaveAttribute('aria-checked','false');
 await master.click();
 await expect.poll(async()=>(await saved(page))?.key,{timeout:3000}).toBe('emoteBurst');
 await expect.poll(value,{timeout:3000}).toMatchObject({count:40,size:56,source:'random',alsoChat:false,events:{sub:true,resub:true,gift:true,meta:true}});

 // Renovação sai; a origem vira os mais usados e a quantidade muda.
 await page.getByRole('switch',{name:'Renovação de sub'}).click();
 await page.getByRole('combobox',{name:'Emotes usados',exact:true}).selectOption('top');
 await page.getByLabel('Quantidade',{exact:true}).fill('60');
 await expect.poll(async()=>(await value())?.events?.resub,{timeout:3000}).toBe(false);
 await expect.poll(async()=>(await value())?.source).toBe('top');
 await expect.poll(async()=>(await value())?.count).toBe(60);

 // O teste avisa em vez de falhar em silêncio (sem emotes no mock, o núcleo recusa).
 await page.getByRole('button',{name:/Testar explosão/}).click();
 await expect(page.locator('.toast')).toContainText('Sem emotes carregados');

 // Registro do card nas duas larguras.
 const card=page.getByRole('heading',{name:'Explosão de emotes'}).locator('..');
 await card.scrollIntoViewIfNeeded();
 await page.waitForTimeout(250);
 await page.screenshot({path:'artifacts/alertas-explosao-1100.png'});
 await page.setViewportSize({width:1680,height:1000});
 await card.scrollIntoViewIfNeeded();
 await page.waitForTimeout(250);
 await page.screenshot({path:'artifacts/alertas-explosao-largo.png'});

 // Reabre: o que ficou guardado volta integral.
 await page.reload();
 await page.locator('.sidebar').getByRole('button',{name:'Alertas',exact:true}).click();
 await expect(page.getByRole('switch',{name:'Ligar explosão de emotes'})).toHaveAttribute('aria-checked','true');
 await expect(page.getByRole('switch',{name:'Renovação de sub'})).toHaveAttribute('aria-checked','false');
 await expect(page.getByRole('combobox',{name:'Emotes usados',exact:true})).toHaveValue('top');
 await expect(page.getByLabel('Quantidade',{exact:true})).toHaveValue('60');
});


test('falha do autosave mantém a edição e permite tentar salvar novamente antes de testar',async({page})=>{
 await setup(page);
 await page.evaluate(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(cmd:string,payload:{op:string;args?:Record<string,unknown>})=>Promise<unknown>}};
  const original=w.__TAURI_INTERNALS__.invoke;
  let fail=true;
  w.__TAURI_INTERNALS__.invoke=async(cmd,payload)=>{
   if(payload?.op==='module.config'&&payload.args?.key==='alerts'&&fail){fail=false;throw Error('Falha controlada ao salvar alerta')}
   return original(cmd,payload);
  };
 });
 await page.getByLabel('Linha grande',{exact:true}).fill('ALERTA EDITADO');
 await expect(page.getByRole('alert').filter({hasText:'Falha controlada ao salvar alerta'})).toBeVisible();
 await expect(page.getByLabel('Linha grande',{exact:true})).toHaveValue('ALERTA EDITADO');
 await expect(page.getByRole('button',{name:/Testar seguidor/})).toBeDisabled();
 await page.getByRole('button',{name:'Tentar salvar alertas novamente',exact:true}).click();
 await expect.poll(async()=>(await saved(page))?.value?.events?.follow?.big).toBe('ALERTA EDITADO');
 await expect(page.getByRole('alert').filter({hasText:'Falha controlada ao salvar alerta'})).toHaveCount(0);
 await expect(page.getByRole('button',{name:/Testar seguidor/})).toBeEnabled();
});
