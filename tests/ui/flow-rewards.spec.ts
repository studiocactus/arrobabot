import {test,expect,type Page} from '@playwright/test';
import {join} from 'path';
import {tmpdir} from 'os';

const OUT=join(tmpdir(),'botlive-ux');

/**
 * Regressão do vínculo recompensa↔voz e da duplicação de automações: o que o
 * usuário escolhe permanece salvo (pelo ID estável), "Qualquer resgate" é
 * escolha explícita e a cópia nasce desativada, independente da original.
 */
async function mockDesktop(page:Page){
 await page.addInitScript(()=>{
  const flows:Record<string,unknown>=JSON.parse(sessionStorage.getItem('flows')||'{}');
  const rewards=()=>sessionStorage.getItem('rewards')==='sem-a'
   ?[{id:'r-b',title:'Voz B',cost:200,enabled:true}]
   :[{id:'r-a',title:'Voz A',cost:100,enabled:true},{id:'r-b',title:'Voz B',cost:200,enabled:true}];
  Object.assign(window,{isTauri:true,__TAURI_EVENT_PLUGIN_INTERNALS__:{registerListener:()=>{},unregisterListener:()=>{}},
   __TAURI_INTERNALS__:{transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(cmd:string,payload:{op:string;args?:Record<string,unknown>})=>{
    if(cmd!=='rpc')return 1;
    const op=payload.op;const args=payload.args||{};
    if(op==='snapshot')return {profiles:[{id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{}}],flows:[],notes:{},presets:[],theme:'dark',logs:[],statuses:{},apiPort:9876,dataDir:'teste'};
    if(op==='flows')return Object.values(flows);
    if(op==='flow.save'){const f=args.flow as {id:string};flows[f.id]=f;sessionStorage.setItem('flows',JSON.stringify(flows));return f}
    if(op==='flow.delete'){delete (flows as Record<string,unknown>)[String(args.id)];sessionStorage.setItem('flows',JSON.stringify(flows));return null}
    if(op==='twitch.rewards')return {channel:'cactus',rewards:rewards()};
    if(op==='voicemod.get')return {status:{phase:'connected',message:'',port:59129,voices:[{voiceID:'robot',name:'Robô'},{voiceID:'alien',name:'Alien'}],currentVoice:'nofx',currentName:'Efeito desligado',voiceChanger:true,hearMyself:false,license:'Pro',attempts:1},test:{active:false,phase:'',voiceId:'',voiceName:'',seconds:10,remainingMs:0,interrupted:false,manual:false},outcome:{kind:'',detail:''},defaultSecs:10,minSecs:5,maxSecs:60,hasKey:true,keyLen:20};
    if(op==='command.counters')return {};
    if(op==='settings.get')return {theme:'dark',apiPort:9876,apiToken:'',updateEndpoint:'',updatePublicKey:'',autoUpdate:false};
    return null;
   }}});
 });
}

test('recompensa e voz escolhidas sobrevivem a salvar e reabrir — sem virar “Qualquer resgate”',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await mockDesktop(page);
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Novo fluxo',exact:true}).click();

 // Escolha: recompensa A, voz Robô, 30 segundos.
 await page.getByRole('combobox',{name:'Evento',exact:true}).selectOption('redemption');
 const reward=page.getByRole('combobox',{name:'Recompensa da Twitch',exact:true});
 await reward.selectOption('r-a');
 await page.locator('.react-flow__node').nth(1).click();
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('voiceFx');
 await page.getByRole('combobox',{name:'Voz do Voicemod',exact:true}).selectOption('robot');
 await page.getByLabel('Duração (segundos)',{exact:true}).fill('30');

 // Os blocos mostram o estado real salvo.
 await expect(page.getByText('Resgate: Voz A')).toBeVisible();
 await expect(page.getByText('Voz: Robô · 30s')).toBeVisible();

 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByRole('dialog')).toHaveCount(0);

 // Reabrir o editor: tudo no lugar.
 await page.getByRole('button',{name:/Editar/}).click();
 await expect(page.getByRole('combobox',{name:'Evento',exact:true})).toHaveValue('redemption');
 await expect(page.getByRole('combobox',{name:'Recompensa da Twitch',exact:true})).toHaveValue('r-a');
 await expect(page.getByText('Resgate: Voz A')).toBeVisible();
 await page.locator('.react-flow__node').nth(1).click();
 await expect(page.getByRole('combobox',{name:'Voz do Voicemod',exact:true})).toHaveValue('robot');
 await expect(page.getByLabel('Duração (segundos)',{exact:true})).toHaveValue('30');

 // Atualizar a lista não apaga a seleção.
 await page.locator('.react-flow__node').first().click();
 await page.getByRole('button',{name:'Atualizar lista',exact:true}).click();
 await expect(page.getByRole('combobox',{name:'Recompensa da Twitch',exact:true})).toHaveValue('r-a');
 await page.screenshot({path:join(OUT,'fluxo-resgate-voz-salvos.png'),fullPage:true});

 // Recompensa fora da lista: o ID fica e a seleção não se amplia.
 await page.evaluate(()=>sessionStorage.setItem('rewards','sem-a'));
 await page.getByRole('button',{name:'Atualizar lista',exact:true}).click();
 await expect(page.getByRole('combobox',{name:'Recompensa da Twitch',exact:true})).toHaveValue('r-a','o vínculo salvo continua valendo');
 await expect(page.getByText(/O resgate salvo não está na lista atual/)).toBeVisible();
 await expect(page.locator('option[value="r-a"]')).toHaveText(/Resgate guardado fora desta lista/);
 await expect(page.getByRole('combobox',{name:'Evento',exact:true})).toHaveValue('redemption','nada vira “Qualquer resgate” sozinho');
 await page.screenshot({path:join(OUT,'fluxo-resgate-fora-da-lista.png'),fullPage:true});
 await page.evaluate(()=>sessionStorage.setItem('rewards',''));
 expect(errors,'nenhuma exceção na página').toEqual([]);
});

