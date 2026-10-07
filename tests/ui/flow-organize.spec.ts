import {test,expect,type Locator,type Page} from '@playwright/test';

async function createProfile(page:Page,name:string){
 await page.goto('/');
 await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();
 await page.getByLabel('Nome do perfil',{exact:true}).fill(name);
 await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();
 await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
 await page.getByRole('button',{name:'Automações',exact:true}).click();
}
async function newFlow(page:Page,name:string){
 await page.getByRole('button',{name:'Novo fluxo',exact:true}).click();
 await page.getByLabel('Nome do fluxo',{exact:true}).fill(name);
 await page.getByLabel('Texto que dispara',{exact:true}).fill('!'+name.toLowerCase().replace(/[^a-z0-9]/g,''));
}
async function link(page:Page,from:Locator,to:Locator){
 const fromBox=await from.locator('.react-flow__handle.source').boundingBox();
 const toBox=await to.locator('.react-flow__handle.target').boundingBox();
 if(!fromBox||!toBox)throw new Error('ponto de conexão fora da tela');
 await page.mouse.move(fromBox.x+fromBox.width/2,fromBox.y+fromBox.height/2);
 await page.mouse.down();
 await page.mouse.move(fromBox.x+fromBox.width/2,fromBox.y+fromBox.height/2+6,{steps:4});
 await page.mouse.move(toBox.x+toBox.width/2,toBox.y+toBox.height/2,{steps:12});
 await page.waitForTimeout(150);
 await page.mouse.up();
 await page.waitForTimeout(100);
}
function trackErrors(page:Page){const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));return errors}
const isFocused=(l:Locator)=>l.evaluate(el=>document.activeElement===el);

test('menu Opções da etapa reordena na cadeia com limites, devolve o foco e não muda o fluxo ao abrir',async({page})=>{
 const errors=trackErrors(page);
 await createProfile(page,'Menu da etapa');
 await newFlow(page,'Fluxo menu');
 const menuBtn=page.getByRole('button',{name:'Opções da etapa',exact:true});
 const subir=page.getByRole('menuitem',{name:'Subir',exact:true});
 const descer=page.getByRole('menuitem',{name:'Descer',exact:true});

 // critério 1: o gatilho não oferece menu, a etapa sim
 await page.locator('.react-flow__node[data-id="trigger"]').click();
 await expect(menuBtn).toHaveCount(0);
 await page.locator('.react-flow__node[data-id="action-0"]').click();
 await expect(menuBtn).toBeVisible();

 // etapa desconectada não reorganiza
 await page.getByRole('button',{name:'Adicionar etapa',exact:true}).click();
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('wait');
 await expect(page.locator('.react-flow__node').filter({hasText:'3.0 s'})).toHaveCount(1);
 await menuBtn.click();
 await expect(subir).toBeDisabled();
 await expect(descer).toBeDisabled();
 await page.keyboard.press('Escape');
 await expect(page.getByRole('menu',{name:'Opções da etapa',exact:true})).toHaveCount(0);
 expect(await isFocused(menuBtn)).toBe(true);

 // liga gatilho → chat → espera
 await page.locator('.react-flow__controls-fitview').click();
 await page.waitForTimeout(300);
 await link(page,page.locator('.react-flow__node[data-id="action-0"]'),page.locator('.react-flow__node').filter({hasText:'3.0 s'}));
 await expect(page.locator('.react-flow__edge')).toHaveCount(2);

 // critério 2: limites da cadeia (primeira não sobe, última não desce)
 const waitNode=page.locator('.react-flow__node').filter({hasText:'3.0 s'});
 await waitNode.click();
 await menuBtn.click();
 await expect(subir).toBeEnabled();
 await expect(descer).toBeDisabled();
 await page.keyboard.press('Escape');
 expect(await isFocused(menuBtn)).toBe(true);
 await page.locator('.react-flow__node[data-id="action-0"]').click();
 await menuBtn.click();
 await expect(subir).toBeDisabled();
 await expect(descer).toBeEnabled();

 // critério 5: Escape fecha só o menu e o editor continua aberto
 await page.keyboard.press('Escape');
 await expect(page.getByRole('menu',{name:'Opções da etapa',exact:true})).toHaveCount(0);
 expect(await isFocused(menuBtn)).toBe(true);
 await expect(page.getByRole('button',{name:'Salvar fluxo',exact:true})).toBeVisible();

 // critério 4: abrir e fechar pelo clique fora não altera o fluxo
 const nodes=await page.locator('.react-flow__node').count();
 const edges=await page.locator('.react-flow__edge').count();
 await menuBtn.click();
 await page.locator('.canvas').click({position:{x:4,y:4}});
 await expect(page.getByRole('menu',{name:'Opções da etapa',exact:true})).toHaveCount(0);
 expect(await page.locator('.react-flow__node').count()).toBe(nodes);
 expect(await page.locator('.react-flow__edge').count()).toBe(edges);
 await expect(page.locator('.inline-error')).toHaveCount(0);

 // Descer na primeira etapa troca as posições e devolve o foco ao botão
 await menuBtn.click();
 await descer.click();
 await expect(page.getByRole('menu',{name:'Opções da etapa',exact:true})).toHaveCount(0);
 expect(await isFocused(menuBtn)).toBe(true);
 await page.waitForTimeout(250);
 const waitBox=await waitNode.boundingBox();
 const chatBox=await page.locator('.react-flow__node[data-id="action-0"]').boundingBox();
 expect(waitBox!.y).toBeLessThan(chatBox!.y);
 await expect(page.locator('.react-flow__edge')).toHaveCount(2);

 // agora o chat é a última etapa: não desce mais
 await page.locator('.react-flow__node[data-id="action-0"]').click();
 await menuBtn.click();
 await expect(subir).toBeEnabled();
 await expect(descer).toBeDisabled();
 await page.keyboard.press('Escape');
 expect(errors).toEqual([]);
});

