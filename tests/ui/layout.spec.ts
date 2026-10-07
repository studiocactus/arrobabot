import {test,expect} from '@playwright/test';
import {mock,auditPage,fmt,NAV,OUT} from './ux-helpers';
import {writeFileSync} from 'fs';

const SIZES=[{w:1280,h:820,tag:'padrao'},{w:1440,h:1000,tag:'ampla'},{w:1152,h:820,tag:'estreita1'},{w:922,h:700,tag:'estreita2'},{w:680,h:540,tag:'minima'}];

test('auditoria de layout',async({page})=>{
 test.setTimeout(900000);
 const report:string[]=[];
 const flush=()=>writeFileSync(`${OUT}\\relatorio.txt`,report.join('\n'),'utf8');
 const errors:string[]=[];
 page.on('pageerror',e=>errors.push(e.message.slice(0,300)));
 await mock(page);

 const step=async(label:string,shot?:string)=>{
  await page.waitForTimeout(400);
  const issues=await page.evaluate(auditPage);
  const info=await page.evaluate(()=>{
   const t=(sel:string)=>document.querySelector(sel)?.textContent||'';
   const h=t('main h1')||t('main h3')||t('main .card-heading h3');
   const ph=document.querySelector('.profile-id h2') as HTMLElement|null;
   const prof=ph?` | perfil="${ph.textContent}" cortado=${ph.scrollWidth>ph.clientWidth+1} meta=${!!document.querySelector('.profile-meta')}`:'';
   return {h,fatal:document.querySelectorAll('.fatal').length,failure:document.querySelectorAll('.screen-failure').length,prof,
    navs:document.querySelectorAll('.nav-item').length};
  });
  const navs=info.navs;
  if(info.failure>0)issues.unshift({k:'tela-com-falha',sel:'.screen-failure',txt:'a tela foi contida pelo ScreenBoundary',extra:''});
  if(info.fatal)issues.unshift({k:'app-quebrado',sel:'main.fatal',txt:'boundary raiz assumiu a tela',extra:''});
  report.push(`\n## ${label} — ${issues.length} achado(s) | nav=${navs} | tela="${info.h}"${info.prof}`);
  report.push(...fmt(issues));
  if(errors.length){report.push('  !! pageerrors: '+errors.join(' || ').slice(0,600));errors.length=0}
  if(shot)await page.screenshot({path:`${OUT}\\${shot}.png`});
  flush();
  return {issues,navs,failed:info.failure};
 };

 for(const s of SIZES){
  await page.setViewportSize({width:s.w,height:s.h});
  await page.goto('/');
  report.push(`\n========== VIEWPORT ${s.w}×${s.h} (${s.tag}) ==========`);
  for(const nav of NAV){
   try{
    const btn=page.getByRole('button',{name:nav,exact:true});
    const vis=await btn.count()&&await btn.first().isVisible();
    if(!vis){
     const menu=page.getByRole('button',{name:'Abrir menu'});
     if(await menu.count()){await menu.click({timeout:4000});await page.waitForTimeout(250)}
    }
    const btn2=page.getByRole('button',{name:nav,exact:true});
    if(await btn2.count()===0){report.push(`\n## ${nav} — BOTÃO AUSENTE`);continue}
    await btn2.first().click({timeout:5000});
   }catch(e){report.push(`\n## ${nav} — clique falhou: ${String(e).slice(0,140)}`);continue}
   const shot=(s.tag==='padrao')?nav.replace(/[^\w]/g,'_'):(s.tag==='minima'&&['Visão geral','Automações','Comunidade','Estatísticas'].includes(nav)?`${s.tag}_${nav.replace(/[^\w]/g,'_')}`:undefined);
   const r=await step(`${s.tag} · ${nav}`,shot);
   if(r.navs===0){report.push('  ⚠ shell perdido');flush();break}
  }
 }

 await page.setViewportSize({width:1280,height:820});
 await page.goto('/');
 try{
  await page.getByRole('button',{name:'Automações',exact:true}).click({timeout:5000});
  await page.waitForTimeout(350);
  await page.getByRole('button',{name:'Editar Boas-vindas',exact:true}).click({timeout:5000});
  await step('Editor de automação','editor');
  await page.getByRole('button',{name:'Testar fluxo',exact:true}).click({timeout:5000});
  await step('Painel Testar fluxo','teste');
  await page.getByRole('button',{name:'Iniciar teste',exact:true}).click({timeout:5000});
  await step('Testar fluxo em execução','teste-execucao');
 }catch(e){report.push(`\n## Editor/Teste — FALHOU: ${String(e).slice(0,300)}`)}

 try{
  const ux=await page.evaluate(()=>JSON.stringify((window as unknown as {__ux?:{unknown:string[]}}).__ux?.unknown||[]));
  report.push('\n## ops não mapeadas no mock: '+ux);
 }catch{report.push('\n## ops: página indisponível')}
 flush();
});

/** Uma tela que falha não pode levar o aplicativo junto: o contêiner precisa segurar. */
test('falha numa tela fica contida no conteúdo',async({page})=>{
 await mock(page);
 await page.addInitScript(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(c:string,p:{op?:string})=>unknown}};
  const inner=w.__TAURI_INTERNALS__;
  const orig=inner.invoke.bind(inner);
  inner.invoke=async(c:string,p:{op?:string})=>p?.op==='stats'?{messages:1,actions:1,followers:1,hours:null,commands:[]}:orig(c,p);
 });
 await page.goto('/');
 await page.getByRole('button',{name:'Estatísticas',exact:true}).click();

 await page.locator('.screen-failure').waitFor({state:'visible',timeout:5000});
 await expect(page.locator('.screen-failure h3')).toHaveText('Esta tela encontrou um problema');
 // a barra lateral continua de pé e o app inteiro não foi substituído
 expect(await page.locator('.nav-item').count()).toBe(19);
 expect(await page.locator('main.fatal').count()).toBe(0);

 await page.getByRole('button',{name:'Ir para a Visão geral',exact:true}).click();
 await page.locator('.screen-failure').waitFor({state:'detached',timeout:5000});
 await expect(page.locator('main h1')).toHaveText('Sua live, bem acompanhada.');
});
