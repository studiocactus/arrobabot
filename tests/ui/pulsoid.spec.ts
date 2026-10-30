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
 await page.getByRole('button',{name:'Usar chave pessoal',exact:true}).click();await expect(page.getByText(/A criação da chave pessoal exige plano BRO/)).toBeVisible();await expect(page.getByRole('button',{name:'Salvar chave e conectar',exact:true})).toBeDisabled();
 const input=page.getByLabel('Chave pessoal do Pulsoid',{exact:true});await expect(input).toHaveAttribute('type','password');await input.fill('QA_PRIVATE_TOKEN_123456');await page.getByRole('button',{name:'Salvar chave e conectar',exact:true}).click();await expect(input).toHaveValue('');await expect(page.locator('.heart-preview strong')).toHaveText('89');
 await expect(page.getByRole('button',{name:'Copiar fonte de batimentos do OBS'})).toBeDisabled();await page.getByRole('switch',{name:'Publicar batimentos no OBS'}).click();await expect(page.getByRole('button',{name:'Copiar fonte de batimentos do OBS'})).toBeEnabled();
 await page.getByRole('button',{name:'Fechar aviso',exact:true}).click();
 for(const width of [1100,1680]){await page.setViewportSize({width,height:1000});await page.locator('.heart-studio').scrollIntoViewIfNeeded();expect(fmt(await page.evaluate(auditPage))).toEqual([]);expect(await page.locator('.lp-preview').evaluate(el=>getComputedStyle(el).position)).toBe('sticky');await page.screenshot({path:join(OUT,`pulsoid-${width}.png`),fullPage:true});}
 await page.getByRole('button',{name:'Desconectar e revogar links'}).click();await expect(page.locator('.heart-preview strong')).toHaveText('—');await expect(page.getByRole('button',{name:'Copiar fonte de batimentos do OBS'})).toBeDisabled();
 await page.getByRole('button',{name:'Conectar Pulsoid',exact:true}).click();await expect(page.locator('.heart-preview strong')).toHaveText('89');await page.getByRole('button',{name:'Remover chave salva',exact:true}).click();await expect(page.getByRole('button',{name:'Salvar chave e conectar',exact:true})).toBeDisabled();
 await page.getByRole('button',{name:'Bluetooth do computador',exact:true}).click();await expect(page.getByRole('button',{name:'Conectar relógio no navegador'})).toBeVisible();
});

test('Pulsoid autoriza pelo navegador sem Client Secret e conserva a mesma prévia',async({page})=>{
 await mock(page);await page.route('https://fonts.googleapis.com/**',r=>r.abort());await page.route('https://fonts.gstatic.com/**',r=>r.abort());
 await page.addInitScript(()=>{
  const w=window as any,original=w.__TAURI_INTERNALS__.invoke;let started=false;
  w.__TAURI_INTERNALS__.invoke=async(cmd:string,payload:any)=>{
   if(payload?.op==='snapshot'){const s=await original(cmd,payload);s.profiles[0].modules.heartRate=true;return s;}
   if(payload?.op==='heartRate.pulsoid.get')return {hasToken:sessionStorage.getItem('approved')==='yes',clientId:'00000000-0000-4000-8000-000000000001'};
   if(payload?.op==='heartRate.links')return {};
   if(payload?.op==='heartRate.pulsoid.auth.start'){sessionStorage.setItem('auth-request',JSON.stringify(payload.args));started=true;return {overlayUrl:'http://127.0.0.1:9876/heart-overlay#token=fixture'};}
   if(payload?.op==='url.open'){sessionStorage.setItem('opened-consent',payload.args.url);return null;}
   if(payload?.op==='heartRate.stop'){started=false;return null;}
   if(payload?.op==='heartRate.get'){const approved=sessionStorage.getItem('approved')==='yes';return {state:started&&approved?'fresh':'disconnected',bpm:started&&approved?88:null,showOnObs:false,source:started?'pulsoid':null,phase:started?(approved?'waiting':'authorizing'):'disconnected',authorization:started&&!approved?{url:'https://pulsoid.net/oauth2/device_authorization_consent?user_code=fixture',expiresAt:Date.now()/1000+600}:null};}
   return original(cmd,payload);
  };
 });
 await page.goto('/',{waitUntil:'domcontentloaded'});await page.getByRole('button',{name:'Comunidade',exact:true}).click();await page.getByRole('button',{name:'Abrir Batimentos da live',exact:true}).click();
 await expect(page.getByLabel('Client ID do Pulsoid',{exact:true})).toHaveValue('00000000-0000-4000-8000-000000000001');await expect(page.getByLabel('Chave pessoal do Pulsoid',{exact:true})).toHaveCount(0);
 await page.getByRole('button',{name:'Autorizar Pulsoid',exact:true}).click();await expect(page.getByRole('button',{name:'Abrir autorização do Pulsoid',exact:true})).toBeVisible();await expect.poll(()=>page.evaluate(()=>sessionStorage.getItem('opened-consent'))).toContain('https://pulsoid.net/oauth2/device_authorization_consent');
 expect(await page.evaluate(()=>JSON.parse(sessionStorage.getItem('auth-request')||'{}'))).toEqual({profileId:'p1',clientId:'00000000-0000-4000-8000-000000000001'});
 if(await page.getByRole('button',{name:'Fechar aviso',exact:true}).isVisible())await page.getByRole('button',{name:'Fechar aviso',exact:true}).click();
 for(const width of [1100,1680]){await page.setViewportSize({width,height:1000});await page.locator('.heart-studio').scrollIntoViewIfNeeded();expect(fmt(await page.evaluate(auditPage))).toEqual([]);await page.screenshot({path:join(OUT,`pulsoid-autorizacao-${width}.png`),fullPage:true});}
 await page.evaluate(()=>sessionStorage.setItem('approved','yes'));await expect(page.locator('.heart-preview strong')).toHaveText('88');await expect(page.getByRole('button',{name:'Abrir autorização do Pulsoid',exact:true})).toHaveCount(0);await expect(page.getByRole('button',{name:'Conectar Pulsoid',exact:true})).toBeVisible();await page.getByRole('button',{name:'Desconectar e revogar links',exact:true}).click();await expect(page.locator('.heart-preview strong')).toHaveText('—');
});