test('reordenar com o menu, salvar e reabrir preserva a ordem das ações e as conexões',async({page})=>{
 const errors=trackErrors(page);
 await createProfile(page,'Reordena de verdade');
 await newFlow(page,'Fluxo reordena');
 await page.getByRole('button',{name:'Adicionar etapa',exact:true}).click();
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('wait');
 const waitNode=page.locator('.react-flow__node').filter({hasText:'3.0 s'});
 await page.getByRole('button',{name:'Adicionar etapa',exact:true}).click();
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('punish');
 const punishNode=page.locator('.react-flow__node').filter({hasText:'Silenciar por um tempo'});

 await page.locator('.react-flow__controls-fitview').click();
 await page.waitForTimeout(300);
 await link(page,page.locator('.react-flow__node[data-id="action-0"]'),waitNode);
 await link(page,waitNode,punishNode);
 await expect(page.locator('.react-flow__edge')).toHaveCount(3);

 // a etapa do meio desce: a punição passa para cima da espera
 await waitNode.click();
 await page.getByRole('button',{name:'Opções da etapa',exact:true}).click();
 await page.getByRole('menuitem',{name:'Descer',exact:true}).click();
 await page.waitForTimeout(250);
 const waitBox=await waitNode.boundingBox();
 const punishBox=await punishNode.boundingBox();
 expect(punishBox!.y).toBeLessThan(waitBox!.y);

 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();

 const saved=await page.evaluate(()=>{
  const flows=JSON.parse(localStorage.getItem('botlive-preview')||'{"flows":[]}').flows||[];
  const f=flows.find((x:{name:string})=>x.name==='Fluxo reordena');
  if(!f)return null;
  return {kinds:f.actions.map((a:{kind:string})=>a.kind),
   layoutNodes:(f.layout?.nodes||[]).length,
   edges:(f.layout?.edges||[]).map((e:{source:string;target:string})=>[e.source,e.target])};
 });
 expect(saved).not.toBeNull();
 // critério 3: ordem salva segue a cadeia e as conexões continuam ligadas em sequência
 expect(saved!.kinds).toEqual(['chat','punish','wait']);
 expect(saved!.layoutNodes).toBe(4);
 expect(saved!.edges).toHaveLength(3);
 expect(saved!.edges[0]).toEqual(['trigger','action-0']);
 expect(saved!.edges[1][0]).toBe('action-0');
 expect(saved!.edges[2][0]).toBe(saved!.edges[1][1]);

 // reabre: a ordem visual e as três conexões continuam corretas
 await page.getByRole('button',{name:'Editar Fluxo reordena',exact:true}).click();
 await expect(page.locator('.react-flow__edge')).toHaveCount(3);
 await page.waitForTimeout(350);
 const waitBox2=await waitNode.boundingBox();
 const punishBox2=await punishNode.boundingBox();
 expect(punishBox2!.y).toBeLessThan(waitBox2!.y);
 await expect(waitNode).toHaveCount(1);
 await expect(punishNode).toHaveCount(1);
 expect(errors).toEqual([]);
});

