import {test,expect} from '@playwright/test';
import {join} from 'path';
import {mock,OUT} from './ux-helpers';

test('organização preserva destinos e roteiro leva às telas existentes',async({page})=>{
 await mock(page);await page.goto('/');
 const nav=page.locator('.sidebar');
 await expect(nav.locator('.nav-group')).toHaveText(['SEU CANAL','CHAT','VISUAL DO OBS','ÁUDIO E VOZ','AUTOMAÇÃO E IA','COMUNIDADE','ACOMPANHAMENTO']);
 await expect(nav.locator('.nav-item')).toHaveCount(24);
 await page.getByText('Preparação e configuração',{exact:true}).click();
 const hub=page.locator('.setup-hub');
 await hub.getByRole('button',{name:/Preparar sons e voz/}).click();
 await expect(hub.getByRole('button',{name:/Preparar sons e voz/})).toHaveAttribute('aria-pressed','true');
 await hub.getByRole('button',{name:'3. Configurar voz e resgates no Voicemod'}).click();
 await expect(page.locator('main h1')).toHaveText('Voicemod');
 await expect(nav.getByRole('button',{name:'Voicemod',exact:true})).toHaveAttribute('aria-current','page');
});

test('busca por tarefa aceita sinônimos, acentos e ausência de resultados',async({page})=>{
 await mock(page);await page.goto('/');
 await page.getByText('Preparação e configuração',{exact:true}).click();
 await expect(page.getByRole('heading',{name:'O que você quer configurar?'})).toBeVisible();
 await page.keyboard.press('Control+k');
 const input=page.getByRole('textbox',{name:'Buscar tela ou tarefa'});
 await input.fill('resgate');
 await expect(page.locator('.palette-results').getByRole('button',{name:'Voicemod',exact:true})).toBeVisible();
 await expect(page.locator('.palette-results').getByRole('button',{name:'Automações',exact:true})).toBeVisible();
 await input.fill('erro');
 await page.locator('.palette-results').getByRole('button',{name:'Histórico',exact:true}).click();
 await expect(page.locator('main h1')).toHaveText('Histórico');
 await page.keyboard.press('Control+k');
 await input.fill('xyzsemresultado');
 await expect(page.getByRole('status').filter({hasText:'Nenhuma tela encontrada'})).toBeVisible();
 await page.keyboard.press('Escape');
 await expect(input).toHaveCount(0);
});

test('primeiro uso explica a necessidade do perfil sem liberar etapas vazias',async({page})=>{
 await page.goto('/');
 const hub=page.locator('.setup-hub');
 await hub.getByRole('button',{name:/Preparar sons e voz/}).click();
 await expect(hub.getByText('Crie um perfil para configurar os recursos do seu canal.')).toBeVisible();
 await expect(hub.getByRole('button',{name:/3. Configurar voz/})).toBeDisabled();
 await hub.getByRole('button',{name:'Criar perfil para começar'}).click();
 await expect(page.getByRole('heading',{name:'Criar perfil de bot'})).toBeVisible();
});

test('roteiro de configuração cabe em janela estreita e larga',async({page})=>{
 await mock(page);await page.goto('/');
 await page.getByText('Preparação e configuração',{exact:true}).click();
 const hub=page.locator('.setup-hub');
 await hub.getByRole('button',{name:/Montar o visual da live/}).click();
 for(const width of [1100,1600]){
  await page.setViewportSize({width,height:1000});
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.screenshot({path:join(OUT,`configurar-objetivo-${width}.png`),fullPage:true});
 }
});


test('banner original aparece sem conexão e sai quando o bot está online',async({page})=>{
 await mock(page);
 await page.addInitScript(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(cmd:string,payload:{op:string})=>Promise<any>}};
  const original=w.__TAURI_INTERNALS__.invoke;
  w.__TAURI_INTERNALS__.invoke=async(cmd,payload)=>{
   const value=await original(cmd,payload);
   if(payload?.op==='snapshot')return {...value,statuses:{p1:sessionStorage.getItem('banner-online')==='yes'?'online':'offline'}};
   return value;
  };
 });
 await page.goto('/');
 const banner=page.locator('.hero');
 await expect(banner).toBeVisible();
 await expect(banner.locator('h2')).toHaveText('Menos tarefas.Mais momentos ao vivo.');
 await expect(banner.locator('.bot-illustration')).toBeVisible();
 for(const width of [1100,1600]){
  await page.setViewportSize({width,height:1000});
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.screenshot({path:join(OUT,`banner-original-${width}.png`),fullPage:true});
 }
 await banner.getByRole('button',{name:'Criar uma automação',exact:true}).click();
 await expect(page.locator('main h1')).toHaveText('Automações');
 await page.evaluate(()=>sessionStorage.setItem('banner-online','yes'));
 await page.reload();
 await expect(page.locator('main h1')).toBeVisible();
 await expect(page.locator('.hero')).toHaveCount(0);
});
