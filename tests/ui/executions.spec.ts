import {test,expect} from '@playwright/test';
import {join} from 'path';
import {mock,OUT} from './ux-helpers';

test('histórico acompanha execuções sem executar efeitos, filtra e preserva eventos gerais',async({page})=>{
 await mock(page);
 await page.goto('/');await expect(page.locator('.sidebar')).toBeVisible();await page.evaluate(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(cmd:string,payload:{op:string})=>Promise<unknown>}};
  const original=w.__TAURI_INTERNALS__.invoke;
  w.__TAURI_INTERNALS__.invoke=async(cmd,payload)=>payload?.op==='flow.runs'?[
   {id:'run-failure',flow:'Som da recompensa',status:'FAILED',test:false,startedAt:1792911600000,durationMs:1250,context:{eventKind:'redemption',eventRef:'evt-0123456789abcdef',mode:'live'},error:'Arquivo de som indisponível',steps:[{index:0,kind:'audio',status:'FAILED',durationMs:1250},{index:1,kind:'chat',status:'PENDING'}]},
   {id:'run-condition',flow:'Condição do chat',status:'STOPPED_BY_CONDITION',test:true,steps:[{index:0,kind:'condition',status:'FALSE'}]}
  ]:original(cmd,payload);
 });
 await page.locator('.sidebar').getByRole('button',{name:'Histórico',exact:true}).click();
 await page.getByRole('button',{name:/Som da recompensa Falhou/}).click();
 await expect(page.getByRole('alert')).toHaveText('Arquivo de som indisponível');
 await expect(page.getByText('Resgate de pontos · Execução da live',{exact:true})).toBeVisible();
 await expect(page.getByText('Referência do evento: evt-0123456789abcdef',{exact:true})).toBeVisible();
 await page.getByLabel('Buscar execução',{exact:true}).fill('evt-0123456789abcdef');
 await expect(page.locator('.execution-list button')).toHaveCount(1);
 await page.getByLabel('Buscar execução',{exact:true}).fill('resgate');
 await expect(page.locator('.execution-list button')).toHaveCount(1);
 await page.getByLabel('Buscar execução',{exact:true}).fill('');
 await expect(page.getByText('Aguardando execução · Não registrado',{exact:true})).toBeVisible();
 await page.getByLabel('Mostrar',{exact:true}).selectOption('failed');
 await expect(page.getByRole('button',{name:/Condição do chat Parada/})).toHaveCount(0);
 await page.getByLabel('Mostrar',{exact:true}).selectOption('test');
 await page.getByRole('button',{name:/Condição do chat Parada/}).click();
 await expect(page.getByText('Condição falsa · Não registrado',{exact:true})).toBeVisible();
 await expect(page.getByText('Origem não registrada para esta execução.',{exact:true})).toBeVisible();
 await page.getByLabel('Buscar execução',{exact:true}).fill('inexistente');
 await expect(page.getByText('Nenhuma execução corresponde ao filtro')).toBeVisible();
 await page.getByLabel('Buscar execução',{exact:true}).fill('');await page.getByLabel('Mostrar',{exact:true}).selectOption('all');
 await page.getByRole('button',{name:/Som da recompensa Falhou/}).click();
 for(const width of [1100,1600]){
  await page.setViewportSize({width,height:1000});
  await expect.poll(()=>page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBeTruthy();
  await page.screenshot({path:join(OUT,`execucoes-${width}.png`),fullPage:true,animations:'disabled'});
 }
 for(const button of await page.locator('.execution-list button').all()){expect(await button.evaluate(el=>el.scrollHeight<=el.clientHeight)).toBeTruthy()}
 await page.getByRole('button',{name:'Eventos gerais',exact:true}).click();
 await expect(page.getByLabel('Filtrar histórico')).toBeVisible();
 await expect(page.getByText('Fonte mutada',{exact:true})).toBeVisible();
 const ops=await page.evaluate(()=>(window as unknown as {__ux:{ops:string[]}}).__ux.ops);
 expect(ops.filter(op=>['flow.test','flow.cancel','flow.rehearse','flow.save'].includes(op))).toEqual([]);
});

