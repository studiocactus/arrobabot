import {test,expect,type Page} from '@playwright/test';
import {join} from 'node:path';
import {mock,OUT,auditPage,fmt} from './ux-helpers';

async function setup(page:Page){
 await mock(page);await page.addInitScript(()=>{
  const w=window as any,original=w.__TAURI_INTERNALS__.invoke;let saved:any[]|null=null;
  w.__TAURI_INTERNALS__.invoke=async(cmd:string,payload:any)=>{
   if(payload?.op==='flows')return saved||await original(cmd,payload);
   if(payload?.op==='flow.save'){const base=saved||await original(cmd,{op:'flows',args:{profileId:'p1'}});saved=[...base.filter((f:any)=>f.id!==payload.args.flow.id),payload.args.flow];sessionStorage.setItem('obs-saved',JSON.stringify(payload.args.flow));return payload.args.flow;}
   if(payload?.op==='obs.execute'){const calls=JSON.parse(sessionStorage.getItem('obs-calls')||'[]');calls.push(payload.args);sessionStorage.setItem('obs-calls',JSON.stringify(calls));return 'Operação confirmada pelo OBS';}
   return original(cmd,payload);
  };
 });await page.goto('/');
}
async function choose(page:Page,name:string){await page.getByRole('button',{name:'Receitas prontas',exact:true}).click();await page.getByRole('button',{name,exact:true}).click();await page.getByRole('button',{name:/Configurar receita|Editar configuração existente/,exact:true}).click();}

test('OBS: fontes compatíveis, filtros e teste real explícito no mesmo painel',async({page})=>{
 await setup(page);await page.getByRole('button',{name:'OBS Studio',exact:true}).click();
 await expect(page.getByRole('combobox',{name:'Entrada de áudio',exact:true}).locator('option')).not.toHaveCount(1);
 const reads=await page.evaluate(()=>(window as any).__ux.ops.filter((op:string)=>op==='obs.targets').length);await page.getByRole('button',{name:'Salvar OBS',exact:true}).click();await expect.poll(()=>page.evaluate(()=>(window as any).__ux.ops.filter((op:string)=>op==='obs.targets').length)).toBeGreaterThan(reads);
 await page.getByRole('combobox',{name:'Operação',exact:true}).selectOption('set_text');await page.getByRole('combobox',{name:'Onde está a fonte?'}).selectOption('Intervalo');
 const source=page.getByRole('combobox',{name:'Fonte do OBS',exact:true});await expect(source.locator('option')).toHaveCount(2);await source.selectOption('Texto da live');
 await page.getByRole('textbox',{name:'Texto que aparecerá no OBS',exact:true}).fill('Boa noite, comunidade!');await expect(page.getByRole('textbox',{name:'Texto que aparecerá no OBS',exact:true})).toHaveValue('Boa noite, comunidade!');
 await page.getByRole('button',{name:'Executar no OBS',exact:true}).click();await expect(page.getByText('Operação confirmada pelo OBS',{exact:true})).toBeVisible();
 expect(await page.evaluate(()=>JSON.parse(sessionStorage.getItem('obs-calls')||'[]')[0])).toMatchObject({op:'set_text',scene:'Intervalo',target:'Texto da live',content:'Boa noite, comunidade!'});
 await page.getByRole('combobox',{name:'Operação',exact:true}).selectOption('filter_toggle');await page.getByRole('combobox',{name:'Fonte ou cena'}).selectOption('Texto da live');await page.getByRole('combobox',{name:'Filtro',exact:true}).selectOption('Cor');await page.getByRole('button',{name:'Executar no OBS',exact:true}).click();
 expect(await page.evaluate(()=>JSON.parse(sessionStorage.getItem('obs-calls')||'[]').at(-1))).toMatchObject({op:'filter_toggle',filter:'Cor'});
 await page.getByRole('combobox',{name:'Operação',exact:true}).selectOption('set_browser_url');await source.selectOption('Página da live');await page.getByRole('textbox',{name:'Endereço que o OBS vai abrir',exact:true}).fill('javascript:alert(1)');await page.getByRole('button',{name:'Executar no OBS',exact:true}).click();await expect(page.locator('.obs-action-test [role=status]')).toContainText('HTTP/HTTPS');expect(await page.evaluate(()=>JSON.parse(sessionStorage.getItem('obs-calls')||'[]').length)).toBe(2);
 await page.getByRole('textbox',{name:'Endereço que o OBS vai abrir',exact:true}).fill('https://example.com/overlay');
 if(await page.getByRole('button',{name:'Fechar aviso',exact:true}).isVisible())await page.getByRole('button',{name:'Fechar aviso',exact:true}).click();
 for(const width of [1100,1680]){await page.setViewportSize({width,height:1000});await page.getByRole('heading',{name:'Testar ação',exact:true}).scrollIntoViewIfNeeded();expect(fmt(await page.evaluate(auditPage))).toEqual([]);await page.screenshot({path:join(OUT,`obs-controles-${width}.png`),fullPage:true});}
});

