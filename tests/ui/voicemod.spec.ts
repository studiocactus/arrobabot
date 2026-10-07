import {test,expect} from '@playwright/test';
import type {Page} from '@playwright/test';
import {mock} from './ux-helpers';

type Cfg={failConnect:string;failStart:string;phase:string;message:string;hasKey:boolean};

/**
 * Camada por cima do mock geral: fala o protocolo do Voicemod para o componente
 * (estados, seleção de voz, erro, controles bloqueados) sem tocar no aparelho.
 */
function voicemod(page:Page,cfg:Partial<Cfg>={}){
 return page.addInitScript((c:Partial<Cfg>)=>{
  const w=window as unknown as Record<string,unknown>;
  const V=[
   {id:'nofx',friendlyName:'Efeito desligado',enabled:true,favorited:false,isNew:false,isCustom:false},
   {id:'robot',friendlyName:'Robô',enabled:true,favorited:true,isNew:false,isCustom:false},
   {id:'alien',friendlyName:'Alien risonho',enabled:true,favorited:false,isNew:true,isCustom:false},
   {id:'blocked',friendlyName:'Voz bloqueada pela licença',enabled:false,favorited:false,isNew:false,isCustom:false}
  ];
  const st={failConnect:'',failStart:'',phase:'connected',message:'Conectado e autorizado no Voicemod.',hasKey:true,selected:'',current:'nofx',testActive:false,outcome:{kind:'',detail:''},...c};
  w.__vm=st;
  // Sem conexão autorizada o backend não tem nenhuma voz para mostrar.
  const shown=st.failConnect?[]:V;
  const name=(id:string)=>(shown.find(x=>x.id===id)||{friendlyName:''}).friendlyName;
  const snap=()=>({status:{phase:st.phase,message:st.message,port:39273,voices:shown,currentVoice:st.current,currentName:name(st.current),
   voiceChanger:st.testActive,hearMyself:false,license:'free',attempts:0},
   test:{active:st.testActive,phase:st.testActive?'running':'',voiceId:st.selected,voiceName:name(st.selected),seconds:10,remainingMs:st.testActive?8000:0,interrupted:false,manual:false},
   outcome:st.outcome,hasKey:st.hasKey,defaultSecs:10,minSecs:1,maxSecs:60});
  const core=window as unknown as {__TAURI_INTERNALS__:{invoke:(c:string,p:{op?:string;args?:Record<string,unknown>})=>unknown}};
  const inner=core.__TAURI_INTERNALS__||{invoke:async()=>null};
  const orig=inner.invoke.bind(inner);
  inner.invoke=async(c:string,p:{op?:string;args?:Record<string,unknown>})=>{
   const op=p.op||'';
   if(op.indexOf('voicemod.')!==0)return orig(c,p);
   if(op==='voicemod.get')return snap();
   if(op==='voicemod.key'){st.hasKey=Boolean(p.args&&p.args.value);return {hasKey:st.hasKey}}
   if(op==='voicemod.connect'){
    if(st.failConnect){st.phase='failed';st.message=st.failConnect;throw st.failConnect}
    st.phase='connected';st.message='Conectado e autorizado no Voicemod.';return snap();
   }
   if(op==='voicemod.disconnect'){st.phase='disconnected';st.message='Desconectado a pedido.';st.testActive=false;st.outcome={kind:'',detail:''};return {stoppedTest:false,restored:false,detail:''}}
   if(op==='voicemod.refresh'){st.message='Vozes atualizadas no Voicemod.';return snap()}
   if(op==='voicemod.testStart'){
    if(st.failStart)throw st.failStart;
    st.selected=String((p.args&&p.args.voiceId)||'');st.testActive=true;st.outcome={kind:'',detail:''};
    return snap().test;
   }
   if(op==='voicemod.testStop'){
    st.testActive=false;st.current=st.selected||st.current;
    st.outcome={kind:'restored',detail:'Voz restaurada: Efeito desligado.'};
    return snap();
   }
   if(op==='voicemod.recover'){st.outcome={kind:'restored',detail:'Voz restaurada: Efeito desligado.'};return snap()}
   return null;
  };
 },cfg);
}

const open=async(page:Page)=>{
 await page.goto('/');
 await page.getByRole('button',{name:'Voicemod',exact:true}).click();
};

