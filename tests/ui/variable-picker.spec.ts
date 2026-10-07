import {test,expect,type Page} from '@playwright/test';

async function novoFluxo(page:Page,name:string){
 await page.goto('/');
 await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();
 await page.getByLabel('Nome do perfil',{exact:true}).fill('Seletor de variáveis');
 await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();
 await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
 await page.getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Novo fluxo',exact:true}).click();
 await page.getByLabel('Nome do fluxo',{exact:true}).fill(name);
 await page.getByLabel('Texto que dispara',{exact:true}).fill('!fluxo');
 await page.locator('.react-flow__node[data-id="action-0"]').click();
}

test('seletor de variáveis busca por nome, código e categoria e mantém o identificador salvo',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await novoFluxo(page,'Condição do pedido');
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('condition');

 // busca por nome compreensível
 const trigger=page.getByLabel(/^Variável: /);
 await trigger.click();
 const busca=page.getByLabel('Buscar variável');
 await expect(busca).toBeVisible();
 await busca.fill('custo');
 await expect(page.getByRole('option',{name:/Custo da recompensa \(ponto\)/})).toBeVisible();

 // busca pelo identificador técnico: a mesma variável, uma única opção
 await busca.fill('reward.cost');
 const opcao=page.getByRole('option',{name:/Custo da recompensa \(ponto\)/});
 await expect(opcao).toBeVisible();
 await expect(opcao).toContainText('reward.cost');
 await expect(page.locator('.var-pop').getByRole('option')).toHaveCount(1);

 // seleciona com um clique, o seletor fecha e o foco volta ao botão
 await opcao.click();
 await expect(page.getByLabel('Buscar variável')).toHaveCount(0);
 await expect(trigger).toContainText('Custo da recompensa (ponto)');
 await expect(trigger).toBeFocused();

 await page.getByRole('combobox',{name:'Operador',exact:true}).selectOption('greater_or_equal');
 await page.getByLabel('Valor esperado',{exact:true}).fill('5000');
 const antes=await page.locator('.react-flow__node[data-id="trigger"]').getAttribute('style');
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();

 // reabre: identificador persistido e rótulo correto no canvas e no seletor
 await page.getByRole('button',{name:'Editar Condição do pedido',exact:true}).click();
 const salvo=await page.evaluate(()=>{
  const s=JSON.parse(localStorage.getItem('botlive-preview')||'{}');
  const f=(s.flows||[])[0];
  return (f.actions||[]).map((a:{kind:string;condVar?:string})=>({kind:a.kind,condVar:a.condVar}));
 });
 expect(salvo).toEqual([{kind:'condition',condVar:'reward.cost'}]);
 await expect(page.locator('.react-flow__node').filter({hasText:'Custo da recompensa'})).toHaveCount(1);
 await expect(page.locator('.react-flow__node').filter({hasText:'≥ 5000'})).toHaveCount(1);

 // abrir não mexe em posições nem conexões
 const depois=await page.locator('.react-flow__node[data-id="trigger"]').getAttribute('style');
 expect(depois).toBe(antes);
 await expect(page.locator('.react-flow__edge')).toHaveCount(1);

 // categoria agrupa os resultados e Escape fecha só o seletor
 await page.locator('.react-flow__node[data-id="action-0"]').click();
 await page.getByLabel(/^Variável: /).click();
 await page.getByLabel('Buscar variável').fill('Eventos');
 await expect(page.locator('.var-group')).toContainText('Eventos');
 await expect(page.getByRole('option',{name:/Custo da recompensa \(ponto\)/})).toBeVisible();
 await page.keyboard.press('Escape');
 await expect(page.getByLabel('Buscar variável')).toHaveCount(0);
 await expect(page.getByLabel('Nome do fluxo',{exact:true})).toBeVisible();
 expect(errors).toEqual([]);
});

test('seletor abre pelo teclado, seleciona com Enter e não é cortado pelo painel',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await novoFluxo(page,'Teclado do seletor');
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('condition');

 const trigger=page.getByLabel(/^Variável: /);
 await trigger.focus();
 await page.keyboard.press('Enter');
 await expect(page.getByLabel('Buscar variável')).toBeFocused();
 await page.keyboard.type('reward.cost');
 await page.keyboard.press('ArrowDown');
 await page.keyboard.press('Enter');
 await expect(trigger).toContainText('Custo da recompensa (ponto)');
 await expect(trigger).toBeFocused();

 // o popover é um portal no body: fora do painel com overflow, dentro da viewport
 await trigger.click();
 await expect(page.getByLabel('Buscar variável')).toBeVisible();
 const portal=await page.locator('.var-pop').evaluate(el=>el.parentElement===document.body);
 expect(portal).toBe(true);
 const box=await page.locator('.var-pop').boundingBox();
 const vp=page.viewportSize()!;
 expect(box).not.toBeNull();
 expect(box!.x).toBeGreaterThanOrEqual(0);
 expect(box!.y).toBeGreaterThanOrEqual(0);
 expect(box!.x+box!.width).toBeLessThanOrEqual(vp.width);
 expect(box!.y+box!.height).toBeLessThanOrEqual(vp.height);

 // sem resultados: mensagem clara, sem opções
 await page.getByLabel('Buscar variável').fill('zzznenhuma');
 await expect(page.locator('.var-pop').getByRole('option')).toHaveCount(0);
 await expect(page.locator('.var-empty')).toContainText('Nenhuma variável encontrada');

 // Escape fecha o seletor e devolve o foco ao botão, editor intacto
 await page.keyboard.press('Escape');
 await expect(page.getByLabel('Buscar variável')).toHaveCount(0);
 await expect(trigger).toBeFocused();
 await expect(page.getByRole('dialog')).toBeVisible();
 await expect(page.getByLabel('Nome do fluxo',{exact:true})).toBeVisible();
 expect(errors).toEqual([]);
});

