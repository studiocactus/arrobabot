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
const showAll=async(page:Page)=>{await page.locator('.react-flow__controls-fitview').click();await page.waitForTimeout(350)};
const zoomScale=(page:Page)=>page.evaluate(()=>{const el=document.querySelector('.react-flow__viewport')as HTMLElement|null;const m=el?/scale\(([\d.]+)\)/.exec(el.style.transform):null;return m?Number(m[1]):1});
type Box={id:string;x:number;y:number;w:number;h:number};
 /** Mede todos os blocos num único quadro, para comparar sobreposição e alinhamento. */
 async function boxes(page:Page):Promise<Box[]>{
  return page.locator('.react-flow__node').evaluateAll(els=>els.map(e=>{const r=(e as HTMLElement).getBoundingClientRect();return {id:e.getAttribute('data-id')||'',x:r.x,y:r.y,w:r.width,h:r.height}}));
 }
 const disjoint=(list:Box[])=>list.every((a,i)=>list.every((b,j)=>i>=j||a.x<b.x+b.w||b.x<a.x+a.w||a.y<b.y+b.h||b.y<a.y+a.h));
 const transforms=(page:Page)=>page.evaluate(()=>Array.from(document.querySelectorAll('.react-flow__node')).map(e=>({id:e.getAttribute('data-id')||'',style:e.getAttribute('style')||''})));
 async function savedFlow(page:Page,name:string){
  return page.evaluate((flowName)=>{
   const store=JSON.parse(localStorage.getItem('botlive-preview')||'{"flows":[]}');
   const f=(store.flows||[]).find((x:{name:string})=>x.name===flowName);
   if(!f)return null;
   return {
    kinds:(f.actions||[]).map((a:{kind:string})=>a.kind),
    actions:(f.actions||[]).map((a:{kind:string;condVar?:string;condValue?:string;condFalse?:string;text?:string})=>({kind:a.kind,condVar:a.condVar||'',condValue:a.condValue||'',condFalse:a.condFalse||'',text:a.text||''})),
    nodes:(f.layout?.nodes||[]).map((n:{id:string;position:{x:number;y:number};data?:{label?:string}})=>({id:n.id,x:Math.round(n.position.x),y:Math.round(n.position.y),label:n.data?.label||''})),
    edges:(f.layout?.edges||[]).map((e:{source:string;target:string})=>[e.source,e.target])
   };
  },name);
 }

test('Adicionar etapa conecta ao fim da cadeia, configura na hora e mantém a sequência',async({page})=>{
 const errors=trackErrors(page);
 await createProfile(page,'Adicionar ao fim');
 await newFlow(page,'Fluxo adicionar');
 const add=page.getByRole('button',{name:'Adicionar etapa',exact:true});
 const antes=await zoomScale(page);

 // critério 1: uma etapa nova, exatamente a conexão esperada e a configuração aberta na hora
 await add.click();
 await expect(page.getByRole('combobox',{name:'Tipo de etapa',exact:true})).toBeVisible();
 await expect(page.locator('.react-flow__node')).toHaveCount(3);
 await expect(page.locator('.react-flow__edge')).toHaveCount(2);
 await page.waitForTimeout(450);

 // critério 9: a etapa nova não fica sobreposta e o zoom não é redefinido
 const primeiro=await boxes(page);
 expect(disjoint(primeiro)).toBe(true);
 expect(await zoomScale(page)).toBe(antes);

 // critério 2: adicionar repetidamente produz a sequência esperada, sem conexão duplicada
 await add.click();
 await add.click();
 await page.waitForTimeout(500);
 await expect(page.locator('.react-flow__node')).toHaveCount(5);
 await expect(page.locator('.react-flow__edge')).toHaveCount(4);
 const todos=await boxes(page);
 expect(disjoint(todos)).toBe(true);
 expect(await zoomScale(page)).toBe(antes);

 // critério 6: Subir/Descer continuam com os limites depois de adicionar
 await showAll(page);
 const menuBtn=page.getByRole('button',{name:'Opções da etapa',exact:true});
 const subir=page.getByRole('menuitem',{name:'Subir',exact:true});
 const descer=page.getByRole('menuitem',{name:'Descer',exact:true});
 const acoes=page.locator('.react-flow__node').filter({hasText:'Enviar mensagem'});
 await acoes.nth(0).click();
 await menuBtn.click();
 await expect(subir).toBeDisabled();
 await expect(descer).toBeEnabled();
 await page.keyboard.press('Escape');
 await acoes.nth(1).click();
 await menuBtn.click();
 await expect(subir).toBeEnabled();
 await expect(descer).toBeEnabled();
 await page.keyboard.press('Escape');
 await acoes.nth(3).click();
 await menuBtn.click();
 await expect(subir).toBeEnabled();
 await expect(descer).toBeDisabled();
 await page.keyboard.press('Escape');
 expect(await isFocused(menuBtn)).toBe(true);

 // critério 5: salvar e reabrir mantém a ordem, os dados e as conexões
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();
 const saved=await savedFlow(page,'Fluxo adicionar');
 expect(saved).not.toBeNull();
 expect(saved!.kinds).toEqual(['chat','chat','chat','chat']);
 expect(saved!.nodes).toHaveLength(5);
 expect(saved!.edges).toHaveLength(4);
 expect(saved!.edges[0]).toEqual(['trigger','action-0']);
 for(let i=0;i+1<saved!.edges.length;i++)expect(saved!.edges[i][1]).toBe(saved!.edges[i+1][0]);
 const pares=saved!.edges.map(e=>e.join('>'));
 expect(new Set(pares).size).toBe(pares.length);

 await page.getByRole('button',{name:'Editar Fluxo adicionar',exact:true}).click();
 await expect(page.locator('.react-flow__node')).toHaveCount(5);
 await expect(page.locator('.react-flow__edge')).toHaveCount(4);
 expect(disjoint(await boxes(page))).toBe(true);
 expect(errors).toEqual([]);
});