test('falha de consulta não vira histórico vazio e permite recuperação',async({page})=>{
 await mock(page);await page.goto('/');await expect(page.locator('.sidebar')).toBeVisible();await page.evaluate(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(cmd:string,payload:{op:string})=>Promise<unknown>}};
  const original=w.__TAURI_INTERNALS__.invoke;
  w.__TAURI_INTERNALS__.invoke=async(cmd,payload)=>{if(payload?.op==='flow.runs'){if(!sessionStorage.getItem('recoverRuns'))throw Error('Conexão interrompida');return []}return original(cmd,payload)};
 });
 await page.locator('.sidebar').getByRole('button',{name:'Histórico',exact:true}).click();
 await expect(page.getByRole('alert')).toContainText('Conexão interrompida');
 await expect(page.getByText('Nenhuma execução registrada')).toHaveCount(0);
 await page.evaluate(()=>sessionStorage.setItem('recoverRuns','yes'));
 await page.getByRole('button',{name:'Tentar novamente',exact:true}).click();
 await expect(page.getByText('Nenhuma execução registrada')).toBeVisible();
 await expect(page.getByRole('alert')).toHaveCount(0);
});


test('cancelamento exige confirmação e aguarda estado final do motor sem repetir o pedido',async({page})=>{
 await mock(page);await page.goto('/');await expect(page.locator('.sidebar')).toBeVisible();await page.evaluate(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(cmd:string,payload:{op:string;args?:Record<string,unknown>})=>Promise<unknown>}};
  const original=w.__TAURI_INTERNALS__.invoke;
  w.__TAURI_INTERNALS__.invoke=async(cmd,payload)=>{
   if(payload?.op==='flow.runs')return [{id:'live-run',flow:'Espera da live',status:sessionStorage.getItem('ended')?'CANCELLED':'WAITING',test:false,startedAt:1792911600000,durationMs:2400,cancelRequested:!!sessionStorage.getItem('cancelSent'),steps:[{index:0,kind:'wait',status:sessionStorage.getItem('ended')?'CANCELLED':'WAITING',durationMs:2400}]}];
   if(payload?.op==='flow.cancel'){sessionStorage.setItem('cancelPayload',JSON.stringify(payload.args));sessionStorage.setItem('cancelCalls',String(Number(sessionStorage.getItem('cancelCalls')||0)+1));sessionStorage.setItem('cancelSent','yes');return null}
   return original(cmd,payload);
  };
 });
 await page.locator('.sidebar').getByRole('button',{name:'Histórico',exact:true}).click();
 await page.getByRole('button',{name:/Espera da live Em espera/}).click();
 await expect(page.getByText(/Tempo total: 2.4 s/)).toBeVisible();
 await page.getByRole('button',{name:'Cancelar execução',exact:true}).click();
 await expect(page.getByRole('dialog')).toContainText('Ações já concluídas não são desfeitas');
 for(const width of [1100,1600]){
  await page.setViewportSize({width,height:1000});
  await expect.poll(()=>page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBeTruthy();
  await page.screenshot({path:join(OUT,`cancelamento-${width}.png`),fullPage:true,animations:'disabled'});
 }
 expect(await page.evaluate(()=>sessionStorage.getItem('cancelCalls'))).toBeNull();
 await page.getByRole('button',{name:'Voltar',exact:true}).click();
 await page.getByRole('button',{name:'Cancelar execução',exact:true}).click();
 await page.getByRole('button',{name:'Confirmar cancelamento',exact:true}).dblclick();
 await expect(page.getByText(/Cancelamento solicitado. Aguardando o motor/)).toBeVisible();
 await expect(page.getByRole('button',{name:/Espera da live Em espera/})).toBeVisible();
 expect(await page.evaluate(()=>JSON.parse(sessionStorage.getItem('cancelPayload')||'{}'))).toEqual({profileId:'p1',executionId:'live-run'});
 expect(await page.evaluate(()=>sessionStorage.getItem('cancelCalls'))).toBe('1');
 await page.evaluate(()=>sessionStorage.setItem('ended','yes'));
 await expect(page.getByRole('button',{name:/Espera da live Cancelada/})).toBeVisible();
 await expect(page.getByRole('button',{name:'Cancelar execução',exact:true})).toHaveCount(0);
 await expect(page.getByText(/Cancelamento solicitado. Aguardando o motor/)).toHaveCount(0);
});

