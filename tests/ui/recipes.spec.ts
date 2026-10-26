import {test,expect,type Page} from '@playwright/test';
import {join} from 'path';
import {mock,profile,OUT} from './ux-helpers';

const choose=async(page:Page,name:string)=>{
 await page.getByRole('button',{name:'Receitas prontas',exact:true}).click();
 await page.getByRole('button',{name,exact:true}).click();
 await page.getByRole('button',{name:/Configurar receita|Editar configuração existente/,exact:true}).click();
};
async function monitorWrites(page:Page){
 await page.evaluate(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(cmd:string,payload:{op:string;args?:Record<string,unknown>})=>Promise<unknown>}};
  const original=w.__TAURI_INTERNALS__.invoke;
  w.__TAURI_INTERNALS__.invoke=async(cmd,payload)=>{
   if(['flow.save','profile.save','voiceResgate.save'].includes(payload?.op)){
    const writes=JSON.parse(sessionStorage.getItem('recipeWrites')||'[]');writes.push(payload);sessionStorage.setItem('recipeWrites',JSON.stringify(writes));
   }
   return original(cmd,payload);
  };
 });
}
test('receitas de raid e voz abrem os painéis atuais sem gravar ou criar fluxos paralelos',async({page})=>{
 await mock(page);await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Comandos',exact:true}).click();
 await monitorWrites(page);
 await choose(page,'Agradecer raid');
 await expect(page.locator('main h1')).toHaveText('Comunidade');
 await expect(page.getByRole('heading',{name:'Shoutout e ganks',exact:true})).toBeVisible();
 expect(await page.evaluate(()=>sessionStorage.getItem('recipeWrites'))).toBeNull();
 await page.locator('.sidebar').getByRole('button',{name:'Comandos',exact:true}).click();
 await choose(page,'Voz por resgate');
 await expect(page.locator('main h1')).toHaveText('Voicemod');
 await expect(page.getByRole('button',{name:/Etapa 3 · Escolher/})).toHaveAttribute('aria-expanded','true');
 await expect(page.getByRole('dialog')).toHaveCount(0);
 expect(await page.evaluate(()=>sessionStorage.getItem('recipeWrites'))).toBeNull();
});

test('receita de timer usa o formulário atual e aparece desativada em Timers',async({page})=>{
 await page.addInitScript(p=>localStorage.setItem('botlive-preview',JSON.stringify({profiles:[p],flows:[],notes:{},presets:[],theme:'dark'})),profile);
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Comandos',exact:true}).click();
 await choose(page,'Lembrete de água');
 await expect(page.getByRole('dialog',{name:'Configurar timer',exact:true})).toBeVisible();
 await expect(page.getByRole('spinbutton',{name:'Repetir a cada (minutos)',exact:true})).toHaveValue('15');
 await page.getByRole('button',{name:'Salvar timer',exact:true}).click();
 await expect(page.locator('main h1')).toHaveText('Timers');
 await expect(page.getByRole('cell',{name:'A cada 15 min',exact:true})).toBeVisible();
 const data=await page.evaluate(()=>JSON.parse(localStorage.getItem('botlive-preview')||'{}'));
 expect(data.flows).toHaveLength(1);expect(data.flows[0].enabled).toBe(false);
 await page.getByRole('button',{name:'Editar Pausa para água',exact:true}).click();
 await page.getByRole('textbox',{name:'Nome',exact:true}).fill('Minha pausa');
 await page.getByRole('button',{name:'Salvar timer',exact:true}).click();

 await page.locator('.sidebar').getByRole('button',{name:'Comandos',exact:true}).click();
 await choose(page,'Lembrete de água');
 await expect(page.getByRole('textbox',{name:'Nome',exact:true})).toHaveValue('Minha pausa');
 expect((await page.evaluate(()=>JSON.parse(localStorage.getItem('botlive-preview')||'{}'))).flows).toHaveLength(1);
});

test('som por resgate exige recompensa específica e áudio antes de salvar',async({page})=>{
 await mock(page);await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Comandos',exact:true}).click();
 await monitorWrites(page);
 await choose(page,'Som por resgate');
 await expect(page.getByRole('combobox',{name:'Evento',exact:true})).toHaveValue('redemption');
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByRole('alert')).toContainText('Escolha uma recompensa específica');
 expect(await page.evaluate(()=>sessionStorage.getItem('recipeWrites'))).toBeNull();
 await page.getByRole('combobox',{name:'Recompensa da Twitch',exact:true}).selectOption('r1');
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByRole('alert')).toContainText('Escolha o som');
 await page.getByRole('combobox',{name:'Tocar áudio ao disparar',exact:true}).selectOption('snd1');
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.locator('main h1')).toHaveText('Automações');
 const writes=JSON.parse(await page.evaluate(()=>sessionStorage.getItem('recipeWrites')||'[]'));
 expect(writes).toHaveLength(1);
 expect(writes[0].args.flow).toMatchObject({enabled:false,audio:'snd1',trigger:{kind:'redemption',pattern:'r1'}});
});

test('catálogo explica o resultado e cabe em janela estreita e larga',async({page})=>{
 await mock(page);await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Comandos',exact:true}).click();
 await page.getByRole('button',{name:'Receitas prontas',exact:true}).click();
 await page.getByRole('button',{name:'Som por resgate',exact:true}).click();
 await expect(page.getByRole('region',{name:'Resumo da receita'})).toContainText('recompensa específica');
 for(const width of [1100,1600]){
  await page.setViewportSize({width,height:1000});
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.mouse.move(10,10);
  await page.screenshot({path:join(OUT,`receitas-${width}.png`),fullPage:true,animations:'disabled'});
 }
 await page.getByRole('textbox',{name:'Buscar receita',exact:true}).fill('agua');
 await expect(page.getByRole('button',{name:'Lembrete de água',exact:true})).toBeVisible();
 await expect(page.getByRole('button',{name:'Som por resgate',exact:true})).toHaveCount(0);
 await expect(page.getByRole('button',{name:'Configurar receita',exact:true})).toBeDisabled();
 await page.getByRole('textbox',{name:'Buscar receita',exact:true}).fill('xyzsemresultado');
 await expect(page.getByRole('status').filter({hasText:'Nenhuma receita encontrada'})).toBeVisible();
});