test('Inserir etapa depois troca só a conexão, preserva dados e cabe na última etapa',async({page})=>{
 const errors=trackErrors(page);
 await createProfile(page,'Inserir no meio');
 await newFlow(page,'Fluxo inserir');
 const add=page.getByRole('button',{name:'Adicionar etapa',exact:true});
 const menuBtn=page.getByRole('button',{name:'Opções da etapa',exact:true});

 // etapa 1: condição com "Pular a próxima etapa"; etapa 2: espera, já conectada
 await add.click();
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('condition');
 await page.getByLabel(/^Variável: /).click();
 await page.getByLabel('Buscar variável').fill('reward.cost');
 await page.getByRole('option',{name:/Custo da recompensa \(ponto\)/}).click();
 await page.getByRole('combobox',{name:'Operador',exact:true}).selectOption('greater_or_equal');
 await page.getByLabel('Valor esperado',{exact:true}).fill('5000');
 await page.getByRole('combobox',{name:'Se falso',exact:true}).selectOption('skip');
 await add.click();
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('wait');
 await page.waitForTimeout(500);
 await expect(page.locator('.react-flow__edge')).toHaveCount(3);

 // critério 3: antes chat → condição → espera; depois chat → condição → nova → espera
 await showAll(page);
 const condNode=page.locator('.react-flow__node').filter({hasText:'Custo da recompensa'});
 const waitNode=page.locator('.react-flow__node').filter({hasText:'3.0 s'});
 await condNode.click();
 await menuBtn.click();
 await page.getByRole('menuitem',{name:'Inserir etapa depois',exact:true}).click();
 await expect(page.locator('.react-flow__node')).toHaveCount(5);
 await expect(page.locator('.react-flow__edge')).toHaveCount(4);
 // a etapa inserida fica selecionada, com a configuração aberta
 await expect(page.getByRole('combobox',{name:'Tipo de etapa',exact:true})).toHaveValue('chat');
 await page.waitForTimeout(500);
 expect(disjoint(await boxes(page))).toBe(true);

 // critério 10 e 5: a condição mantém "Pular a próxima etapa" e a ordem salva reflete a inserção
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();
 const saved=await savedFlow(page,'Fluxo inserir');
 expect(saved).not.toBeNull();
 expect(saved!.kinds).toEqual(['chat','condition','chat','wait']);
 const cond=saved!.actions.find(a=>a.kind==='condition')!;
 expect(cond.kind).toBe('condition');
 expect(cond.condVar).toBe('reward.cost');
 expect(cond.condValue).toBe('5000');
 expect(cond.condFalse).toBe('skip');
 const condId=saved!.nodes.find(n=>n.label.includes('Custo'))!.id;
 const waitId=saved!.nodes.find(n=>n.label.includes('3.0 s'))!.id;
 const novaId=saved!.nodes.map(n=>n.id).find(id=>!['trigger','action-0',condId,waitId].includes(id))!;
 const hasPair=(a:string,b:string)=>saved!.edges.some(e=>e[0]===a&&e[1]===b);
 expect(saved!.edges).toHaveLength(4);
 expect(hasPair(condId,novaId)).toBe(true);
 expect(hasPair(novaId,waitId)).toBe(true);
 expect(hasPair(condId,waitId)).toBe(false);

 // reabre: a ordem, os dados e as conexões continuam
 await page.getByRole('button',{name:'Editar Fluxo inserir',exact:true}).click();
 await expect(page.locator('.react-flow__node')).toHaveCount(5);
 await expect(page.locator('.react-flow__edge')).toHaveCount(4);
 await expect(page.locator('.react-flow__node').filter({hasText:'Custo da recompensa'})).toHaveCount(1);

 // critério 4: inserir na última etapa funciona
 await showAll(page);
 await waitNode.click();
 await menuBtn.click();
 await page.getByRole('menuitem',{name:'Inserir etapa depois',exact:true}).click();
 await expect(page.locator('.react-flow__node')).toHaveCount(6);
 await expect(page.locator('.react-flow__edge')).toHaveCount(5);
 await expect(page.getByRole('combobox',{name:'Tipo de etapa',exact:true})).toHaveValue('chat');
 await page.waitForTimeout(500);
 expect(disjoint(await boxes(page))).toBe(true);

 // critério 6: os limites continuam corretos após as inserções
 await showAll(page);
 await page.locator('.react-flow__node[data-id="action-0"]').click();
 await menuBtn.click();
 await expect(page.getByRole('menuitem',{name:'Subir',exact:true})).toBeDisabled();
 await expect(page.getByRole('menuitem',{name:'Descer',exact:true})).toBeEnabled();
 await page.keyboard.press('Escape');

 // critério 12: ajuda e minimapa seguem funcionando
 const help=page.getByRole('button',{name:'Ajuda do editor de fluxos',exact:true});
 await help.click();
 await expect(page.locator('.flow-help')).toContainText('A ordem das etapas segue as conexões entre os blocos');
 await page.keyboard.press('Escape');
 await expect(page.locator('.flow-help')).toHaveCount(0);
 const mini=page.getByRole('button',{name:'Minimapa',exact:true});
 await mini.click();
 await expect(page.locator('.react-flow__minimap')).toBeVisible();
 await mini.click();
 await expect(page.locator('.react-flow__minimap')).toHaveCount(0);
 expect(errors).toEqual([]);
});

