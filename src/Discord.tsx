import {useCallback,useEffect,useState} from 'react';
import {RefreshCw,Save,Search,Download,Plug,Power,Gift,Cake,Link2,Terminal,ShieldAlert,Undo2,Send,Unlink} from 'lucide-react';
import {api,errorText} from './api';
import {Card,Field,Tag,Toggle,Busy,Empty} from './components';

type Any=Record<string,any>;
type Opt={id:string;label:string};
type Status={status:string;running:boolean;enabled:boolean;hasToken:boolean;config:Any;guilds:Any[];guildId:string;channels:Any[];roles:Any[];moduleEnabled:boolean;platform:string;auditCount:number};

const blank:Any={enabled:false,guildId:'',logChannelId:'',autoroleId:'',counterChannelId:'',counterLabel:'membros',appId:'',slowmodeSeconds:0,roleMap:{moderator:'',subscriber:''},intents:{members:true,content:true},automod:{enabled:false},mirror:{enabled:false,toDiscord:true,toTwitch:true,channelId:''},welcome:{enabled:false,channelId:'',text:''},goodbye:{enabled:false,channelId:'',text:''},notify:{enabled:false,channelId:''},birthday:{enabled:false,channelId:'',text:''},xp:{enabled:false,perMessage:15},giveaway:{enabled:false,minutes:10}};

const merge=(base:Any,over:Any|undefined):Any=>{
 const out:Any={...base,...(over||{})};
 for(const k of Object.keys(base)){
  if(base[k]&&typeof base[k]==='object'&&!Array.isArray(base[k]))out[k]={...base[k],...((over&&over[k])||{})};
 }
 return out;
};
const arr=(v:Any|null|undefined):Any[]=>Array.isArray(v)?v:[];
const texts=(v:Any[])=>v.filter(c=>Number(c.type)===0||Number(c.type)===5);
function download(name:string,text:string){
 const blob=new Blob([text],{type:'text/csv;charset=utf-8'});
 const url=URL.createObjectURL(blob);
 const a=document.createElement('a');a.href=url;a.download=name;
 document.body.appendChild(a);a.click();a.remove();URL.revokeObjectURL(url);
}

function Pick({label,value,options,onChange,hint}:{label:string;value:string;options:Opt[];onChange:(v:string)=>void;hint?:string}){
 if(!options.length)return <Field label={label} hint={hint}><input value={value} placeholder="ID do Discord" onChange={e=>onChange(e.target.value)}/></Field>;
 return <Field label={label} hint={hint}><select value={value} onChange={e=>onChange(e.target.value)}><option value="">— nenhum —</option>{options.map(o=><option key={o.id} value={o.id}>{o.label}</option>)}{value&&!options.some(o=>o.id===value)&&<option value={value}>{value}</option>}</select></Field>;
}

