import {test,expect} from '@playwright/test';
import {mock} from './ux-helpers';
import {join} from 'node:path';
import {tmpdir} from 'node:os';

test('topo usa o mesmo box da busca e foto do canal, com alternativa se a imagem falhar',async({page})=>{
 await mock(page);
 await page.route('https://fonts.googleapis.com/**',route=>route.abort());
 await page.route('https://fonts.gstatic.com/**',route=>route.abort());
 await page.route('https://avatar.example/**',route=>route.fulfill({contentType:'image/svg+xml',body:'<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="#8866bb"/><text x="15" y="43" fill="white" font-size="30">TC</text></svg>'}));
 await page.addInitScript(()=>{
  const w=window as any,original=w.__TAURI_INTERNALS__.invoke;
  w.__TAURI_INTERNALS__.invoke=async(cmd:string,payload:any)=>{
   if(payload?.op==='snapshot'){const s=await original(cmd,payload);s.statuses.p1='online';s.profiles[0].channelId='canal-1';return s;}
   if(payload?.op==='labels.viewers')return {total:7,users:['ana','pedro']};
   if(payload?.op==='twitch.identity')return {displayName:'Canal Twitch',imageUrl:'https://avatar.example/channel.svg'};
   return original(cmd,payload);
  };
 });
 await page.goto('/',{waitUntil:'domcontentloaded'});
 const image=page.getByRole('img',{name:'Foto de Canal Twitch na Twitch'});
 await expect(image).toBeVisible();await expect.poll(()=>image.evaluate((img:HTMLImageElement)=>img.naturalWidth)).toBeGreaterThan(0);
 for(const width of [1100,1680]){
  await page.setViewportSize({width,height:1000});
  const boxes=await page.evaluate(()=>['.presence-pill','.quick-search'].map(sel=>{const e=document.querySelector(sel)!;const s=getComputedStyle(e);return {height:e.getBoundingClientRect().height,radius:s.borderRadius,background:s.backgroundColor,border:s.borderColor}}));
  expect(boxes[0]).toEqual(boxes[1]);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.screenshot({path:join(tmpdir(),'botlive-ux',`topbar-conta-${width}.png`),fullPage:true});
 }
 await page.getByRole('button',{name:'Quem está no chat agora',exact:true}).click();await expect(page.getByRole('dialog',{name:'Quem está no chat agora'})).toContainText('ana');
 await image.evaluate(img=>img.dispatchEvent(new Event('error')));
 await expect(image).toHaveCount(0);await expect(page.getByLabel('Conta: Canal Twitch',{exact:true})).toBeVisible();
});