test('Fluxo inválido: adicionar explica sem perder blocos e a inserção some do menu',async({page})=>{
 const errors=trackErrors(page);
 await createProfile(page,'Fluxo inválido');
 await newFlow(page,'Fluxo quebrado');
 const add=page.getByRole('button',{name:'Adicionar etapa',exact:true});
 await add.click();
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('wait');
 await page.waitForTimeout(450);
 await expect(page.locator('.react-flow__edge')).toHaveCount(2);

 // remover um bloco deixa o gatilho e a espera soltos
 await page.locator('.react-flow__node[data-id="action-0"]').click();
 await page.getByRole('button',{name:'Remover bloco',exact:true}).click();
 await expect(page.locator('.react-flow__node')).toHaveCount(2);
 await expect(page.locator('.react-flow__edge')).toHaveCount(0);

 // critério 8: adicionar explica o problema em português e não cria nem reorganiza nada
 await add.click();
 await expect(page.locator('.inline-error')).toContainText('Conecte todos os blocos');
 await expect(page.locator('.react-flow__node')).toHaveCount(2);
 await expect(page.locator('.react-flow__edge')).toHaveCount(0);
 await expect(page.locator('.react-flow__node').filter({hasText:'3.0 s'})).toHaveCount(1);

 // fora de uma única cadeia válida, a inserção não é oferecida e os limites ficam travados
 const waitNode=page.locator('.react-flow__node').filter({hasText:'3.0 s'});
 await showAll(page);
 await waitNode.click();
 const menuBtn=page.getByRole('button',{name:'Opções da etapa',exact:true});
 await menuBtn.click();
 await expect(page.getByRole('menuitem',{name:'Subir',exact:true})).toBeDisabled();
 await expect(page.getByRole('menuitem',{name:'Descer',exact:true})).toBeDisabled();
 await expect(page.getByRole('menuitem',{name:'Inserir etapa depois',exact:true})).toHaveCount(0);
 await page.keyboard.press('Escape');
 await expect(page.getByRole('menu',{name:'Opções da etapa',exact:true})).toHaveCount(0);
 expect(await isFocused(menuBtn)).toBe(true);
 expect(errors).toEqual([]);
});