test('estados, seleção de voz, teste e restauração na tela do Voicemod',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await mock(page);await voicemod(page);
 await open(page);

 await expect(page.locator('main h1')).toHaveText('Voicemod');
 await expect(page.getByRole('heading',{name:'Voicemod nesta máquina',exact:true})).toBeVisible();
 await expect(page.getByRole('heading',{name:'Chave da Control API',exact:true})).toBeVisible();
 await expect(page.getByRole('heading',{name:'Vozes do Voicemod',exact:true})).toBeVisible();
 await expect(page.getByRole('heading',{name:'Testar voz',exact:true})).toBeVisible();

 // estado: conexão real só depois de autorizar, e o texto explica o próximo passo
 await expect(page.getByText('Conectado',{exact:true})).toBeVisible();
 await expect(page.getByText('Conectado e autorizado no Voicemod.',{exact:true})).toBeVisible();
 await expect(page.getByRole('button',{name:'Desconectar',exact:true})).toBeVisible();
 await expect(page.getByRole('button',{name:'Atualizar vozes',exact:true})).toBeEnabled();

 // a lista mostra nome amigável e o id real da voz, nunca um id inventado
 const rows=page.locator('.voice-row');
 await expect(rows).toHaveCount(4);
 await expect(rows.nth(1)).toContainText('Robô');
 await expect(rows.nth(1)).toContainText('robot');
 await expect(rows.nth(0)).toContainText('atual');
 await expect(page.getByRole('button',{name:/Voz bloqueada pela licença/})).toBeDisabled();
 await page.screenshot({path:'artifacts/voicemod-vozes.png',fullPage:true});

 // busca por nome
 await page.getByLabel('Buscar voz pelo nome',{exact:true}).fill('rob');
 await expect(rows).toHaveCount(1);
 await expect(rows.nth(0)).toContainText('Robô');
 await page.getByLabel('Buscar voz pelo nome',{exact:true}).fill('');
 await expect(rows).toHaveCount(4);

 // sem seleção não existe teste; a tela diz o que falta
 const start=page.getByRole('button',{name:'Testar voz',exact:true});
 await expect(start).toBeDisabled();
 await expect(page.getByText('Escolha uma voz na lista acima para testar.',{exact:true})).toBeVisible();
 await expect(page.getByRole('button',{name:/Encerrar teste e restaurar/})).toHaveCount(0);

 // duração configurável com o padrão do backend
 await expect(page.getByRole('combobox',{name:'Duração do teste',exact:true})).toHaveValue('10');

 // seleção liga o teste
 await rows.nth(1).click();
 await expect(rows.nth(1)).toHaveAttribute('aria-pressed','true');
 await expect(start).toBeEnabled();

 await start.click();
 await expect(page.getByText('Testando Robô',{exact:false})).toBeVisible();
 await expect(page.locator('.test-timer')).toHaveText('8s');
 await expect(page.getByRole('button',{name:/Encerrar teste e restaurar/})).toBeVisible();
 await expect(page.getByRole('button',{name:'Testar voz',exact:true})).toHaveCount(0);
 await page.screenshot({path:'artifacts/voicemod-teste.png',fullPage:true});

 await page.getByRole('button',{name:/Encerrar teste e restaurar/}).click();
 await expect(page.locator('.outcome.ok')).toContainText('Voz restaurada');
 await expect(page.getByRole('button',{name:'Testar voz',exact:true})).toBeEnabled();

 // a chave não é obrigatória para nada além de conectar, mas o botão espera conteúdo
 await expect(page.getByRole('button',{name:'Salvar chave',exact:true})).toBeDisabled();
 await expect(page.getByText('Chave guardada no cofre. Preencha para trocar.',{exact:true})).toBeVisible();

 expect(errors).toEqual([]);
});

test('falha de conexão fica na tela e vira estado de Falha, não mensagem de chat',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await mock(page);
 await voicemod(page,{phase:'disconnected',message:'Abra o Voicemod nesta máquina e clique em Conectar.',hasKey:false,
  failConnect:'Chave da Control API ausente. Peça sua chave em control-api.voicemod.net e salve em Voicemod → Chave da API.'});
 await open(page);

 await expect(page.getByText('Desconectado',{exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Conectar',exact:true}).click();

 const err=page.locator('.inline-error');
 await expect(err).toContainText('Chave da Control API ausente');
 await expect(page.getByText('Falha',{exact:true})).toBeVisible();
 await expect(page.locator('.list-row small').first()).toHaveText('Chave da Control API ausente. Peça sua chave em control-api.voicemod.net e salve em Voicemod → Chave da API.');
 await expect(page.locator('.voice-row')).toHaveCount(0);
 await expect(page.getByText('Ainda sem vozes. Conecte e use Atualizar vozes.',{exact:true})).toBeVisible();
 expect(errors).toEqual([]);
});

test('falha ao iniciar o teste aparece na tela sem fingir que o teste rodou',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await mock(page);
 await voicemod(page,{failStart:'O Voicemod não confirmou a troca para robot: voz indisponível ou comando não confirmado.'});
 await open(page);

 await page.locator('.voice-row').nth(1).click();
 await page.getByRole('button',{name:'Testar voz',exact:true}).click();
 await expect(page.locator('.inline-error')).toContainText('não confirmou a troca');
 await expect(page.getByText(/Testando Robô/)).toHaveCount(0);
 await expect(page.getByRole('button',{name:'Testar voz',exact:true})).toBeEnabled();
 expect(errors).toEqual([]);
});

test('prévia no navegador se identifica e desliga o que depende do Voicemod',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('/');
 await page.getByRole('button',{name:'Criar primeiro bot',exact:true}).click();
 await page.getByLabel('Nome do perfil',{exact:true}).fill('Canal de teste');
 await page.getByRole('button',{name:'Salvar perfil',exact:true}).click();
 await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
 await page.locator('.sidebar').getByRole('button',{name:'Voicemod',exact:true}).click();

 await expect(page.getByText('Prévia no navegador: conectar, listar vozes e testar exigem o aplicativo desktop. Aqui a tela aparece apenas para conferência.',{exact:true})).toBeVisible();
 await expect(page.getByText('Prévia no navegador: a lista de vozes só existe no aplicativo desktop.',{exact:true})).toBeVisible();
 await expect(page.getByRole('button',{name:'Conectar',exact:true})).toBeDisabled();
 await expect(page.getByRole('button',{name:'Atualizar vozes',exact:true})).toBeDisabled();
 await expect(page.getByRole('button',{name:'Salvar chave',exact:true})).toBeDisabled();
 await expect(page.getByRole('button',{name:'Testar voz',exact:true})).toBeDisabled();
 await expect(page.getByRole('combobox',{name:'Duração do teste',exact:true})).toBeEnabled();
 expect(errors).toEqual([]);
});
