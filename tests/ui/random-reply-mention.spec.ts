import {test,expect} from '@playwright/test';
test('ficha Número sorteado insere a faixa e explica as casas decimais em Opções',async({page})=>{
  await page.goto('/');
  await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();
  await page.getByLabel('Nome do perfil',{exact:true}).fill('Sorteio');
  await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();
  await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
  await page.getByRole('button',{name:'Comandos',exact:true}).click();
  await page.getByRole('button',{name:'Novo comando',exact:true}).click();
  const response=page.getByRole('textbox',{name:'Resposta',exact:true});
  await response.fill('O número é ');
  await page.getByText('Inserir variável e testar mensagem',{exact:true}).click();
  const cards=page.locator('.variable-card');
  const drawn=cards.filter({hasText:'Número sorteado'});
  await expect(drawn).toHaveCount(1);
  await drawn.locator('.variable-card-main').click();
  await expect(response).toHaveValue('O número é {{random:1,50}}');
  await expect(page.locator('.variable-done')).toContainText('{{random:1,50}}');

  // Opções explica a faixa em vez de oferecer alternativa e formato
  await response.fill('O número é ');
  await drawn.locator('.variable-card-more').click();
  const options=page.locator('.variable-options');
  await expect(options).toBeVisible();
  await expect(options).toContainText('{{random:1,50.00}}');
  await expect(options).toContainText('6,65');
  await expect(options.getByLabel('Se não houver valor, mostrar',{exact:true})).toHaveCount(0);
  await expect(options.getByRole('combobox',{name:'Como mostrar',exact:true})).toHaveCount(0);
  await page.getByRole('button',{name:'Inserir na mensagem',exact:true}).click();
  await expect(response).toHaveValue('O número é {{random:1,50}}');
  await expect(page.locator('.message-readable')).toContainText('Número sorteado');
});

test('gatilho Chamada pelo nome do bot vem na lista, explica a lista por vírgula e guarda a resposta a quem enviou',async({page})=>{
  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto('/');
  await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();
  await page.getByLabel('Nome do perfil',{exact:true}).fill('Menção e fio');
  await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();
  await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
  await page.getByRole('button',{name:'Automações',exact:true}).click();
  await page.getByRole('button',{name:'Novo fluxo',exact:true}).click();
  await page.getByLabel('Nome do fluxo',{exact:true}).fill('Chamada do bot');

  // a opção existe e a dica fala em lista por vírgula e palavra inteira
  const event=page.getByRole('combobox',{name:'Evento',exact:true});
  await event.selectOption('mention');
  await expect(event.locator('option[value="mention"]')).toHaveText('Chamada pelo nome do bot');
  await expect(page.getByText('Separe os nomes do bot com vírgula: o gatilho passa quando um deles aparece na mensagem, como palavra inteira. Ex.: Arroba, ArrobaSrv, arromba.',{exact:true})).toBeVisible();
  await page.getByLabel('Texto que dispara',{exact:true}).fill('Arroba, ArrobaSrv');

  // responder a quem enviou fica em Como sai
  const reply=page.getByRole('switch',{name:'Responder à pessoa que enviou',exact:true});
  await expect(reply).toBeVisible();
  await expect(reply).toHaveAttribute('aria-checked','false');
  await reply.click();
  await expect(reply).toHaveAttribute('aria-checked','true');
  await expect(page.getByText('A mensagem entra no fio da pessoa que disparou, como um reply da Twitch. Vale quando a automação parte de uma mensagem de chat.',{exact:true})).toBeVisible();
  await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
  await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();

  // reabre e confere que gatilho e fio continuam como ficaram
  await expect(page.getByRole('button',{name:'Editar Chamada do bot',exact:true})).toBeVisible();
  await page.getByRole('button',{name:'Editar Chamada do bot',exact:true}).click();
  await expect(page.getByRole('combobox',{name:'Evento',exact:true})).toHaveValue('mention');
  await expect(page.getByLabel('Texto que dispara',{exact:true})).toHaveValue('Arroba, ArrobaSrv');
  await expect(page.getByRole('switch',{name:'Responder à pessoa que enviou',exact:true})).toHaveAttribute('aria-checked','true');
  await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
  expect(errors).toEqual([]);
});

test('comando simples também oferece a resposta a quem enviou e a guarda',async({page})=>{
  await page.goto('/');
  await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();
  await page.getByLabel('Nome do perfil',{exact:true}).fill('Fio do comando');
  await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();
  await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
  await page.getByRole('button',{name:'Comandos',exact:true}).click();
  await page.getByRole('button',{name:'Novo comando',exact:true}).click();
  await page.getByLabel('Nome',{exact:true}).fill('Valeu');
  await page.getByLabel('Comando',{exact:true}).fill('!valeu');
  await page.getByRole('textbox',{name:'Resposta',exact:true}).fill('Valeu, {{user}}!');
  const reply=page.getByRole('switch',{name:'Responder à pessoa que enviou',exact:true});
  await expect(reply).toBeVisible();
  await reply.click();
  await page.getByRole('button',{name:'Salvar comando',exact:true}).click();
  await expect(page.getByText('Automação salva.',{exact:true})).toBeVisible();
  await page.getByRole('button',{name:'Editar Valeu',exact:true}).click();
  await expect(page.getByRole('switch',{name:'Responder à pessoa que enviou',exact:true})).toHaveAttribute('aria-checked','true');
});

test('quadro de instrução da IA cresce com o texto e para no teto de 42vh',async({page})=>{
  await page.goto('/');
  await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();
  await page.getByLabel('Nome do perfil',{exact:true}).fill('Campo crescente');
  await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();
  await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
  await page.getByRole('button',{name:'Comandos',exact:true}).click();
  await page.getByRole('button',{name:'Resenha com IA',exact:true}).click();
  const box=page.getByRole('textbox',{name:'Como a IA deve responder',exact:true});
  const heightOf=()=>box.evaluate(el=>({auto:el.style.height,box:el.clientHeight}));
  const start=await heightOf();
  await box.fill('Uma linha só.');
  const one=await heightOf();
  expect(one.auto).not.toBe('');
  expect(one.box).toBeGreaterThanOrEqual(start.box);
  await box.fill(Array.from({length:12},(_,i)=>`Linha ${i+1} da instrução, para ocupar várias linhas do quadro.`).join('\n'));
  const many=await heightOf();
  expect(many.box).toBeGreaterThan(one.box);
  const ceiling=await box.evaluate(el=>Math.round(window.innerHeight*0.42));
  expect(many.box).toBeLessThanOrEqual(ceiling);
  await page.screenshot({path:'artifacts/auto-grow-instruction.png',fullPage:true});
});