export default function Discord({profileId,blocklist,notify}:{profileId:string;blocklist:string[];notify:(s:string)=>void}){
 const [st,setSt]=useState<Status|null>(null);
 const [cfg,setCfg]=useState<Any>(blank);
 const [token,setToken]=useState('');
 const [busy,setBusy]=useState('');
 const [members,setMembers]=useState<Any[]>([]);
 const [q,setQ]=useState('');
 const [target,setTarget]=useState<Any|null>(null);
 const [reason,setReason]=useState('');
 const [warns,setWarns]=useState<Any[]>([]);
 const [audit,setAudit]=useState<Any[]>([]);
 const [rank,setRank]=useState<Any[]>([]);
 const [birthdays,setBirthdays]=useState<Any[]>([]);
 const [bday,setBday]=useState({target:'',date:''});
 const [gives,setGives]=useState<Any[]>([]);
 const [give,setGive]=useState({channel:'',prize:'',minutes:10,winners:1});
 const [links,setLinks]=useState<Any[]>([]);
 const [linkTarget,setLinkTarget]=useState('');
 const [lastCode,setLastCode]=useState('');

 const call=useCallback(async(op:string,args:Any={},ok?:string):Promise<Any|null>=>{
  setBusy(op);
  try{const r=await api<Any>(op,{profileId,...args});if(ok)notify(ok);return r}
  catch(e){notify(errorText(e));return null}
  finally{setBusy('')}
 },[profileId,notify]);

 const action=useCallback(async(args:Any,ok?:string)=>call('discord.action',args,ok),[call]);

 const loadLists=useCallback(async()=>{
  setAudit(arr(await action({action:'audit',limit:60})));
  setRank(arr(await action({action:'xpRank',limit:20})));
  setBirthdays(arr(await action({action:'birthdayList'})));
  setGives(arr(await action({action:'giveawayList'})));
  const l=await action({action:'linkList'});
  setLinks(arr(l&&l.links));
 },[action]);

 const refresh=useCallback(async()=>{
  const r=await call('discord.status');
  if(!r)return;
  setSt(r as unknown as Status);setCfg(merge(blank,r.config));
 },[call]);

 useEffect(()=>{void refresh();void loadLists();},[refresh,loadLists]);

 const channels:Opt[]=texts(arr(st&&st.channels)).map(c=>({id:String(c.id),label:'#'+String(c.name)}));
 const roleOptions:Opt[]=arr(st&&st.roles).map(r=>({id:String(r.id),label:String(r.name)}));
 const guildOptions:Opt[]=arr(st&&st.guilds).map(g=>({id:String(g.id),label:String(g.name)}));
 const set=(k:string,v:any)=>setCfg((c:Any)=>({...c,[k]:v}));
 const nest=(k:string,sub:string,v:any)=>setCfg((c:Any)=>({...c,[k]:{...(c[k]||{}),[sub]:v}}));
 const sub=(k:string)=>({...blank[k],...(cfg[k]||{})});
 const nameOf=(m:Any)=>String(m.nick||m.user?.global_name||m.user?.username||m.user?.id||'');

 async function saveToken(){await call('secret.save',{key:'discord_token',value:token},'Token do bot salvo no cofre do sistema.');setToken('')}
 async function saveCfg(){const r=await call('discord.save',{config:cfg},'Configuração do Discord salva.');if(r!==null)await refresh()}
 async function discover(){const d=await call('discord.discover',{},'Servidor, canais e cargos carregados.');if(d)setSt(s=>s?{...s,guilds:arr(d.guilds),channels:arr(d.channels),roles:arr(d.roles)}:s)}
 async function invite(){const r=await action({action:'invite'});if(r&&r.url)window.open(String(r.url),'_blank','noopener')}
 async function searchMembers(){setMembers(arr(await action({action:'members',q})))}
 async function punish(act:string){
  if(!target)return notify('Escolha um membro primeiro.');
  await action({action:act,target:String(target.user.id),targetName:nameOf(target),reason},'Ação aplicada e registrada na auditoria.');
  await loadLists();
 }
 async function warnMember(){
  if(!target)return notify('Escolha um membro primeiro.');
  await action({action:'warn',target:String(target.user.id),targetName:nameOf(target),reason},'Aviso registrado nas duas casas.');
  await loadWarns();await loadLists();
 }
 async function loadWarns(){
  if(!target)return notify('Escolha um membro primeiro.');
  setWarns(arr(await action({action:'warnlist',target:String(target.user.id)})));
 }
 async function exportAudit(){
  const r:unknown=await action({action:'auditExport'});
  if(typeof r==='string'&&r.trim()){download('auditoria-botlive.csv',r);notify('Auditoria exportada em CSV.')}
  else notify('Ainda não há registros para exportar.');
 }
 async function undo(id:string){await action({action:'undo',id},'Ação desfeita no Discord.');await loadLists()}
 async function startGiveaway(){
  await action({action:'giveawayStart',channel:give.channel,prize:give.prize,minutes:Number(give.minutes)||10,winners:Number(give.winners)||1},'Sorteio criado. A reação 🎉 abre as inscrições.');
  setGive(g=>({...g,prize:''}));
  setGives(arr(await action({action:'giveawayList'})));
 }
 async function addBirthday(){
  await action({action:'birthdayAdd',target:bday.target,targetName:bday.target,date:bday.date},'Aniversário registrado.');
  setBday({target:'',date:''});
  setBirthdays(arr(await action({action:'birthdayList'})));
 }
 async function removeBirthday(user:string){
  await action({action:'birthdayRemove',target:user},'Aniversário removido.');
  setBirthdays(arr(await action({action:'birthdayList'})));
 }
 async function createLink(){
  const r=await action({action:'linkCreate',target:linkTarget},'Código gerado. Peça para digitar !vincular no chat da Twitch.');
  if(r&&r.code)setLastCode(String(r.code));
  setLinks(arr((await action({action:'linkList'}))?.links));
 }
 async function removeLink(id:string){
  await action({action:'linkRemove',target:id},'Vínculo removido.');
  setLinks(arr((await action({action:'linkList'}))?.links));
 }
 async function registerSlash(){await action({action:'slash'},'Comandos slash registrados no servidor.')}
 async function toggleStatus(){
  if(st&&st.running)await call('discord.disconnect',{},'Bot do Discord desconectado.');
  else await call('discord.connect',{},'Bot do Discord conectado.');
  await refresh();
 }

 const connectionTag=st&&st.running
  ?(st.status==='online'?<Tag>Conectado</Tag>:<Tag>Conectando</Tag>)
  :<Tag color="gray">Desconectado</Tag>;

 return <div className="discord-screen">
 <Card title="Conexão com o Discord">
  <p className="help">Use o token de um bot criado no portal do desenvolvedor do Discord. O token fica no cofre do sistema e nunca é gravado nas configurações exportadas.</p>
  <Field label="Token do bot" hint="Formato 1234567890.abcdefghijklmnopqrstuvwxyz. Ele não volta para a tela depois de salvo.">
   <input type="password" autoComplete="off" value={token} placeholder={st&&st.hasToken?'Token já salvo':'Cole o token do bot'} onChange={e=>setToken(e.target.value)}/>
  </Field>
  <div className="row">
   <button disabled={!token.trim()} onClick={saveToken}><Save size={15}/>Salvar token</button>
   <Busy busy={busy==='discord.discover'}><button onClick={discover}><Search size={15}/>Descobrir servidor e canais</button></Busy>
   <Busy busy={busy==='discord.connect'||busy==='discord.disconnect'}><button onClick={toggleStatus}>{st&&st.running?<><Power size={15}/>Desconectar</>:<><Plug size={15}/>Conectar</>}</button></Busy>
   <button onClick={invite}>Convite do bot</button>
  </div>
  <div className="row between"><div className="row">{connectionTag}{st&&st.enabled?<Tag color="green">Bot ativo</Tag>:<Tag color="gray">Bot inativo</Tag>}</div><p className="help">{st&&st.auditCount?`${st.auditCount} registros de auditoria`:'Sem registros de auditoria'}</p></div>
 </Card>

 <div className="two-columns">
  <Card title="Servidor e canais">
   <Pick label="Servidor do Discord" hint="Escolha um servidor descoberto ou cole o ID do guild." value={String(cfg.guildId||'')} options={guildOptions} onChange={v=>set('guildId',v)}/>
   <Pick label="Canal de logs de auditoria" hint="Entradas, saídas, banimentos e ações do bot." value={String(cfg.logChannelId||'')} options={channels} onChange={v=>set('logChannelId',v)}/>
   <Pick label="Cargo automático na entrada" value={String(cfg.autoroleId||'')} options={roleOptions} onChange={v=>set('autoroleId',v)}/>
   <div className="row">
    <Field label="Canal do contador de membros" hint="O nome do canal passa a ser membros-1234."><input value={String(cfg.counterChannelId||'')} placeholder="ID do canal" onChange={e=>set('counterChannelId',e.target.value)}/></Field>
    <Field label="Rótulo do contador"><input value={String(cfg.counterLabel||'membros')} onChange={e=>set('counterLabel',e.target.value)}/></Field>
   </div>
   <div className="row">
    <Pick label="Cargo de moderador no mapa" value={String(cfg.roleMap?.moderator||'')} options={roleOptions} onChange={v=>nest('roleMap','moderator',v)}/>
    <Pick label="Cargo de assinante no mapa" value={String(cfg.roleMap?.subscriber||'')} options={roleOptions} onChange={v=>nest('roleMap','subscriber',v)}/>
   </div>
   <div className="list-row"><div><strong>Intent de membros</strong><small>Ative “Server Members Intent” no portal do Discord.</small></div><Toggle label="Intent de membros" checked={sub('intents').members!==false} onChange={v=>nest('intents','members',v)}/></div>
   <div className="list-row"><div><strong>Intent de conteúdo de mensagens</strong><small>Sem ela o bot não lê o texto para moderar.</small></div><Toggle label="Intent de conteúdo" checked={sub('intents').content!==false} onChange={v=>nest('intents','content',v)}/></div>
   <div className="list-row"><div><strong>Ativar o bot do Discord</strong><small>Conexão, respostas e moderação neste servidor.</small></div><Toggle label="Ativar bot do Discord" checked={cfg.enabled===true} onChange={v=>set('enabled',v)}/></div>
   <div className="row">
    <Busy busy={busy==='discord.save'}><button className="primary" onClick={saveCfg}><Save size={15}/>Salvar configuração</button></Busy>
    <button onClick={()=>setCfg(merge(blank,st?st.config:cfg))}>Restaurar o que está salvo</button>
   </div>
  </Card>

  <Card title="Entrada, saída e espelho">
   <div className="list-row"><div><strong>Mensagem de boas-vindas</strong><small>Aceita {'{user}'} e {'{name}'}.</small></div><Toggle label="Boas-vindas" checked={sub('welcome').enabled===true} onChange={v=>nest('welcome','enabled',v)}/></div>
   <Pick label="Canal de boas-vindas" value={String(sub('welcome').channelId||'')} options={channels} onChange={v=>nest('welcome','channelId',v)}/>
   <Field label="Texto de boas-vindas"><input value={String(sub('welcome').text||'')} placeholder="Bem-vindo(a) ao servidor, {user}!" onChange={e=>nest('welcome','text',e.target.value)}/></Field>
   <div className="list-row"><div><strong>Mensagem de despedida</strong><small>Quando alguém sai do servidor.</small></div><Toggle label="Despedida" checked={sub('goodbye').enabled===true} onChange={v=>nest('goodbye','enabled',v)}/></div>
   <Pick label="Canal de despedida" value={String(sub('goodbye').channelId||'')} options={channels} onChange={v=>nest('goodbye','channelId',v)}/>
   <Field label="Texto de despedida"><input value={String(sub('goodbye').text||'')} placeholder="Até logo, {name}." onChange={e=>nest('goodbye','text',e.target.value)}/></Field>
   <div className="list-row"><div><strong>Espelho do chat</strong><small>Discord e Twitch no mesmo fluxo de conversa.</small></div><Toggle label="Espelho do chat" checked={sub('mirror').enabled===true} onChange={v=>nest('mirror','enabled',v)}/></div>
   <Pick label="Canal do espelho" value={String(sub('mirror').channelId||'')} options={channels} onChange={v=>nest('mirror','channelId',v)}/>
   <div className="list-row"><div><strong>Espelhar Twitch para o Discord</strong></div><Toggle label="Espelhar Twitch para o Discord" checked={sub('mirror').toDiscord===true} onChange={v=>nest('mirror','toDiscord',v)}/></div>
   <div className="list-row"><div><strong>Espelhar Discord para a Twitch</strong></div><Toggle label="Espelhar Discord para a Twitch" checked={sub('mirror').toTwitch===true} onChange={v=>nest('mirror','toTwitch',v)}/></div>
   <div className="list-row"><div><strong>Notificações da Twitch</strong><small>Seguidores, inscrições, bits e raids.</small></div><Toggle label="Notificações da Twitch" checked={sub('notify').enabled===true} onChange={v=>nest('notify','enabled',v)}/></div>
   <Pick label="Canal de notificações" value={String(sub('notify').channelId||'')} options={channels} onChange={v=>nest('notify','channelId',v)}/>
  </Card>
 </div>

 <div className="two-columns">
  <Card title="Moderação em dupla">
   <div className="list-row"><div><strong>Auto-moderação do Discord</strong><small>Mesmas regras da Twitch: {blocklist.length} expressões bloqueadas, anti-link e anti-repetição.</small></div><Toggle label="Auto-moderação do Discord" checked={sub('automod').enabled===true} onChange={v=>nest('automod','enabled',v)}/></div>
   <p className="help">A mensagem violadora é apagada e registrada na auditoria. O aviso, o timeout e o banimento continuam saindo pela página Comunidade e valem nas duas casas quando a identidade está vinculada. As entradas e saídas vão para o canal de logs escolhido no cartão “Servidor e canais”.</p>
   <Field label="Modo lento do servidor (segundos)" hint="0 desativa; o máximo é 21600 segundos (6 horas)."><input type="number" min={0} max={21600} value={Number(cfg.slowmodeSeconds)||0} onChange={e=>set('slowmodeSeconds',Number(e.target.value))}/></Field>
  </Card>

  <Card title="Ranking de XP">
   <p className="help">Quem conversa no Discord ganha XP a cada mensagem, com intervalo de um minuto por membro.</p>
   <div className="list-row"><div><strong>XP e níveis</strong><small>Pontos por mensagem no servidor.</small></div><Toggle label="XP e níveis" checked={sub('xp').enabled===true} onChange={v=>nest('xp','enabled',v)}/></div>
   <Field label="XP por mensagem"><input type="number" min={1} max={500} value={Number(sub('xp').perMessage)||15} onChange={e=>nest('xp','perMessage',Number(e.target.value))}/></Field>
   <Busy busy={busy==='discord.action'}><button onClick={()=>{void (async()=>setRank(arr(await action({action:'xpRank',limit:20}))))()}}><RefreshCw size={15}/>Atualizar ranking</button></Busy>
   {rank.length?rank.map((r,i)=><div className="list-row" key={String(r.id)}><div><span>{i+1}º {String(r.name)}</span>{Number(r.next)>Number(r.xp)&&<small>Faltam {Number(r.next)-Number(r.xp)} XP para o próximo nível</small>}</div><strong>Nível {Number(r.level)} · {Number(r.xp)} XP</strong></div>):<p className="help">Ninguém tem XP ainda neste servidor.</p>}
  </Card>
 </div>

 <div className="two-columns">
  <Card title="Sorteios">
   <div className="list-row"><div><strong>Sorteios liberados</strong><small>Participação pela reação 🎉.</small></div><Toggle label="Sorteios liberados" checked={sub('giveaway').enabled===true} onChange={v=>nest('giveaway','enabled',v)}/></div>
   <Field label="Duração padrão (minutos)"><input type="number" min={1} max={10080} value={Number(sub('giveaway').minutes)||10} onChange={e=>nest('giveaway','minutes',Number(e.target.value))}/></Field>
   <Pick label="Canal dos sorteios" value={give.channel} options={channels} onChange={v=>setGive(g=>({...g,channel:v}))}/>
   <div className="row">
    <Field label="Prêmio do sorteio"><input value={give.prize} placeholder="Nitro, jogo, destaque…" onChange={e=>setGive(g=>({...g,prize:e.target.value}))}/></Field>
    <Field label="Minutos do sorteio"><input type="number" min={1} max={10080} value={give.minutes} onChange={e=>setGive(g=>({...g,minutes:Number(e.target.value)}))}/></Field>
    <Field label="Quantos vencedores"><input type="number" min={1} max={20} value={give.winners} onChange={e=>setGive(g=>({...g,winners:Number(e.target.value)}))}/></Field>
   </div>
   <button className="primary" disabled={!give.channel||!give.prize.trim()||busy==='discord.action'} onClick={startGiveaway}><Gift size={15}/>Criar sorteio</button>
   {gives.length?gives.map(g=><div className="list-row" key={String(g.id)}>
    <div><span>{String(g.prize)}</span><small>{g.status==='open'?'aberto':'encerrado'} · {arr(g.entries).length} participantes</small></div>
    <div className="row">
     {g.status==='open'&&<button onClick={()=>{void (async()=>{await action({action:'giveawayEnd',id:String(g.id)},'Sorteio encerrado e vencedores anunciados.');setGives(arr(await action({action:'giveawayList'})))})()}}>Encerrar</button>}
     <button onClick={()=>{void (async()=>{await action({action:'giveawayReroll',id:String(g.id)},'Vencedores sorteados de novo.');setGives(arr(await action({action:'giveawayList'})))})()}}>Sortear de novo</button>
    </div>
   </div>):<p className="help">Nenhum sorteio criado.</p>}
  </Card>

  <Card title="Aniversariantes">
   <div className="list-row"><div><strong>Mensagens de aniversário</strong><small>Um cumprimento no dia certo.</small></div><Toggle label="Mensagens de aniversário" checked={sub('birthday').enabled===true} onChange={v=>nest('birthday','enabled',v)}/></div>
   <Pick label="Canal dos aniversários" value={String(sub('birthday').channelId||'')} options={channels} onChange={v=>nest('birthday','channelId',v)}/>
   <Field label="Texto do cumprimento"><input value={String(sub('birthday').text||'')} placeholder="Feliz aniversário, {name}! Você está com {age} anos." onChange={e=>nest('birthday','text',e.target.value)}/></Field>
   <div className="row">
    <Field label="ID do aniversariante"><input value={bday.target} placeholder="ID no Discord" onChange={e=>setBday(b=>({...b,target:e.target.value}))}/></Field>
    <Field label="Data do aniversário" hint="DD/MM ou DD/MM/AAAA."><input value={bday.date} placeholder="25/12/1990" onChange={e=>setBday(b=>({...b,date:e.target.value}))}/></Field>
   </div>
   <button disabled={!bday.target.trim()||!bday.date.trim()} onClick={addBirthday}><Cake size={15}/>Registrar aniversário</button>
   {birthdays.length?birthdays.map((b,i)=><div className="list-row" key={String(b.user)+i}>
    <div><span>{String(b.name||b.user)}</span><small>{String(b.day).padStart(2,'0')}/{String(b.month).padStart(2,'0')}{b.year?' · '+String(b.year):''}</small></div>
    <button aria-label={'Remover aniversário de '+String(b.name||b.user)} onClick={()=>{void removeBirthday(String(b.user))}}>Remover</button>
   </div>):<p className="help">Nenhum aniversário registrado.</p>}
  </Card>
 </div>

 <div className="two-columns">
  <Card title="Identidade unificada Twitch e Discord">
   <p className="help">Um código de seis dígitos liga quem fala na Twitch a quem reage no Discord. Depois do vínculo, avisos, punições e reconhecimento valem nas duas casas.</p>
   <div className="row">
    <Field label="ID do membro vinculado"><input value={linkTarget} placeholder="ID no Discord" onChange={e=>setLinkTarget(e.target.value)}/></Field>
    <button disabled={!linkTarget.trim()} onClick={createLink}><Link2 size={15}/>Gerar código</button>
   </div>
   {lastCode&&<p className="help">Código gerado: <strong>{lastCode}</strong> — peça para digitar <code>!vincular {lastCode}</code> no chat da Twitch.</p>}
   {links.length?links.map((l,i)=><div className="list-row" key={i}>
    <div><span>Discord {String(l.discordId)}</span><small>Twitch {String(l.twitchId)}</small></div>
    <button aria-label={'Remover vínculo '+String(l.discordId)} onClick={()=>{void removeLink(String(l.discordId))}}><Unlink size={15}/>Remover</button>
   </div>):<p className="help">Nenhum vínculo criado ainda.</p>}
  </Card>

  <Card title="Comandos slash próprios">
   <p className="help">Publica /painel, /convite, /limpar, /ban, /mute, /warn, /avisos, /xp, /ranking, /aniversario, /vincular e /sorteio dentro do servidor escolhido.</p>
   <div className="row">
    <Busy busy={busy==='discord.action'}><button className="primary" onClick={registerSlash}><Terminal size={15}/>Registrar comandos slash</button></Busy>
    <button onClick={invite}><Send size={15}/>Abrir convite do bot</button>
   </div>
   <p className="help">Quem invoca um comando que altera o servidor precisa de permissão de moderação no Discord.</p>
  </Card>
 </div>

 <Card title="Moderação no servidor">
  <div className="row">
   <Field label="Buscar membro no Discord"><input value={q} placeholder="Nome ou ID" onChange={e=>setQ(e.target.value)}/></Field>
   <button onClick={searchMembers}><Search size={15}/>Buscar</button>
  </div>
  {members.length?<div className="row">{members.slice(0,8).map(m=><button key={String(m.user.id)} className={target&&target.user.id===m.user.id?'selected':''} onClick={()=>setTarget(m)}>{nameOf(m)} · {String(m.user.id)}</button>)}</div>:<p className="help">Busque um membro para liberar as ações.</p>}
  {target&&<div className="row between"><strong>Alvo: {nameOf(target)} ({String(target.user.id)})</strong><button onClick={()=>{setTarget(null);setWarns([])}}>Limpar alvo</button></div>}
  <Field label="Motivo registrado na auditoria"><input value={reason} onChange={e=>setReason(e.target.value)} placeholder="Regra violada, contexto da ação…"/></Field>
  <div className="row">
   <button disabled={!target} onClick={()=>punish('timeout')}>Timeout</button>
   <button disabled={!target} onClick={()=>punish('kick')}>Expulsar</button>
   <button disabled={!target} onClick={()=>punish('ban')}>Banir</button>
   <button disabled={!target} onClick={()=>punish('unban')}>Desbanir</button>
   <Busy busy={busy==='discord.action'}><button className="primary" disabled={!target} onClick={warnMember}><ShieldAlert size={15}/>Avisar nos dois chats</button></Busy>
   <button disabled={!target} onClick={loadWarns}>Ver avisos</button>
  </div>
  {warns.length>0&&warns.map((w,i)=><div className="list-row" key={i}><div><span>Aviso de {String(w.targetName||w.target)}</span><small>{String(w.reason||'sem motivo')}</small></div><Tag color="gray">{String(w.date||'')}</Tag></div>)}
 </Card>

 <Card title="Auditoria única" action={<div className="row"><button onClick={exportAudit}><Download size={15}/>Exportar CSV</button><button onClick={()=>{void loadLists()}}><RefreshCw size={15}/>Atualizar</button></div>}>
  <p className="help">Cada ação fica registrada com a plataforma em que aconteceu. O desfazer aparece só quando existe operação reversível na API do Discord.</p>
  {audit.length?audit.map(a=><div className="list-row" key={String(a.id)}>
   <div><span>{String(a.action)} · {String(a.targetName||a.target)}</span><small>{arr(a.platforms).join(' + ')}{a.reason?' · '+String(a.reason):''}{a.at?' · '+String(a.at):''}</small></div>
   {a.undoable&&<button aria-label={'Desfazer '+String(a.action)+' de '+String(a.targetName||a.target)} onClick={()=>{void undo(String(a.id))}}><Undo2 size={15}/>Desfazer</button>}
  </div>):<Empty title="Ainda não há registros">As ações feitas pelo bot aparecem aqui, com o caminho de desfazer quando ele existe.</Empty>}
 </Card>
 </div>;
}
