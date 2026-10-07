import {test,expect,type Page} from '@playwright/test';

/** Testar fluxo 0.1.92: preparar, iniciar uma vez, acompanhar, cancelar e fechar. */

type Step={index:number;kind:string;status:string};
type Run={id:string;flow:string;status:string;test:boolean;steps:Step[];error:string|null};
type St={
 attempts:number;success:number;runsCalls:number;ops:string[];forbidden:string[];
 scenario:'run'|'hold'|'condition'|'fail';
 testFail:boolean;runsFail:boolean;cancelError:boolean;cancelAt:number;
 run:Run|null;sent:{name:string;vars:Record<string,string>;kinds:string[]}|null;
};

const profile={id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{}};

const act=(kind:string,text='',extra:Record<string,unknown>={})=>({
 kind,text,target:'',value:0,condition:'',aiAnchor:'',aiKnowledge:'',aiLength:'',aiStyle:'',aiNoRepeat:null,...extra
});

const simple={
 id:'f1',profileId:'p1',name:'Boas-vindas',enabled:true,audio:'',audioVolume:1,sendType:'chat',sendColor:'primary',counter:false,
 trigger:{kind:'command',pattern:'!bemvindo',permission:'everyone',cooldown:5,userCooldown:15},
 actions:[act('chat','Olá {{viewer.name}}')]
};

const visual={
 id:'f2',profileId:'p1',name:'Alerta da live',enabled:true,audio:'',audioVolume:1,sendType:'chat',sendColor:'primary',counter:false,
 trigger:{kind:'command',pattern:'!alerta',permission:'everyone',cooldown:5,userCooldown:15},
 actions:[
  act('condition','',{condVar:'reward.cost',condOp:'greater_or_equal',condValue:'100',condFalse:'stop'}),
  act('chat','Oi {{viewer.name}} · {{xyzabc}}'),
  act('overlay','Aparece na OBS')
 ]
};

/** Mock desktop: controla chamadas, estados intermediários e falhas de cada operação. */
async function mock(page:Page,flows:unknown[]){
 await page.addInitScript(arg=>{
  const profile=arg.profile;
  const st:St={
   attempts:0,success:0,runsCalls:0,ops:[],forbidden:[],scenario:'run',
   testFail:false,runsFail:false,cancelError:false,cancelAt:0,run:null,sent:null
  };
  const advance=()=>{
   const r=st.run;if(!r)return;
   if(st.cancelAt&&Date.now()-st.cancelAt>=1200){
    const i=r.steps.findIndex(s=>s.status==='RUNNING'||s.status==='WAITING');
    if(i>=0)r.steps[i].status='CANCELLED';
    r.status='CANCELLED';return;
   }
   if(r.status!=='RUNNING'&&r.status!=='WAITING')return;
   if(st.scenario==='hold')return;
   if(st.scenario==='fail'){r.steps[0].status='FAILED';r.error='unknown field `layout`, expected one of `actions`';r.status='FAILED';return}
   if(st.scenario==='condition'){r.steps[0].status='FALSE';r.status='STOPPED_BY_CONDITION';return}
   const i=r.steps.findIndex(s=>s.status==='RUNNING'||s.status==='WAITING');
   if(i<0){r.status='COMPLETED';return}
   r.steps[i].status='SUCCESS';
   if(i+1<r.steps.length)r.steps[i+1].status='RUNNING';else r.status='COMPLETED';
  };
  const bad=/^(flow\.save|simulate|obs\.|punish|timer\.preview|command\.counter\.|memory\.|points\.|chatExtras\.(send|save))/;
  const w=window as unknown as Record<string,unknown>;
  w.__st=st;
  Object.assign(w,{
   isTauri:true,
   __TAURI_EVENT_PLUGIN_INTERNALS__:{registerListener:()=>{},unregisterListener:()=>{}},
   __TAURI_INTERNALS__:{
    transformCallback:()=>1,
    unregisterCallback:()=>{},
    invoke:async(cmd:string,payload:{op:string;args?:Record<string,any>})=>{
     if(cmd!=='rpc')return 1;
     const op=payload.op;const a=payload.args||{};
     st.ops.push(op);
     if(bad.test(op))st.forbidden.push(op);
     if(op==='snapshot')return {profiles:[profile],logs:[],statuses:{},theme:'dark',apiPort:0,dataDir:'teste'};
     if(op==='flows')return arg.flows;
     if(op==='command.counters')return {};
     if(op==='module.config.get')return {};
     if(op==='flow.test'){
      st.attempts++;
      await new Promise(r=>setTimeout(r,150));
      if(st.testFail)throw Error('Falha de teste simulada');
      st.success++;
      const id='exec-'+st.success;
      st.sent={name:a.flow.name,vars:a.testVars||{},kinds:a.flow.actions.map((x:any)=>x.kind)};
      st.run={id,flow:a.flow.name,status:'RUNNING',test:true,
       steps:a.flow.actions.map((x:any,i:number)=>({index:i,kind:x.kind,status:'PENDING'})),error:null};
      if(st.run.steps[0])st.run.steps[0].status='RUNNING';
      st.cancelAt=0;
      return {executionId:id};
     }
     if(op==='flow.runs'){
      st.runsCalls++;
      if(st.runsFail)throw Error('Sem resposta do aplicativo');
      advance();
      return st.run?[JSON.parse(JSON.stringify(st.run))]:[];
     }
     if(op==='flow.cancel'){
      if(st.cancelError)throw Error('Falha de comunicação');
      st.cancelAt=Date.now();
      return null;
     }
     return null;
    }
   }
  });
 },{flows,profile});
 await page.goto('/');
}