test('minimapa começa recolhido, abre compacto e mantém a preferência sem tocar no fluxo',async({page})=>{
 const errors=trackErrors(page);
 await createProfile(page,'Minimapa');
 await newFlow(page,'Fluxo mapa');
 const toggle=page.getByRole('button',{name:'Minimapa',exact:true});

 // começa recolhido, com estado acessível
 await expect(page.locator('.react-flow__minimap')).toHaveCount(0);
 await expect(toggle).toHaveAttribute('aria-expanded','false');

 // dá zoom e seleciona uma etapa antes de alternar
 await page.locator('.react-flow__controls-zoomout').click();
 await page.locator('.react-flow__node[data-id="action-0"]').click();
 await expect(page.getByRole('button',{name:'Opções da etapa',exact:true})).toBeVisible();
 const snap=async()=>page.evaluate(()=>({
  viewport:(document.querySelector('.react-flow__viewport')as HTMLElement).style.transform,
  node:(document.querySelector('.react-flow__node')as HTMLElement).style.transform,
  edges:document.querySelectorAll('.react-flow__edge').length
 }));
 const before=await snap();

 // critério 7: mostrar não muda zoom, posições, conexões nem seleção
 await toggle.click();
 await expect(toggle).toHaveAttribute('aria-expanded','true');
 const mini=page.locator('.react-flow__minimap');
 await expect(mini).toBeVisible();
 const miniBox=(await mini.boundingBox())!;
 expect(miniBox.width).toBeLessThanOrEqual(160);
 expect(miniBox.height).toBeLessThanOrEqual(110);
 const controlsBox=(await page.locator('.react-flow__controls').boundingBox())!;
 const overlap=!(miniBox.x+miniBox.width<=controlsBox.x||controlsBox.x+controlsBox.width<=miniBox.x||miniBox.y+miniBox.height<=controlsBox.y||controlsBox.y+controlsBox.height<=miniBox.y);
 expect(overlap).toBe(false);
 expect(await snap()).toEqual(before);
 await expect(page.getByRole('button',{name:'Opções da etapa',exact:true})).toBeVisible();
 await expect(page.locator('.react-flow__controls')).toBeVisible();

 // a preferência fica fora do layout do fluxo
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();
 const stored=await page.evaluate(()=>{
  const flows=JSON.parse(localStorage.getItem('botlive-preview')||'{"flows":[]}').flows||[];
  const f=flows.find((x:{name:string})=>x.name==='Fluxo mapa');
  return {layoutKeys:Object.keys(f.layout||{}).sort(),pref:localStorage.getItem('botlive-editor')};
 });
 expect(stored.layoutKeys).toEqual(['edges','nodes']);
 expect(JSON.parse(stored.pref||'{}')).toEqual({minimap:true});

 // critério 8: a preferência sobrevive ao fechar e reabrir o editor
 await page.getByRole('button',{name:'Editar Fluxo mapa',exact:true}).click();
 await expect(page.locator('.react-flow__minimap')).toBeVisible();
 await expect(toggle).toHaveAttribute('aria-expanded','true');
 await toggle.click();
 await expect(page.locator('.react-flow__minimap')).toHaveCount(0);
 await expect(toggle).toHaveAttribute('aria-expanded','false');
 expect(errors).toEqual([]);
});

test('ajuda compacta no lugar do banner: teclado, Escape e tela pequena',async({page})=>{
 const errors=trackErrors(page);
 await page.goto('/');
 await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();
 await page.getByLabel('Nome do perfil',{exact:true}).fill('Ajuda do editor');
 await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();
 await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
 await page.getByRole('button',{name:'Comandos',exact:true}).click();
 await page.getByRole('button',{name:'Novo comando',exact:true}).click();
 await page.getByLabel('Nome',{exact:true}).fill('Alô');
 await page.getByLabel('Comando',{exact:true}).fill('!alohelp');
 await page.getByRole('textbox',{name:'Resposta',exact:true}).fill('Oi, {{user}}!');
 await page.getByRole('button',{name:'Abrir no editor visual',exact:true}).click();

 // o banner saiu; a explicação e a nota do comando continuam disponíveis
 await expect(page.locator('.flow-intro')).toHaveCount(0);
 const help=page.getByRole('button',{name:'Ajuda do editor de fluxos',exact:true});
 await expect(help).toBeVisible();
 await help.focus();
 await page.keyboard.press('Enter');
 await expect(page.locator('.flow-help')).toBeVisible();
 await expect(page.locator('.flow-help')).toContainText('A ordem das etapas segue as conexões entre os blocos. A posição na tela não muda a execução.');
 await expect(page.locator('.flow-help')).toContainText('mantém o mesmo nome, gatilho e ativação');
 // avisos de erro não moram dentro da ajuda
 await expect(page.locator('.flow-help [role="alert"]')).toHaveCount(0);

 // critério 5: Escape fecha só a ajuda e devolve o foco
 await page.keyboard.press('Escape');
 await expect(page.locator('.flow-help')).toHaveCount(0);
 expect(await isFocused(help)).toBe(true);
 await expect(page.getByRole('button',{name:'Salvar fluxo',exact:true})).toBeVisible();

 // critério 9: em tela pequena o popover cabe na janela e não gera rolagem horizontal
 await page.setViewportSize({width:640,height:760});
 await help.click();
 const box=await page.locator('.flow-help').boundingBox();
 expect(box).not.toBeNull();
 expect(box!.x).toBeGreaterThanOrEqual(0);
 expect(box!.y).toBeGreaterThanOrEqual(0);
 expect(box!.x+box!.width).toBeLessThanOrEqual(641);
 expect(box!.y+box!.height).toBeLessThanOrEqual(761);
 const overflow=await page.evaluate(()=>{const el=document.querySelector('.flow-editor')as HTMLElement;return el.scrollWidth-el.clientWidth});
 expect(overflow).toBeLessThanOrEqual(1);
 await page.keyboard.press('Escape');
 await expect(page.locator('.flow-help')).toHaveCount(0);
 expect(await isFocused(help)).toBe(true);
 await expect(page.getByRole('button',{name:'Salvar fluxo',exact:true})).toBeVisible();
 expect(errors).toEqual([]);
});
