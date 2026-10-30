import {test,expect,type Page} from '@playwright/test';
import {join} from 'path';
import {mock,OUT,auditPage,fmt} from './ux-helpers';
async function dashboard(page:Page){
 await mock(page);
 await page.addInitScript(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(cmd:string,payload:{op:string;args?:{profileId?:string}})=>Promise<any>}};
  const original=w.__TAURI_INTERNALS__.invoke;
  w.__TAURI_INTERNALS__.invoke=async(cmd,payload)=>{
   const op=payload?.op;const at=Math.floor(Date.now()/1000);const mode=sessionStorage.getItem('dashboard-mode')||'live';
   if(op==='snapshot'){const v=await original(cmd,payload);return {...v,profiles:[...v.profiles,{...v.profiles[0],id:'p2',name:'Segundo canal',channel:'segundo'}],statuses:{p1:'online',p2:'online'}}}
   if(op==='dashboard.get'){
    if(payload.args?.profileId==='p1'&&sessionStorage.getItem('slow'))await new Promise(r=>setTimeout(r,1500));
    const second=payload.args?.profileId==='p2';
    const data={live:mode!=='offline',id:'stream-'+payload.args?.profileId,title:second?'Live do segundo canal':'Minecraft com a comunidade',category:'Minecraft',startedAt:new Date((at-3600)*1000).toISOString(),viewers:128};
    return {platform:{state:mode==='error'||mode==='auth'?'stale':'fresh',lastSuccessAt:mode==='error'?at-200:at,observedAt:at,error:mode==='auth'?'A autorização expirou. Conecte a conta novamente.':mode==='error'?'Twitch indisponível (HTTP 429)':null,data},local:{observedAt:at,messagesPerMinute:24,activeChatters:37,subscriptions:{state:mode==='auth'?'unavailable':'fresh',registered:10,failed:0,at},session:{id:data.id,startedAt:data.startedAt,observedAt:at-300,coveredSeconds:240,viewerSeconds:24000,peak:142,samples:[[at-300,80],[at-270,90],[at-240,null],[at-30,128],[at,128]],messages:200,follows:5,subs:2,resubs:1,gifts:3,bits:100,raids:1,redemptions:8},request:{at,state:'completed',text:'Título alterado: Segredo do canal',delivery:{chat:'sent',voice:'delivered'}},audio:{id:'sound1',kind:'tts',sink:'obs',phase:'done',at},goal:{enabled:true,title:'Meta teste',target:10000,current:1000,updatedAt:at,startAt:at-300}}};
   }
   if(op==='dashboard.obs')return {state:'fresh',lastSuccessAt:at,observedAt:at,data:{streaming:true,reconnecting:false,recording:true,fps:60,cpu:10,bytes:10000+at*1000,durationMs:3600000,skipped:0,total:10000,scene:'Jogo'}};
   if(op==='liveConsole.get')return {channel:'online',audio:{clients:1},voice:{phase:'connected',restorePending:mode==='pending',testActive:false},voiceQueue:{activeName:null,waiting:2,pendingRestore:mode==='pending'},obs:{temporary:0,pendingRestore:0},running:1,issues:mode==='history'?5:0};
   if(op==='labels.viewers')return {total:400,users:['PessoaNoChat']};
   return original(cmd,payload);
  };
 });await page.goto('/');
}
test('dashboard prioriza dados reais e preserva distinção de presença, com capturas',async({page})=>{
 await dashboard(page);
 await expect(page.getByText('● AO VIVO NA TWITCH',{exact:true})).toBeVisible();
 await expect(page.locator('.dashboard-kpi').filter({hasText:'Espectadores agora'}).locator('strong')).toHaveText('128');
 await expect(page.locator('.dashboard-kpi').filter({hasText:'Mensagens por minuto'}).locator('strong')).toHaveText('24');
 await expect(page.locator('.hero')).toHaveCount(0);
 await expect(page.getByRole('heading',{name:'O que você quer configurar?'})).toBeHidden();
 await expect(page.getByRole('heading',{name:'Conexões e transmissão'})).toBeVisible();
 await expect(page.getByText('10 assinaturas ativas')).toBeVisible();
 for(const width of [1100,1600]){await page.setViewportSize({width,height:1000});expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await page.screenshot({path:join(OUT,`dashboard-live-${width}.png`),fullPage:true,animations:'disabled'});}
 const ops=await page.evaluate(()=>(window as unknown as {__ux:{ops:string[]}}).__ux.ops);expect(ops.filter(op=>/\.(save|execute|connect|test|stop)$/.test(op))).toEqual([]);
});
test('offline confirmado mostra banner; falha não vira zero ou fora do ar',async({page})=>{
 await dashboard(page);await page.evaluate(()=>sessionStorage.setItem('dashboard-mode','offline'));await page.getByRole('button',{name:'Atualizar painel',exact:true}).click();
 await expect(page.getByText('FORA DO AR NA TWITCH',{exact:true})).toBeVisible();await expect(page.locator('.hero')).toBeVisible();
 await page.evaluate(()=>sessionStorage.setItem('dashboard-mode','error'));await page.getByRole('button',{name:'Atualizar painel',exact:true}).click();
 await expect(page.getByText('ESTADO DA LIVE NÃO CONFIRMADO',{exact:true})).toBeVisible();await expect(page.locator('.hero')).toHaveCount(0);
 await expect(page.locator('.dashboard-kpi').filter({hasText:'Espectadores agora'}).locator('strong')).toHaveText('—');
 await expect(page.getByText('Twitch indisponível (HTTP 429)',{exact:true})).toBeVisible();
});
test('privacidade e personalização persistem sem esconder alertas importantes',async({page})=>{
 await dashboard(page);await page.getByText('Personalizar painel',{exact:true}).click();await page.getByRole('switch',{name:'Modo privacidade',exact:true}).click();
 await expect(page.getByText('Título alterado: Segredo do canal',{exact:true})).toHaveCount(0);
 await page.getByRole('switch',{name:'Gráfico de audiência',exact:true}).click();await expect(page.getByRole('heading',{name:'Audiência durante esta live',exact:true})).toHaveCount(0);
 await page.reload();await expect(page.getByText('Informações da live ocultas',{exact:true})).toBeVisible();await expect(page.getByRole('heading',{name:'Audiência durante esta live',exact:true})).toHaveCount(0);
 await page.evaluate(()=>sessionStorage.setItem('dashboard-mode','pending'));await page.getByRole('button',{name:'Atualizar estado',exact:true}).click();
 await expect(page.getByLabel('Precisa de atenção').getByText('Voicemod: a restauração da voz precisa de atenção.',{exact:true})).toBeVisible();
});
test('resposta atrasada não contamina outro perfil',async({page})=>{
 await dashboard(page);await page.evaluate(()=>sessionStorage.setItem('slow','yes'));await page.getByRole('button',{name:'Atualizar painel',exact:true}).click();
 await page.getByLabel('Perfil ativo',{exact:true}).selectOption('p2');await expect(page.getByRole('heading',{name:'Live do segundo canal',exact:true})).toBeVisible();
 await page.waitForTimeout(1800);await expect(page.getByRole('heading',{name:'Minecraft com a comunidade',exact:true})).toHaveCount(0);
});