const st=async(page:Page)=>page.evaluate(()=>{
 const s=(window as unknown as {__st:St}).__st;
 return {attempts:s.attempts,success:s.success,runsCalls:s.runsCalls,ops:s.ops,forbidden:s.forbidden,sent:s.sent};
});
const setSt=async(page:Page,patch:Partial<St>)=>{
 await page.evaluate(p=>{Object.assign((window as unknown as {__st:St}).__st,p)},patch);
};
const D=(page:Page,name:string)=>page.getByRole('dialog',{name,exact:true});
const errorsOf=(page:Page)=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));return errors;
};

test('painel de teste abre nos dois editores sem executar, sem salvar e sem efeitos externos',async({page})=>{
 const errors=errorsOf(page);
 await mock(page,[simple,visual]);

 // editor simples (Comandos)
 await page.getByRole('button',{name:'Comandos',exact:true}).click();
 await page.getByRole('button',{name:'Editar Boas-vindas',exact:true}).click();
 await expect(D(page,'Configurar comando')).toBeVisible();
 await expect(page.locator('[data-test-flow]')).toHaveCount(1);
 await page.getByRole('button',{name:'Testar fluxo',exact:true}).click();

 const test1=D(page,'Testar Boas-vindas');
 await expect(test1).toBeVisible();
 await expect(test1.getByText('Gatilho: Comando de chat · !bemvindo',{exact:true})).toBeVisible();
 await expect(test1.getByText('1 etapa',{exact:true})).toBeVisible();
 await expect(test1.getByText('Abrir este painel não salva o fluxo e não executa nada. Confira os valores e escolha quando começar.',{exact:true})).toBeVisible();
 // o que é simulado fica dito por extenso
 await expect(test1.getByText('não publica no chat, não altera o OBS',{exact:false})).toBeVisible();
 // nome humano do catálogo, preservando o código ao lado
 await expect(test1.getByLabel('Nome de quem disparou',{exact:true})).toHaveValue('TesteViewer');
 await expect(test1.getByText('Código: viewer.name',{exact:true})).toBeVisible();
 // abrir não executa nem salva
 let s=await st(page);
 expect(s.attempts,'abrir não chama flow.test').toBe(0);
 expect(s.forbidden,'nenhuma operação externa').toEqual([]);

 // Escape fecha só o teste
 await page.keyboard.press('Escape');
 await expect(test1).toHaveCount(0);
 await expect(D(page,'Configurar comando')).toBeVisible();

 // botão Fechar fecha só o teste e o editor continua
 await page.getByRole('button',{name:'Testar fluxo',exact:true}).click();
 await test1.getByRole('button',{name:'Fechar',exact:true}).click();
 await expect(test1).toHaveCount(0);
 await expect(D(page,'Configurar comando')).toBeVisible();
 s=await st(page);
 expect(s.attempts).toBe(0);
 expect(s.forbidden).toEqual([]);

 // editor visual (Automações)
 await D(page,'Configurar comando').getByRole('button',{name:'Fechar',exact:true}).click();
 await page.getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Editar Alerta da live',exact:true}).click();
 await expect(D(page,'Editor de automação')).toBeVisible();
 await expect(page.locator('[data-test-flow]')).toHaveCount(1);
 await page.getByRole('button',{name:'Testar fluxo',exact:true}).click();
 const test2=D(page,'Testar Alerta da live');
 await expect(test2).toBeVisible();
 await expect(test2.getByText('3 etapas',{exact:true})).toBeVisible();
 await expect(test2.getByLabel('Custo da recompensa (ponto)',{exact:true})).toHaveValue('5000');
 await expect(test2.getByLabel('xyzabc',{exact:true})).toHaveValue('');
 await expect(test2.getByText('Código: viewer.name',{exact:true})).toBeVisible();
 s=await st(page);
 expect(s.attempts,'nenhuma execução ao abrir no editor visual').toBe(0);
 expect(s.forbidden).toEqual([]);
 expect(errors).toEqual([]);
});