test('Avançado nasce recolhido, preserva valores e os editores das demais ações seguem acessíveis',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await novoFluxo(page,'Avançado da etapa');
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('ai.generate');
 const avancado=page.locator('.node-inspector').getByText('Avançado',{exact:true});
 await expect(avancado).toBeVisible();

 // recolhido por padrão: prévia e texto técnico fora da vista
 await expect(page.getByRole('textbox',{name:'Mensagem da pessoa',exact:true})).toBeHidden();
 await expect(page.locator('.node-inspector').getByText('Como a IA monta a resposta',{exact:true})).toBeHidden();

 // abrir, preencher, recolher: o valor continua lá
 await avancado.click();
 await page.getByText('Testar resposta contextual',{exact:true}).click();
 await page.getByRole('textbox',{name:'Mensagem da pessoa',exact:true}).fill('prévia de teste');
 await avancado.click();
 await expect(page.getByRole('textbox',{name:'Mensagem da pessoa',exact:true})).toBeHidden();
 await avancado.click();
 await expect(page.getByRole('textbox',{name:'Mensagem da pessoa',exact:true})).toHaveValue('prévia de teste');

 // campos essenciais continuam fora do Avançado
 await expect(page.getByLabel('Nome da resposta',{exact:true})).toHaveValue('aiResponse');

 // editores das demais ações seguem acessíveis
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('punish');
 await expect(page.getByRole('combobox',{name:'O que aplicar',exact:true})).toHaveValue('timeout');
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('obs');
 await expect(page.getByRole('combobox',{name:'Operação no OBS',exact:true})).toBeVisible();
 await expect(page.locator('.node-inspector').getByText('Avançado',{exact:true})).toHaveCount(1);
 await expect(page.getByRole('button',{name:'Testar no OBS',exact:true})).toBeHidden();
 await page.locator('.node-inspector').getByText('Avançado',{exact:true}).click();
 await expect(page.getByRole('button',{name:'Testar no OBS',exact:true})).toBeVisible();
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('condition');
 await expect(page.getByLabel(/^Variável: /)).toBeVisible();
 expect(errors).toEqual([]);
});

test('salvar destaca a etapa e mostra a orientação do campo obrigatório ao lado dele',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await novoFluxo(page,'Validação da etapa');
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('condition');

 // condição sem variável: orientação junto do campo, etapa destacada, nada salvo
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.locator('.field-warn')).toHaveText('Escolha uma variável');
 await expect(page.locator('.react-flow__node.step-invalid')).toHaveCount(1);
 await expect(page.getByText('Automação salva.',{exact:true})).toHaveCount(0);
 await expect(page.getByLabel('Nome do fluxo',{exact:true})).toBeVisible();

 // corrigir tira o aviso e o destaque na hora
 await page.getByLabel(/^Variável: /).click();
 await page.getByLabel('Buscar variável').fill('reward.cost');
 await page.getByRole('option',{name:/Custo da recompensa \(ponto\)/}).click();
 await expect(page.locator('.field-warn')).toHaveCount(0);
 await expect(page.locator('.react-flow__node.step-invalid')).toHaveCount(0);
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();

 // salvar fecha o editor: reabre para a segunda metade do teste
 await page.getByRole('button',{name:'Editar Validação da etapa',exact:true}).click();

 // OBS: a orientação respeita a operação escolhida
 await page.getByRole('button',{name:'Adicionar etapa',exact:true}).click();
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('obs');
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.locator('.field-warn')).toHaveText('Escolha uma entrada de áudio');
 await expect(page.locator('.react-flow__node.step-invalid')).toHaveCount(1);
 await page.getByRole('combobox',{name:'Operação no OBS',exact:true}).selectOption('scene');
 await expect(page.locator('.field-warn')).toHaveText('Escolha uma cena');
 await page.getByRole('combobox',{name:'Operação no OBS',exact:true}).selectOption('show');
 await expect(page.locator('.field-warn')).toHaveText('Escolha uma fonte');
 expect(errors).toEqual([]);
});
