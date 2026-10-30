import {test,expect} from '@playwright/test';
import {mock,auditPage,fmt,OUT} from './ux-helpers';
import {join} from 'node:path';

test('Pulsoid conecta no estúdio existente, protege chave e reaproveita overlay',async({page})=>{
 await mock(page);await page.route('https://fonts.googleapis.com/**',r=>r.abort());await page.route('https://fonts.gstatic.com/**',r=>r.abort());
 await page.addInitScript(()=>{
  const w=window as any,original=w.__TAURI_INTERNALS__.invoke;let connected=false,saved=false,publish=false;
  w.__TAURI_INTERNALS__.invoke=async(cmd:string,payload:any)=>{
   if(payload?.op==='snapshot'){const s=await original(cmd,payload);s.profiles[0].modules.heartRate=true;return s;}
   switch(payload?.op){
    case 'heartRate.pulsoid.get':return {hasToken:saved};
    case 'heartRate.links':return connected?{overlayUrl:'http://127.0.0.1:9876/heart-overlay#token=fixture'}:{};
    case 'heartRate.get':return {state:connected?'fresh':'disconnected',bpm:connected?89:null,ageSeconds:connected?0:null,showOnObs:publish,source:connected?'pulsoid':null,phase:connected?'waiting':'disconnected',error:''};
    case 'heartRate.pulsoid.connect':if(payload.args.token!=='QA_PRIVATE_TOKEN_123456'&&!saved)throw Error('Chave ausente');connected=true;saved=true;return {overlayUrl:'http://127.0.0.1:9876/heart-overlay#token=fixture'};
    case 'heartRate.visibility':publish=payload.args.enabled;return null;
    case 'heartRate.stop':connected=false;return null;
    case 'heartRate.pulsoid.forget':connected=false;saved=false;return null;
   }return original(cmd,payload);
  };
 });
 await page.goto('/',{waitUntil:'domcontentloaded'});await page.getByRole('button',{name:'Comunidade',exact:true}).click();await page.getByRole('button',{name:'Abrir Batimentos da live',exact:true}).click();
 await expect(page.getByText(/A criação da chave pessoal exige plano BRO/)).toBeVisible();await expect(page.getByRole('button',{name:'Conectar Pulsoid',exact:true})).toBeDisabled();
 const input=page.getByLabel('Chave pessoal do Pulsoid',{exact:true});await expect(input).toHaveAttribute('type','password');await input.fill('QA_PRIVATE_TOKEN_123456');await page.getByRole('button',{name:'Conectar Pulsoid',exact:true}).click();await expect(input).toHaveValue('');await expect(page.locator('.heart-preview strong')).toHaveText('89');
 await expect(page.getByRole('button',{name:'Copiar fonte de batimentos do OBS'})).toBeDisabled();await page.getByRole('switch',{name:'Publicar batimentos no OBS'}).click();await expect(page.getByRole('button',{name:'Copiar fonte de batimentos do OBS'})).toBeEnabled();
 await page.getByRole('button',{name:'Fechar aviso',exact:true}).click();
 for(const width of [1100,1680]){await page.setViewportSize({width,height:1000});await page.locator('.heart-studio').scrollIntoViewIfNeeded();expect(fmt(await page.evaluate(auditPage))).toEqual([]);expect(await page.locator('.lp-preview').evaluate(el=>getComputedStyle(el).position)).toBe('sticky');await page.screenshot({path:join(OUT,`pulsoid-${width}.png`),fullPage:true});}
 await page.getByRole('button',{name:'Desconectar e revogar links'}).click();await expect(page.locator('.heart-preview strong')).toHaveText('—');await expect(page.getByRole('button',{name:'Copiar fonte de batimentos do OBS'})).toBeDisabled();
 await page.getByRole('button',{name:'Conectar Pulsoid',exact:true}).click();await expect(page.locator('.heart-preview strong')).toHaveText('89');await page.getByRole('button',{name:'Remover chave salva',exact:true}).click();await expect(page.getByRole('button',{name:'Conectar Pulsoid',exact:true})).toBeDisabled();
 await page.getByRole('button',{name:'Bluetooth do computador',exact:true}).click();await expect(page.getByRole('button',{name:'Conectar relógio no navegador'})).toBeVisible();
});