test('Conexão manual continua possível: religar à mão volta a valer cadeia válida',async({page})=>{
 const errors=trackErrors(page);
 await createProfile(page,'Conexão manual');
 await newFlow(page,'Fluxo manual');
 const add=page.getByRole('button',{name:'Adicionar etapa',exact:true});
 const menuBtn=page.getByRole('button',{name:'Opções da etapa',exact:true});
 await add.click();
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('wait');
 const waitNode=page.locator('.react-flow__node').filter({hasText:'3.0 s'});
 await page.waitForTimeout(450);

 // some com um bloco e religa o gatilho à espera arrastando dos pontos de conexão
 await page.locator('.react-flow__node[data-id="action-0"]').click();
 await page.getByRole('button',{name:'Remover bloco',exact:true}).click();
 await expect(page.locator('.react-flow__edge')).toHaveCount(0);
 await showAll(page);
 await link(page,page.locator('.react-flow__node[data-id="trigger"]'),waitNode);
 await expect(page.locator('.react-flow__edge')).toHaveCount(1);

 // a religação manual devolve a cadeia válida: inserir volta ao menu e adicionar funciona
 await waitNode.click();
 await menuBtn.click();
 await expect(page.getByRole('menuitem',{name:'Inserir etapa depois',exact:true})).toBeVisible();
 await page.keyboard.press('Escape');
 await add.click();
 await page.waitForTimeout(450);
 await expect(page.locator('.react-flow__edge')).toHaveCount(2);

 // salva e reabre com a conexão feita à mão preservada
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();
 const saved=await savedFlow(page,'Fluxo manual');
 expect(saved).not.toBeNull();
 expect(saved!.kinds).toEqual(['wait','chat']);
 expect(saved!.edges).toHaveLength(2);
 expect(saved!.edges[0][0]).toBe('trigger');
 expect(saved!.edges[0][1]).toBe(saved!.edges[1][0]);
 await page.getByRole('button',{name:'Editar Fluxo manual',exact:true}).click();
 await expect(page.locator('.react-flow__node')).toHaveCount(3);
 await expect(page.locator('.react-flow__edge')).toHaveCount(2);
 expect(errors).toEqual([]);
});

test('Layout horizontal antigo: abrir não muda o desenho e a inserção respeita o eixo',async({page})=>{
 const errors=trackErrors(page);
 await createProfile(page,'Layout antigo');
 await newFlow(page,'Fluxo horizontal');
 const add=page.getByRole('button',{name:'Adicionar etapa',exact:true});
 await add.click();
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('wait');
 await page.waitForTimeout(450);
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();

 // o fluxo salvo ganha o desenho antigo: fila horizontal, tudo na mesma altura
 const waitId=await page.evaluate(()=>{
  const store=JSON.parse(localStorage.getItem('botlive-preview')||'{"flows":[]}');
  const f=(store.flows||[]).find((x:{name:string})=>x.name==='Fluxo horizontal');
  const ids=(f.layout?.nodes||[]).map((n:{id:string})=>n.id);
  f.layout.nodes=(f.layout.nodes||[]).map((n:{id:string;position:{x:number;y:number}},i:number)=>({...n,position:{x:60+i*280,y:200}}));
  localStorage.setItem('botlive-preview',JSON.stringify(store));
  return ids.find(id=>id!=='trigger'&&id!=='action-0')!;
 });
 await page.reload();
 await page.getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Editar Fluxo horizontal',exact:true}).click();
 await page.waitForTimeout(450);

 // critério 7: abrir preserva exatamente as posições salvas (três blocos na mesma linha)
 const antes=await transforms(page);
 expect(antes).toHaveLength(3);
 for(const n of antes)expect(n.style).toMatch(/translate\(\d+px, 200px\)/);
 expect(antes.map(n=>Number(/translate\((\d+)px/.exec(n.style)![1]))).toEqual([60,340,620]);
 await expect(page.locator('.react-flow__edge')).toHaveCount(2);

 // critério 7 e 9: inserir no meio mantém a fila na horizontal, só abrindo o espaço necessário
 await page.locator('.react-flow__node[data-id="action-0"]').click();
 await page.getByRole('button',{name:'Opções da etapa',exact:true}).click();
 await page.getByRole('menuitem',{name:'Inserir etapa depois',exact:true}).click();
 await expect(page.locator('.react-flow__node')).toHaveCount(4);
 await page.waitForTimeout(500);
 const depois=await transforms(page);
 for(const n of depois)expect(n.style).toMatch(/translate\(\d+px, 200px\)/);
 const xs=depois.map(n=>Number(/translate\((\d+)px/.exec(n.style)![1]));
 expect(xs[0]).toBe(60);
 expect(xs[1]).toBe(340);
 const novaX=xs.find(x=>x>340&&x<xs[2])!;
 expect(novaX).toBeDefined();
 expect(xs[2]).toBeGreaterThan(620);
 expect(disjoint(await boxes(page))).toBe(true);

 // salvar e reabrir mantém o desenho horizontal intacto
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Editar Fluxo horizontal',exact:true}).click();
 await page.waitForTimeout(450);
 const salvo=await savedFlow(page,'Fluxo horizontal');
 expect(salvo!.nodes.find(n=>n.id===waitId)!.y).toBe(200);
 for(const n of await transforms(page))expect(n.style).toMatch(/translate\(\d+px, 200px\)/);
 expect(await page.locator('.react-flow__edge').count()).toBe(3);
 expect(errors).toEqual([]);
});