test('erro ao solicitar cancelamento mantém erro e permite tentar novamente',async({page})=>{
 await mock(page);await page.goto('/');await expect(page.locator('.sidebar')).toBeVisible();await page.evaluate(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(cmd:string,payload:{op:string})=>Promise<unknown>}};
  const original=w.__TAURI_INTERNALS__.invoke;
  w.__TAURI_INTERNALS__.invoke=async(cmd,payload)=>{
   if(payload?.op==='flow.runs')return [{id:'busy',flow:'Fluxo ocupado',status:'RUNNING',test:true,context:{eventKind:'chat',eventRef:'evt-1123456789abcdef',mode:'rehearsal'},steps:[]}];
   if(payload?.op==='flow.cancel'){if(!sessionStorage.getItem('allowCancel'))throw Error('Falha ao solicitar parada');return null}
   return original(cmd,payload);
  };
 });
 await page.locator('.sidebar').getByRole('button',{name:'Histórico',exact:true}).click();
 await page.getByRole('button',{name:/Fluxo ocupado Executando · Ensaio/}).click();
 await expect(page.getByText('Mensagem no chat · Ensaio',{exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Cancelar execução',exact:true}).click();
 await page.getByRole('button',{name:'Confirmar cancelamento',exact:true}).click();
 await expect(page.getByRole('dialog').getByRole('alert')).toHaveText('Falha ao solicitar parada');
 await page.evaluate(()=>sessionStorage.setItem('allowCancel','yes'));
 await page.getByRole('button',{name:'Confirmar cancelamento',exact:true}).click();
 await expect(page.getByRole('dialog')).toHaveCount(0);
 await expect(page.getByText(/Cancelamento solicitado. Aguardando o motor/)).toBeVisible();
});


test('execução interrompida após reiniciar preserva o diagnóstico sem oferecer cancelamento ou repetição',async({page})=>{
 await mock(page);await page.goto('/');await expect(page.locator('.sidebar')).toBeVisible();
 await page.evaluate(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(cmd:string,payload:{op:string})=>Promise<unknown>}};const original=w.__TAURI_INTERNALS__.invoke;
  w.__TAURI_INTERNALS__.invoke=async(cmd,payload)=>payload?.op==='flow.runs'?[{id:'interrupted-run',flow:'Troca antes do encerramento',status:'INTERRUPTED',test:false,startedAt:1792911600000,durationMs:3200,steps:[{index:0,kind:'wait',status:'INTERRUPTED',durationMs:3200},{index:1,kind:'chat',status:'NOT_EXECUTED'}],interruptionNote:'O aplicativo encerrou sem confirmar o resultado final. Efeitos externos podem ter ocorrido. Nenhuma ação foi repetida.'}]:original(cmd,payload);
 });
 await page.locator('.sidebar').getByRole('button',{name:'Histórico',exact:true}).click();
 await page.getByLabel('Mostrar',{exact:true}).selectOption('failed');
 await page.getByRole('button',{name:/Troca antes do encerramento Interrompida/}).click();
 await expect(page.getByText(/Resultado não confirmado ·/)).toBeVisible();await expect(page.getByText(/Não iniciada ·/)).toBeVisible();
 await expect(page.getByText(/Efeitos externos podem ter ocorrido/)).toBeVisible();
 await expect(page.getByRole('button',{name:'Cancelar execução',exact:true})).toHaveCount(0);
 for(const width of [1100,1600]){await page.setViewportSize({width,height:1000});await expect.poll(()=>page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBeTruthy();await page.screenshot({path:join(OUT,`historico-persistente-${width}.png`),fullPage:true,animations:'disabled'});}
 const ops=await page.evaluate(()=>(window as unknown as {__ux:{ops:string[]}}).__ux.ops);expect(ops.filter(x=>['flow.test','flow.cancel','flow.rehearse'].includes(x))).toEqual([]);
});