test('receita de texto configura em formulário, salva e reabre a mesma automação',async({page})=>{
 await setup(page);await page.getByRole('button',{name:'Automações',exact:true}).click();await choose(page,'Comando atualiza texto no OBS');
 await expect(page.getByRole('heading',{name:'1. Quando acontece'})).toBeVisible();await expect(page.locator('.react-flow')).toHaveCount(0);
 await page.getByRole('combobox',{name:'Fonte do OBS',exact:true}).selectOption('Texto da live');await page.getByRole('combobox',{name:'Onde está a fonte?'}).selectOption('Grupo de alertas');await page.getByRole('combobox',{name:'Fonte do OBS',exact:true}).selectOption('Texto da live');
 await page.getByRole('textbox',{name:'Mensagem de confirmação',exact:true}).fill('Tudo certo no OBS!');
 await expect(page.getByRole('button',{name:'Testar efeito no OBS',exact:true})).toBeDisabled();await page.getByRole('textbox',{name:'Conteúdo de exemplo para testar no OBS',exact:true}).fill('Teste na fonte');await page.getByRole('button',{name:'Testar efeito no OBS',exact:true}).click();await expect(page.locator('.obs-action-test [role=status]')).toContainText('confirmada');expect(await page.evaluate(()=>JSON.parse(sessionStorage.getItem('obs-calls')!)[0].content)).toBe('Teste na fonte');
 for(const width of [1100,1680]){await page.setViewportSize({width,height:1000});await page.getByRole('dialog').evaluate(el=>{el.scrollTop=0});expect(fmt(await page.evaluate(auditPage))).toEqual([]);await page.screenshot({path:join(OUT,`obs-receita-${width}.png`),fullPage:true});await page.getByRole('button',{name:'Salvar automação',exact:true}).scrollIntoViewIfNeeded();await page.screenshot({path:join(OUT,`obs-receita-final-${width}.png`),fullPage:true});}
 await page.getByRole('button',{name:'Salvar automação',exact:true}).click();await expect(page.getByRole('dialog')).toHaveCount(0);
 const saved=await page.evaluate(()=>JSON.parse(sessionStorage.getItem('obs-saved')!));expect(saved.enabled).toBe(false);expect(saved.actions[1]).toMatchObject({kind:'obs',obsOp:'set_text',obsScene:'Grupo de alertas',obsTarget:'Texto da live',text:'{{rawInput}}'});expect(saved.actions[2].text).toBe('Tudo certo no OBS!');
 await choose(page,'Comando atualiza texto no OBS');await expect(page.getByRole('textbox',{name:'Mensagem de confirmação',exact:true})).toHaveValue('Tudo certo no OBS!');await expect(page.getByRole('combobox',{name:'Onde está a fonte?'})).toHaveValue('Grupo de alertas');await page.getByRole('button',{name:'Editar etapas no editor completo'}).click();await expect(page.locator('.react-flow')).toBeVisible();
});

test('receita de filtro exige recompensa e preserva a escolha até salvar',async({page})=>{
 await setup(page);await page.getByRole('button',{name:'Automações',exact:true}).click();await choose(page,'Resgate ativa um filtro no OBS');await page.getByRole('combobox',{name:'Fonte ou cena'}).selectOption('Texto da live');await page.getByRole('combobox',{name:'Filtro',exact:true}).selectOption('Cor');
 await page.getByRole('button',{name:'Salvar automação',exact:true}).click();await expect(page.getByRole('alert')).toContainText('recompensa específica');expect(await page.evaluate(()=>sessionStorage.getItem('obs-saved'))).toBeNull();
 await page.getByRole('combobox',{name:'Recompensa da Twitch',exact:true}).selectOption('r1');await page.getByRole('button',{name:'Salvar automação',exact:true}).click();await expect(page.getByRole('dialog')).toHaveCount(0);const saved=await page.evaluate(()=>JSON.parse(sessionStorage.getItem('obs-saved')!));expect(saved.trigger.pattern).toBe('r1');expect(saved.actions[0]).toMatchObject({obsTarget:'Texto da live',obsFilter:'Cor'});
});