test('duplicar cria cópia independente, desativada, sem mexer na original',async({page})=>{
 await mockDesktop(page);
 await page.goto('/');
 await page.locator('.sidebar').getByRole('button',{name:'Automações',exact:true}).click();
 await page.getByRole('button',{name:'Novo fluxo',exact:true}).click();
 await page.getByLabel('Nome do fluxo',{exact:true}).fill('Resgate da voz');
 await page.getByRole('combobox',{name:'Evento',exact:true}).selectOption('redemption');
 await page.getByRole('combobox',{name:'Recompensa da Twitch',exact:true}).selectOption('r-a');
 await page.locator('.react-flow__node').nth(1).click();
 await page.getByRole('combobox',{name:'Tipo de etapa',exact:true}).selectOption('voiceFx');
 await page.getByRole('combobox',{name:'Voz do Voicemod',exact:true}).selectOption('robot');
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByRole('dialog')).toHaveCount(0);

 // Duplicar abre a cópia no editor, já desativada e com o conteúdo preservado.
 await page.getByRole('button',{name:/Duplicar/}).click();
 await expect(page.getByRole('dialog')).toBeVisible();
 await expect(page.getByLabel('Nome do fluxo',{exact:true})).toHaveValue('Cópia de Resgate da voz');
 await expect(page.getByText(/desativada/).first()).toBeVisible();
 await expect(page.getByText('Resgate: Voz A')).toBeVisible('a cópia preserva o vínculo');
 await page.screenshot({path:join(OUT,'fluxo-duplicar-copia-desativada.png'),fullPage:true});

 // Mudar recompensa e voz na cópia não toca a original.
 await page.getByRole('combobox',{name:'Recompensa da Twitch',exact:true}).selectOption('r-b');
 await page.locator('.react-flow__node').nth(1).click();
 await page.getByRole('combobox',{name:'Voz do Voicemod',exact:true}).selectOption('alien');
 await page.getByRole('button',{name:'Salvar fluxo',exact:true}).click();
 await expect(page.getByRole('dialog')).toHaveCount(0);

 // A original segue com a recompensa e a voz de antes.
 await page.getByRole('button',{name:/Editar Resgate da voz/}).click();
 await expect(page.getByRole('combobox',{name:'Recompensa da Twitch',exact:true})).toHaveValue('r-a');
 await page.locator('.react-flow__node').nth(1).click();
 await expect(page.getByRole('combobox',{name:'Voz do Voicemod',exact:true})).toHaveValue('robot');
 await page.getByRole('button',{name:'Fechar',exact:true}).click();

 // A cópia nasce desativada e não dispara até ser ligada.
 await expect(page.getByRole('switch',{name:'Ativar Cópia de Resgate da voz'})).toHaveAttribute('aria-checked','false');
 await expect(page.getByRole('switch',{name:'Ativar Resgate da voz'})).toHaveAttribute('aria-checked','true');
 await page.screenshot({path:join(OUT,'fluxo-duplicar-lista.png'),fullPage:true});
});
