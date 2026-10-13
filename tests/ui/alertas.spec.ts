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
      return {kind:String(args.kind),vars:{name:'NovoSeguidor',amount:500,months:1,tier:'TIER 1',message:''}};
     }
     if(op==='module.config'){
      sessionStorage.setItem('alertsSave',JSON.stringify({key:args.key,value:args.value}));
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

 await page.getByRole('button',{name:/Testar bits/}).click();
 await expect.poll(async()=>await page.evaluate(()=>sessionStorage.getItem('alertsTest')||''),{timeout:3000}).toBe('cheer');

 // O painel animado (não o estático) mostra os dados de exemplo e some sozinho.
 await expect(page.locator('.bl-alert:not(.is-static)')).toContainText('NovoSeguidor',{timeout:5000});
 await expect(page.locator('.bl-alert:not(.is-static)')).toContainText('500 BITS',{timeout:5000});
 await expect(page.locator('.bl-alert:not(.is-static)')).toHaveCount(0,{timeout:25000});
 // A prévia parada volta pronta para a próxima edição.
 await expect(page.locator('.bl-alert.is-static')).toContainText('500 BITS');
});

test('estúdio de alertas cabe em janela de 1100px e ampla sem espremer texto',async({page})=>{
 await page.setViewportSize({width:1100,height:950});
 await setup(page);
 await expect(page.locator('.bl-alert.is-static')).toBeVisible();
 const narrow=await page.evaluate(auditPage);
 expect(fmt(narrow),`${narrow.length} achado(s) em 1100px`).toEqual([]);
 await page.screenshot({path:'artifacts/estudio-alertas-1100.png'});

 await page.setViewportSize({width:1680,height:1000});
 await page.waitForTimeout(250);
 const wide=await page.evaluate(auditPage);
 expect(fmt(wide),`${wide.length} achado(s) em 1680px`).toEqual([]);
 await page.screenshot({path:'artifacts/estudio-alertas-largo.png'});

 // Nenhuma largura produz rolagem horizontal.
 const overflow=await page.evaluate(()=>document.documentElement.scrollWidth-document.documentElement.clientWidth);
 expect(overflow).toBeLessThanOrEqual(1);
});

test('configurações ficam em acordeão abaixo da prévia',async({page})=>{
 await setup(page);

 // Prévia em cima, configurações embaixo (ordem do DOM).
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
