import {test,expect} from '@playwright/test';
import {execFileSync} from 'node:child_process';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
test('manual organiza tarefas, exemplos e referência sem quebrar links antigos',async({page})=>{
 execFileSync(process.execPath,['scripts/build-docs.cjs'],{cwd:process.cwd()});
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto(pathToFileURL(resolve('entrega/MANUAL-BOTLIVE.html')).href);
 await expect(page.locator('article')).toHaveCount(20);await expect(page.locator('.nav-group')).toHaveCount(4);
 const broken=await page.locator('a[href^="#"]').evaluateAll(links=>links.filter(a=>!document.getElementById((a as HTMLAnchorElement).hash.slice(1))).map(a=>a.getAttribute('href')));expect(broken).toEqual([]);
 await page.getByRole('link',{name:'Timers e contadores',exact:true}).first().click();await expect(page.locator('#capitulo-19')).toBeInViewport();
 await page.locator('#capitulo-19 .chapter-toc summary').click();await page.locator('#capitulo-19 .chapter-toc').getByRole('link',{name:'Receita: contador de mortes',exact:true}).click();await expect(page.locator('#capitulo-19--receita-contador-de-mortes')).toBeInViewport();
 await page.getByLabel('Encontrar no manual').fill('contagem regressiva');await expect(page.locator('article:visible')).toHaveCount(1);await page.getByRole('button',{name:'Limpar busca'}).click();await expect(page.locator('article:visible')).toHaveCount(20);
 const tables=await page.locator('table').evaluateAll(tables=>tables.every(t=>Array.from(t.querySelectorAll('tbody tr')).every(r=>(r as HTMLTableRowElement).cells.length===t.querySelectorAll('thead th').length)));expect(tables).toBe(true);
 await page.goto(pathToFileURL(resolve('entrega/MANUAL-BOTLIVE.html')).href);await page.screenshot({path:'artifacts/manual-016-desktop.png',fullPage:false});
 await page.setViewportSize({width:390,height:844});expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await page.screenshot({path:'artifacts/manual-016-mobile.png',fullPage:false});
 await page.emulateMedia({media:'print'});await expect(page.locator('article:visible')).toHaveCount(20);expect(errors).toEqual([]);
});