test('um único clique inicia uma execução, mesmo em clique duplo, usando o texto não salvo',async({page})=>{
 await mock(page,[simple,visual]);
 await page.getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Editar Alerta da live',exact:true}).click();

 // muda o nome no editor e não salva
 await page.getByLabel('Nome do fluxo',{exact:true}).fill('Alerta da live (não salvo)');
 await page.getByRole('button',{name:'Testar fluxo',exact:true}).click();
 await expect(D(page,'Testar Alerta da live (não salvo)').getByText('Alerta da live (não salvo)',{exact:true})).toBeVisible();

 // clique duplo não vira duas execuções
 await page.getByRole('button',{name:'Iniciar teste',exact:true}).dblclick();
 const dlg=D(page,'Testar Alerta da live (não salvo)');
 await expect(dlg.getByText('Execução exec-1',{exact:true})).toBeVisible();
 await expect(dlg.getByRole('button',{name:'Iniciar teste',exact:true})).toHaveCount(0);

 const s=await st(page);
 expect(s.attempts,'clique duplo pede uma execução só').toBe(1);
 expect(s.success).toBe(1);
 expect(s.sent?.name).toBe('Alerta da live (não salvo)');
 expect(s.sent?.kinds).toEqual(['condition','chat','overlay']);
 expect(s.sent?.vars).toEqual({'reward.cost':'5000','viewer.name':'TesteViewer',xyzabc:''});
 expect(s.ops.includes('flow.save'),'nada foi salvo').toBe(false);
 expect(s.forbidden).toEqual([]);
});

test('acompanha etapa a etapa com estados traduzidos e repete só por ação explícita',async({page})=>{
 await mock(page,[visual]);
 await setSt(page,{scenario:'run'});
 await page.getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Editar Alerta da live',exact:true}).click();
 await page.getByRole('button',{name:'Testar fluxo',exact:true}).click();
 const dlg=D(page,'Testar Alerta da live');
 await page.getByRole('button',{name:'Iniciar teste',exact:true}).click();

 await expect(dlg.getByText('Execução exec-1 · Executando',{exact:true})).toBeVisible();
 await expect(dlg.getByText('✓ Concluída',{exact:true}).first()).toBeVisible();
 await expect(dlg.getByText('Execução exec-1 · Concluída',{exact:true})).toBeVisible({timeout:6000});
 await expect(dlg.getByText('Executar novamente',{exact:true})).toBeVisible();
 expect((await st(page)).attempts).toBe(1);

 // repetição é uma ação explícita e cria outra execução
 await dlg.getByRole('button',{name:'Executar novamente',exact:true}).click();
 await expect(dlg.getByText('Execução exec-2 · Concluída',{exact:true})).toBeVisible({timeout:6000});
 expect((await st(page)).attempts).toBe(2);
});

test('condição falsa aparece como parada do fluxo, nunca como erro técnico',async({page})=>{
 await mock(page,[visual]);
 await setSt(page,{scenario:'condition'});
 await page.getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Editar Alerta da live',exact:true}).click();
 await page.getByRole('button',{name:'Testar fluxo',exact:true}).click();
 const dlg=D(page,'Testar Alerta da live');
 await page.getByRole('button',{name:'Iniciar teste',exact:true}).click();

 await expect(dlg.getByText('Execução exec-1 · Parada por condição',{exact:true})).toBeVisible();
 await expect(dlg.getByText('○ Condição falsa',{exact:true})).toBeVisible();
 await expect(dlg.getByText('Condição falsa: o fluxo parou aqui. Não é uma falha.',{exact:true})).toBeVisible();
 await expect(dlg.getByText('○ Aguardando execução',{exact:true}).first()).toBeVisible();
 await expect(dlg.getByRole('alert')).toHaveCount(0);
 // parada por condição é conclusão: dá para repetir
 await expect(dlg.getByText('Executar novamente',{exact:true})).toBeVisible();
});

