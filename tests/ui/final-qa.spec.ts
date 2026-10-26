import {test,expect} from '@playwright/test';
import {join} from 'path';
import {profile,flows,mock,OUT} from './ux-helpers';

test('etapa reutilizável usa o inspetor atual, vincula e reabre sem copiar o original',async({page})=>{
 await page.addInitScript(arg=>localStorage.setItem('botlive-preview',JSON.stringify({profiles:[arg.profile],flows:[{...arg.flow,name:'Reuso',actions:[arg.flow.actions[0]]}],notes:{},presets:[],theme:'dark',modules:{}})),{profile,flow:flows[0]});
 await page.goto('/');await page.locator('.sidebar').getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Editar Reuso',exact:true}).click();await page.locator('.react-flow__node[data-id="action-0"]').click();
 await page.getByRole('button',{name:'Guardar etapa reutilizável',exact:true}).click();
 await page.getByLabel('Nome da etapa reutilizável',{exact:true}).fill('Saudação compartilhada');
 await page.getByRole('button',{name:'Salvar e vincular etapa',exact:true}).click();
 await expect(page.getByLabel('Etapa da biblioteca',{exact:true})).toHaveValue(/.+/);
 for(const width of [1100,1600]){await page.setViewportSize({width,height:1000});await expect.poll(()=>page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBeTruthy();await page.screenshot({path:join(OUT,`etapa-reutilizavel-${width}.png`),fullPage:true,animations:'disabled'});}
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByRole('dialog')).toHaveCount(0);await page.getByRole('button',{name:'Editar Reuso',exact:true}).click();await page.locator('.react-flow__node[data-id="action-0"]').click();
 await expect(page.getByLabel('Etapa da biblioteca',{exact:true})).toHaveValue(/.+/);
 await page.getByRole('button',{name:'Excluir original da biblioteca',exact:true}).click();await page.getByRole('button',{name:'Confirmar exclusão',exact:true}).click();
 await expect(page.getByRole('dialog',{name:'Excluir etapa reutilizável?'}).getByRole('alert')).toContainText('usada por um fluxo');
 await page.getByRole('dialog',{name:'Excluir etapa reutilizável?'}).getByRole('button',{name:'Cancelar',exact:true}).click();
 await page.getByRole('button',{name:'Editar uma cópia no inspetor',exact:true}).click();
 await expect(page.getByLabel('Tipo de etapa',{exact:true})).toHaveValue('chat');
 await page.getByRole('button',{name:'Guardar etapa reutilizável',exact:true}).click();await page.getByLabel('Guardar como',{exact:true}).selectOption({label:'Atualizar: Saudação compartilhada'});
 await expect(page.getByText('Atualizar altera todas as referências deste perfil nos próximos disparos.',{exact:true})).toBeVisible();await page.getByRole('button',{name:'Salvar e vincular etapa',exact:true}).click();
 const stored=await page.evaluate(()=>JSON.parse(localStorage.getItem('botlive-preview')||'{}'));
 expect(stored.flows[0].actions[0].kind).toBe('action.ref');expect(Object.values(stored.modules)[0]).toHaveLength(1);
});

test('central substitui o roteiro repetido, mostra pendências e falha sem executar serviços',async({page})=>{
 await mock(page);await page.goto('/');await expect(page.getByRole('heading',{name:'Central da live',exact:true})).toBeVisible();
 await expect(page.getByRole('heading',{name:'Comece por aqui',exact:true})).toHaveCount(0);
 await page.evaluate(()=>{
  const w=window as unknown as {__TAURI_INTERNALS__:{invoke:(cmd:string,payload:{op:string})=>Promise<unknown>}};const original=w.__TAURI_INTERNALS__.invoke;
  w.__TAURI_INTERNALS__.invoke=async(cmd,payload)=>{if(payload?.op==='liveConsole.get'){if(sessionStorage.getItem('consoleFail'))throw Error('Consulta interrompida');return {channel:'online',audio:{clients:1},voice:{phase:'connected',restorePending:true,testActive:false},voiceQueue:{activeName:null,waiting:2,pendingRestore:true},obs:{temporary:1,pendingRestore:1},running:2,issues:1}}return original(cmd,payload)};
 });
 await page.getByRole('button',{name:'Atualizar estado',exact:true}).click();await expect(page.getByText('Restauração precisa de atenção',{exact:true})).toBeVisible();
 for(const width of [1100,1600]){await page.setViewportSize({width,height:1000});await expect.poll(()=>page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBeTruthy();await page.screenshot({path:join(OUT,`central-live-${width}.png`),fullPage:true,animations:'disabled'});}
 await page.evaluate(()=>sessionStorage.setItem('consoleFail','yes'));await page.getByRole('button',{name:'Atualizar estado',exact:true}).click();await expect(page.getByRole('alert')).toContainText('desatualizados');
 await page.getByRole('button',{name:'Recuperar na tela Voicemod',exact:true}).click();await expect(page.locator('main h1')).toHaveText('Voicemod');
 const ops=await page.evaluate(()=>(window as unknown as {__ux:{ops:string[]}}).__ux.ops);expect(ops.filter(op=>/\.(save|execute|connect|test|stop)$/.test(op))).toEqual([]);
});