test('avisos resolvidos somem sem transformar falhas antigas em pendências atuais',async({page})=>{
 await dashboard(page);await page.evaluate(()=>sessionStorage.setItem('dashboard-mode','auth'));await page.getByRole('button',{name:'Atualizar painel',exact:true}).click();
 const attention=page.getByLabel('Precisa de atenção',{exact:true});await expect(attention).toContainText('A autorização expirou.');await expect(attention).toContainText('Recebimento de eventos da Twitch');
 await page.evaluate(()=>sessionStorage.setItem('dashboard-mode','live'));await page.getByRole('button',{name:'Atualizar painel',exact:true}).click();await expect(attention).toHaveCount(0);
 await page.evaluate(()=>sessionStorage.setItem('dashboard-mode','pending'));await page.getByRole('button',{name:'Atualizar estado',exact:true}).click();await expect(attention).toContainText('restauração da voz');
 await page.evaluate(()=>sessionStorage.setItem('dashboard-mode','history'));await page.getByRole('button',{name:'Atualizar estado',exact:true}).click();await expect(attention).toHaveCount(0);await expect(page.locator('.operation-card').last()).toContainText('5 falhas ou interrupções registradas');
 for(const width of [1100,1680]){await page.setViewportSize({width,height:1000});await page.locator('.live-console').scrollIntoViewIfNeeded();expect(fmt(await page.evaluate(auditPage))).toEqual([]);await page.screenshot({path:join(OUT,`operacao-moderna-${width}.png`),fullPage:true});}
});

test('opções da IA mantêm descrições dentro dos botões e rolagem usa a cor do BotLive',async({page})=>{
 await mock(page);await page.route('https://fonts.googleapis.com/**',r=>r.abort());await page.route('https://fonts.gstatic.com/**',r=>r.abort());await page.goto('/',{waitUntil:'domcontentloaded'});await page.getByRole('button',{name:'Inteligência artificial',exact:true}).click();
 for(const width of [1100,1680]){await page.setViewportSize({width,height:1000});const contained=await page.locator('.provider').evaluateAll(buttons=>buttons.every(b=>{const outer=b.getBoundingClientRect();return [...b.children].every(c=>{const r=c.getBoundingClientRect();return r.top>=outer.top&&r.bottom<=outer.bottom&&r.right<=outer.right&&r.left>=outer.left})}));expect(contained).toBe(true);expect(fmt(await page.evaluate(auditPage))).toEqual([]);await page.screenshot({path:join(OUT,`ia-campos-${width}.png`),fullPage:true});}
 const colors=await page.evaluate(()=>({actual:getComputedStyle(document.documentElement).scrollbarColor,accent:getComputedStyle(document.documentElement).getPropertyValue('--accent')}));expect(colors.actual).not.toBe('auto');expect(colors.accent.trim()).not.toBe('');
});