test('falha mostra a etapa, o motivo e os detalhes técnicos recolhidos',async({page})=>{
 await mock(page,[visual]);
 await setSt(page,{scenario:'fail'});
 await page.getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Editar Alerta da live',exact:true}).click();
 await page.getByRole('button',{name:'Testar fluxo',exact:true}).click();
 const dlg=D(page,'Testar Alerta da live');
 await page.getByRole('button',{name:'Iniciar teste',exact:true}).click();

 await expect(dlg.getByText('Execução exec-1 · Falhou',{exact:true})).toBeVisible();
 await expect(dlg.getByText('Esta etapa falhou:',{exact:true})).toBeVisible();
 const alert=dlg.getByRole('alert');
 await expect(alert).toHaveCount(1);
 await expect(alert).toContainText('O aplicativo não conseguiu ler os dados salvos');
 // o motivo técnico fica recolhido, ao lado da etapa, e só abre quando pedem
 await expect(dlg.getByText('Detalhes técnicos',{exact:true})).toBeVisible();
 await expect(dlg.locator('details code')).toBeHidden();
 await dlg.getByText('Detalhes técnicos',{exact:true}).click();
 await expect(dlg.locator('details code')).toBeVisible();
 await expect(dlg.locator('details code')).toContainText('unknown field');
 // a falha interrompe: as etapas seguintes continuam aguardando
 await expect(dlg.getByText('1. ◆ Custo da recompensa (ponto) ≥ 100',{exact:true})).toBeVisible();
 await expect(dlg.getByText('2. Enviar mensagem',{exact:true})).toBeVisible();
 await expect(dlg.getByText('○ Aguardando execução',{exact:true}).first()).toBeVisible();
});

test('cancelar só anuncia depois que a execução confirma o estado final',async({page})=>{
 await mock(page,[visual]);
 await setSt(page,{scenario:'hold'});
 await page.getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Editar Alerta da live',exact:true}).click();
 await page.getByRole('button',{name:'Testar fluxo',exact:true}).click();
 const dlg=D(page,'Testar Alerta da live');
 await page.getByRole('button',{name:'Iniciar teste',exact:true}).click();
 await expect(dlg.getByText('Execução exec-1 · Executando',{exact:true})).toBeVisible();

 await dlg.getByRole('button',{name:'Cancelar execução',exact:true}).click();
 await expect(dlg.getByRole('button',{name:'Cancelando…',exact:true})).toBeVisible();
 await expect(dlg.getByRole('status')).toHaveCount(0);
 await expect(dlg.getByText('Execução cancelada. Nada mais roda nesta execução.',{exact:true})).toHaveCount(0);

 await expect(dlg.getByRole('status')).toBeVisible({timeout:6000});
 await expect(dlg.getByText('Execução cancelada. Nada mais roda nesta execução.',{exact:true})).toBeVisible();
 await expect(dlg.getByText('Execução exec-1 · Cancelada',{exact:true})).toBeVisible();
 await expect(dlg.getByText('■ Cancelada',{exact:true})).toBeVisible();
});

test('cancelamento recusado avisa sem fechar nem encerrar o acompanhamento',async({page})=>{
 await mock(page,[visual]);
 await setSt(page,{scenario:'hold',cancelError:true});
 await page.getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Editar Alerta da live',exact:true}).click();
 await page.getByRole('button',{name:'Testar fluxo',exact:true}).click();
 const dlg=D(page,'Testar Alerta da live');
 await page.getByRole('button',{name:'Iniciar teste',exact:true}).click();
 await expect(dlg.getByText('Execução exec-1 · Executando',{exact:true})).toBeVisible();

 await dlg.getByRole('button',{name:'Cancelar execução',exact:true}).click();
 const alert=dlg.getByRole('alert');
 await expect(alert).toHaveCount(1);
 await expect(alert).toContainText('Não foi possível cancelar:');
 await expect(alert).toContainText('Falha de comunicação');
 // volta para o estado de espera e o editor continua intacto
 await expect(dlg.getByRole('button',{name:'Cancelar execução',exact:true})).toBeEnabled();
 await expect(dlg.getByText('Execução exec-1 · Executando',{exact:true})).toBeVisible();
 await expect(D(page,'Editor de automação')).toBeVisible();
});

