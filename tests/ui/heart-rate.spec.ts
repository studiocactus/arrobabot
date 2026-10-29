import {test,expect} from '@playwright/test';
import {join} from 'node:path';
import {readFileSync} from 'node:fs';
import {mock,OUT,auditPage,fmt} from './ux-helpers';

test('batimentos: prévia privada, sinal antigo e layout do estúdio',async({page})=>{
 await mock(page);
 await page.addInitScript(()=>{
  const w=window as any,original=w.__TAURI_INTERNALS__.invoke;let publish=false;
  w.__TAURI_INTERNALS__.invoke=async(cmd:string,payload:any)=>{
   if(payload?.op==='snapshot'){const value=await original(cmd,payload);value.profiles[0].modules.heartRate=true;return value;}
   if(payload?.op==='heartRate.get')return {state:sessionStorage.getItem('sensor-stale')?'waiting':'fresh',bpm:sessionStorage.getItem('sensor-stale')?null:83,ageSeconds:sessionStorage.getItem('sensor-stale')?11:0,showOnObs:publish};
   if(payload?.op==='heartRate.visibility'){publish=payload.args.enabled;return null;}
   if(payload?.op==='heartRate.stop'){sessionStorage.setItem('sensor-stale','1');return null;}
   return original(cmd,payload);
  };
 });
 await page.goto('/');await page.getByRole('button',{name:'Comunidade',exact:true}).click();await page.getByRole('button',{name:'Abrir Batimentos da live',exact:true}).click();
 await expect(page.locator('.heart-preview strong')).toHaveText('83');await expect(page.getByRole('switch',{name:'Publicar batimentos no OBS'})).toHaveAttribute('aria-checked','false');
 for(const width of [1100,1680]){
  await page.setViewportSize({width,height:1000});await page.locator('.heart-studio').scrollIntoViewIfNeeded();
  const issues=await page.evaluate(auditPage);expect(fmt(issues)).toEqual([]);
  expect(await page.locator('.lp-preview').evaluate(el=>getComputedStyle(el).position)).toBe('sticky');
  await page.screenshot({path:join(OUT,`batimentos-${width}.png`),fullPage:true});
 }
 await page.evaluate(()=>sessionStorage.setItem('sensor-stale','1'));await expect(page.locator('.heart-preview strong')).toHaveText('—');
 await page.getByRole('switch',{name:'Publicar batimentos no OBS'}).click();await expect(page.getByRole('switch',{name:'Publicar batimentos no OBS'})).toHaveAttribute('aria-checked','true');
});

test('OBS oferece os novos controles no painel existente',async({page})=>{
 await mock(page);await page.goto('/');await page.getByRole('button',{name:'OBS Studio',exact:true}).click();
 await page.getByRole('combobox',{name:'Operação',exact:true}).selectOption('rotate');await expect(page.getByLabel('Rotação (graus)')).toHaveValue('0');
 for(const op of ['media_play','media_pause','media_stop','media_restart','media_next','media_previous','flip_horizontal','flip_vertical'])await page.getByRole('combobox',{name:'Operação',exact:true}).selectOption(op);
 await expect(page.getByLabel('Rotação (graus)')).toHaveCount(0);
});

test('ponte mostra só a leitura confirmada e encerra a sessão Bluetooth',async({page})=>{
 await page.route('**/__heart-test',route=>route.fulfill({contentType:'text/html',body:readFileSync('scripts/heart-connect.html','utf8')}));
 await page.addInitScript(()=>{
  const w=window as any;const sessions:any[]=[];
  Object.defineProperty(navigator,'bluetooth',{value:{requestDevice:async()=>{
   const characteristic:any=new EventTarget();characteristic.startNotifications=async()=>{};
   const device:any=new EventTarget();device.gatt={connected:false,connect:async()=>{device.gatt.connected=true;return device.gatt},disconnect:()=>{device.gatt.connected=false;device.dispatchEvent(new Event('gattserverdisconnected'))},getPrimaryService:async()=>({getCharacteristic:async()=>characteristic})};
   sessions.push({device,characteristic});return device;
  }}});
  w.WebSocket=class{
   readyState=0;onopen:any;onclose:any;onmessage:any;closed=false;
   constructor(){sessions.at(-1).ws=this;setTimeout(()=>{this.readyState=1;this.onopen?.()},0)}
   send(raw:string){const value=JSON.parse(raw);if(value.mode==='heart-publisher')setTimeout(()=>this.onmessage?.({data:JSON.stringify({type:'heart.ready'})}),0);else sessions.at(-1).packet=value.packet;}
   close(){if(this.closed)return;this.closed=true;this.readyState=3;this.onclose?.()}
  };
  w.sensorQA={push:()=>{const s=sessions.at(-1);s.characteristic.value=new DataView(new Uint8Array([0,77]).buffer);s.characteristic.dispatchEvent(new Event('characteristicvaluechanged'));},ack:()=>sessions.at(-1).ws.onmessage({data:JSON.stringify({type:'heart-reading',payload:{state:'fresh',bpm:78}})}),sessions};
 });
 await page.goto('/__heart-test#token=fixture');await page.getByRole('button',{name:'Escolher relógio e conectar'}).click();await expect(page.locator('#status')).toHaveText('Conectado. Aguardando medição Bluetooth.');
 await page.evaluate(()=>(window as any).sensorQA.push());await expect(page.locator('#bpm')).toHaveText('—');expect(await page.evaluate(()=>(window as any).sensorQA.sessions[0].packet)).toEqual([0,77]);
 await page.evaluate(()=>(window as any).sensorQA.ack());await expect(page.locator('#bpm')).toHaveText('78');
 await page.getByRole('button',{name:'Desconectar',exact:true}).click();await expect(page.locator('#bpm')).toHaveText('—');expect(await page.evaluate(()=>(window as any).sensorQA.sessions[0].device.gatt.connected)).toBe(false);
 await page.getByRole('button',{name:'Escolher relógio e conectar'}).click();await expect(page.locator('#status')).toHaveText('Conectado. Aguardando medição Bluetooth.');
 await page.evaluate(()=>(window as any).sensorQA.sessions[0].device.dispatchEvent(new Event('gattserverdisconnected')));expect(await page.evaluate(()=>(window as any).sensorQA.sessions[1].ws.closed)).toBe(false);
});