test('fechar durante a execução pede escolha, respeita Escape e devolve o foco ao editor',async({page})=>{
 await mock(page,[visual]);
 await setSt(page,{scenario:'hold'});
 await page.getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Editar Alerta da live',exact:true}).click();
 await page.getByRole('button',{name:'Testar fluxo',exact:true}).click();
 const dlg=D(page,'Testar Alerta da live');
 await page.getByRole('button',{name:'Iniciar teste',exact:true}).click();
 await expect(dlg.getByText('Execução exec-1 · Executando',{exact:true})).toBeVisible();

 // Escape não fecha em cheio: dá a escolha
 await page.keyboard.press('Escape');
 const choice=page.getByRole('alertdialog',{name:'Teste ainda em andamento',exact:true});
 await expect(choice).toBeVisible();
 await expect(D(page,'Editor de automação')).toBeVisible();
 // outro Escape não fecha nada
 await page.keyboard.press('Escape');
 await expect(choice).toBeVisible();
 await choice.getByRole('button',{name:'Continuar acompanhando',exact:true}).click();
 await expect(dlg.getByText('Execução exec-1 · Executando',{exact:true})).toBeVisible();

 // clicar fora do painel também dá a escolha
 await page.mouse.click(8,8);
 await expect(choice).toBeVisible();
 await expect(choice.getByRole('button',{name:'Cancelar e fechar',exact:true})).toBeVisible();

 // cancelar e fechar só conclui quando o motor confirmar
 await choice.getByRole('button',{name:'Cancelar e fechar',exact:true}).click();
 await expect(dlg.getByRole('button',{name:'Cancelando…',exact:true})).toBeVisible();
 await expect(dlg).toHaveCount(0,{timeout:6000});

 // o editor continua aberto e o foco volta para Testar fluxo
 await expect(D(page,'Editor de automação')).toBeVisible();
 await expect(page.getByLabel('Nome do fluxo',{exact:true})).toHaveValue('Alerta da live');
 await expect.poll(()=>page.evaluate(()=>document.activeElement?.hasAttribute('data-test-flow')??false)).toBe(true);
});

test('falha ao iniciar orienta sem perder os valores e falha de acompanhamento não repete a execução',async({page})=>{
 await mock(page,[visual]);
 await setSt(page,{scenario:'hold',testFail:true});
 await page.getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Editar Alerta da live',exact:true}).click();
 await page.getByRole('button',{name:'Testar fluxo',exact:true}).click();
 const dlg=D(page,'Testar Alerta da live');

 // início falho: avisa, mantém a preparação e os valores
 await page.getByRole('button',{name:'Iniciar teste',exact:true}).click();
 await expect(dlg.getByText('Não foi possível iniciar o teste.',{exact:true})).toBeVisible();
 await expect(dlg.getByRole('alert')).toContainText('Falha de teste simulada');
 await expect(dlg.getByText('Nenhum teste foi executado e os valores continuam aqui. Escolha Iniciar teste de novo.',{exact:true})).toBeVisible();
 await expect(dlg.getByLabel('Nome de quem disparou',{exact:true})).toHaveValue('TesteViewer');
 let s=await st(page);
 expect(s.attempts).toBe(1);
 expect(s.success).toBe(0);

 // recomeça e acompanha
 await setSt(page,{testFail:false});
 await page.getByRole('button',{name:'Iniciar teste',exact:true}).click();
 await expect(dlg.getByText('Execução exec-1 · Executando',{exact:true})).toBeVisible();

 // o acompanhamento falhou: avisa e não cria outra execução
 await setSt(page,{runsFail:true});
 await expect(dlg.getByRole('alert')).toContainText('Não foi possível acompanhar a execução:');
 await expect(dlg.getByRole('alert')).toContainText('Sem resposta do aplicativo');
 await expect(dlg.getByText('Execução exec-1 · Executando',{exact:true})).toBeVisible();
 await new Promise(r=>setTimeout(r,1200));
 s=await st(page);
 expect(s.attempts,'a falha de acompanhamento não começa outro teste').toBe(2);
 expect(s.success).toBe(1);
});
