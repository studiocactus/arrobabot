import {useCallback,useEffect,useRef,useState} from 'react';
import {Play,Square,Trash2,Plus,Copy,Check,RefreshCw,Pencil} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {Card,Field} from './components';
import './overlay-studio.css';
import OverlayPreview,{type StageBg} from './OverlayPreview';
import {TEMPLATE_IDS,TEMPLATES,defaultDesign,fontGroups,normalizeDesign,type Align,type Layout,type OverlayDesign,type VAlign} from './overlayDesign';

type GoalDef={title:string;target:number;fontFamily?:string};
type Config={enabled?:boolean;clientId?:string;activeGoal?:string;goals?:Record<string,GoalDef>};
type Data={current?:number;active_goal?:string;last_error?:string};
type Info={token:string;port:number;url:string};
type Sound={id:string;name:string};

const LAYOUTS:[Layout,string][]=[['bar','Barra'],['row','Linha'],['panel','Painel'],['ring','Anel']];
const ALIGNS:[Align,string][]=[['left','Esquerda'],['center','Centro'],['right','Direita']];
const VALIGNS:[VAlign,string][]=[['top','Topo'],['center','Centro'],['bottom','Base']];
const WEIGHTS=[400,500,600,700,800,900];
const BGS:[StageBg,string][]=[['checker','Transparente'],['dark','Cena escura'],['light','Cena clara']];

/** #rgb/#rrggbb/#rrggbbaa → #rrggbb para o seletor de cores do navegador. */
const rgbOf=(h:string):string=>{
 let s=(h||'').replace('#','');
 if(s.length===3||s.length===4)s=[...s].map(c=>c+c).join('');
 s=s.slice(0,6);
 return /^[0-9a-fA-F]{6}$/.test(s)?'#'+s:'#101721';
};
const alphaOf=(h:string):number=>h.length>=9?parseInt(h.slice(7,9),16):255;
const withAlpha=(h:string,a:number)=>{
 const rgb=rgbOf(h);
 return a>=250?rgb:rgb+Math.max(0,Math.min(255,Math.round(a))).toString(16).padStart(2,'0');
};

export default function LivePix({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [clientId,setClientId]=useState('');
 const [clientSecret,setClientSecret]=useState('');
 const [config,setConfig]=useState<Config|null>(null);
 const [data,setData]=useState<Data|null>(null);
 const [design,setDesign]=useState<OverlayDesign>(()=>defaultDesign());
 const [info,setInfo]=useState<Info|null>(null);
 const [saveState,setSaveState]=useState<'idle'|'saving'|'saved'|'error'>('idle');
 const [designError,setDesignError]=useState('');
 const [busy,setBusy]=useState(false);
 const [error,setError]=useState('');
 const [bg,setBg]=useState<StageBg>('checker');
 const [sim,setSim]=useState<number|null>(null);
 const [gainKey,setGainKey]=useState(0);
 const [celebrateKey,setCelebrateKey]=useState(0);
 const [copied,setCopied]=useState(false);
 const [ensaio,setEnsaio]=useState(false);
 const [editing,setEditing]=useState('');
 const [sounds,setSounds]=useState<Sound[]>([]);
 const [burstMeta,setBurstMeta]=useState(false);
 const hydrated=useRef(false);
 const lastSaved=useRef('');
 const ensaioTimer=useRef<number|undefined>(undefined);
 useEffect(()=>()=>window.clearTimeout(ensaioTimer.current),[]);

 const load=useCallback(async()=>{
  if(!desktop)return;
  try{const c=await api<Config>('module.config.get',{profileId,key:'livepix'});setConfig(c||{});if(c?.clientId)setClientId(c.clientId)}catch{}
  try{setData(await api<Data>('module.config.get',{profileId,key:'livepix_data'}))}catch{}
  // As duas ops do livepix exigem profileId — sem ele o backend responde
  // "Perfil não encontrado" e o endereço nunca sairia do "Gerando…".
  try{const d=normalizeDesign(await api<unknown>('livepix.design_get',{profileId}));lastSaved.current=JSON.stringify(d);setDesign(d)}catch{}
  try{const i=await api<Info|null>('livepix.overlay_info',{profileId});setInfo(i&&i.token?i:null)}catch{}
  // Sons da Meta: a prévia toca pelo mesmo caminho da live (saída única).
  try{const g=await api<{assets:{id:string;kind:string;name:string}[]}>('chatExtras.get',{profileId});setSounds((g.assets||[]).filter(a=>a.kind==='sound'))}catch{}
  try{const b=await api<{events?:{meta?:boolean}}|null>('module.config.get',{profileId,key:'emoteBurst'});setBurstMeta(b?.events?.meta!==false)}catch{}
  hydrated.current=true;
 },[profileId,desktop]);
 useEffect(()=>{if(desktop)void load()},[load]);

 /** Salva o interruptor da meta na mesma configuração da explosão (tela Alertas). */
 const saveBurstMeta=async(v:boolean)=>{
  setBurstMeta(v);
  try{
   const cur=await api<{enabled?:boolean;events?:Record<string,boolean>}|null>('module.config.get',{profileId,key:'emoteBurst'});
   // Ligar a meta também liga a explosão; desligar a meta não apaga as dos alertas.
   await api('module.config',{profileId,key:'emoteBurst',value:{...(cur||{}),enabled:v?true:cur?.enabled===true,events:{...cur?.events,meta:v}}});
  }catch(e){setBurstMeta(!v);notify(errorText(e))}
 };

 // Autosave: o Estúdio grava com atraso curto e o backend publica a mudança por
 // WebSocket, então o que aparece na prévia é exatamente o que o OBS desenha.
 useEffect(()=>{
  if(!desktop||!hydrated.current)return;
  const key=JSON.stringify(design);
  if(key===lastSaved.current)return;
  setSaveState('saving');
  let cancelled=false;
  const timer=setTimeout(async()=>{
   try{
    await api('livepix.design_save',{profileId,design});
    lastSaved.current=key;
    if(!cancelled)setSaveState('saved');
   }catch(e){if(!cancelled){setSaveState('error');setDesignError(errorText(e))}}
  },700);
  return()=>{cancelled=true;clearTimeout(timer)};
 },[design,profileId,desktop]);

 const call=async(op:string,args:Record<string,unknown>)=>{
  setBusy(true);setError('');
  try{await api(op,{profileId,...args});await load()}catch(e){setError(errorText(e))}
  setBusy(false);
 };

 const saveCreds=async()=>{
  setBusy(true);setError('');
  try{
   await api('livepix.save_credentials',{profileId,clientId:clientId.trim(),clientSecret});
   if(clientSecret)setClientSecret('');
   await load();
   notify('Credenciais salvas no cofre do sistema.');
  }catch(e){setError(errorText(e))}
  setBusy(false);
 };

 const enable=async()=>{
  const goals=config?.goals?Object.values(config.goals):[];
  if(!goals.length){setError('Crie ao menos uma meta antes de ativar.');return}
  await call('livepix.enable',{goals:goals.map(g=>({title:g.title,target:g.target,font_family:g.fontFamily||''}))});
 };

 const copyUrl=async()=>{
  if(!info)return;
  try{
   await navigator.clipboard.writeText(layerUrl());
   setCopied(true);setTimeout(()=>setCopied(false),2200);
   notify('Endereço da camada da meta copiado.');
  }catch{notify('Não foi possível copiar. Selecione o endereço e copie manualmente.')}
 };
 /** Endereço da camada da meta: mesma base do overlay com mode=goal. */
 const layerUrl=()=>info?info.url+'&mode=goal':'';

 const fmt=(cents:number)=>(cents/100).toLocaleString('pt-BR',{minimumFractionDigits:2,maximumFractionDigits:2});
 const active=config?.activeGoal||'';
 const activeGoal=active?config?.goals?.[active]:undefined;
 const showReal=Boolean(config?.enabled&&activeGoal&&activeGoal.target>0);
 const target=showReal?(activeGoal as GoalDef).target:50000;
 const baseCurrent=showReal?(data?.current||0):31000;
 const title=showReal?(activeGoal as GoalDef).title:'Meta do mês';
 const current=sim??baseCurrent;
 const pct=target>0?Math.round(current*100/target):0;

 // Ensaio da Meta: no aplicativo os botões saem pelo backend (mesmo payload,
 // som e explosão do pagamento real, com o selo ENSAIO no overlay). Na prévia
 // web sem aplicativo a simulação fica local, só para conferir o visual.
 const flagEnsaio=()=>{
  setEnsaio(true);
  window.clearTimeout(ensaioTimer.current);
  ensaioTimer.current=window.setTimeout(()=>setEnsaio(false),5000);
 };
 const simulate=async()=>{
  if(desktop){
   try{
    const out=await api<{fired?:boolean;reason?:string;current?:number}>('livepix.simulate',{profileId,mode:'donate'});
    if(out?.fired===false){notify(out.reason||'Não foi possível simular a doação.');return}
    if(typeof out?.current==='number'){setSim(out.current);setGainKey(k=>k+1);flagEnsaio()}
   }catch(e){notify(errorText(e))}
   return;
  }
  if(current>=target)setSim(Math.round(target*0.3));
  else setSim(Math.min(Math.round(target*0.97),current+Math.max(Math.round(target*0.17),1000)));
  setGainKey(k=>k+1);
  previewSound(design.soundDonate);
 };
 const celebrate=async()=>{
  if(desktop){
   try{
    const out=await api<{fired?:boolean;reason?:string;current?:number}>('livepix.simulate',{profileId,mode:'celebrate'});
    if(out?.fired===false){notify(out.reason||'Não foi possível simular a celebração.');return}
    if(typeof out?.current==='number'){setSim(out.current);setCelebrateKey(k=>k+1);flagEnsaio()}
   }catch(e){notify(errorText(e))}
   return;
  }
  setSim(target);setCelebrateKey(k=>k+1);previewSound(design.soundCelebrate);
 };
 const previewSound=(asset:string)=>{
  if(!desktop||!asset)return;
  // Mesmo caminho da live: sai pela fonte do OBS quando conectada; sem fonte,
  // o aplicativo toca na saída padrão.
  void api('audio.preview',{profileId,asset,volume:1}).catch(e=>notify(errorText(e)));
 };

 const set=(patch:Partial<OverlayDesign>)=>setDesign(d=>({...d,...patch}));

 return <div>
  <Card title="LivePix — Meta de doações" action={<span className={'status-pill '+(config?.enabled?'on':'off')}><span className="status-dot" aria-hidden="true"></span>{config?.enabled?'Ativa':'Desligada'}</span>}>
   <p className="help">Conecte sua conta LivePix para exibir uma meta no OBS. O BotLive consulta a API a cada 15 segundos e envia o estado por WebSocket local (porta 9876). Desligar e ligar a integração <strong>não apaga o progresso</strong>: o contador só muda com dinheiro entrando ou com o botão Reiniciar contador nas Metas.</p>

   <div className="form-grid">
    <Field label="Client ID" hint="UUID da aplicação criada no painel da LivePix."><input value={clientId} onChange={e=>setClientId(e.target.value)} placeholder="Ex.: 12345678-abcd-..."/></Field>
    <Field label="Client Secret" hint="Gerado no painel. Fica no cofre do sistema."><input type="password" autoComplete="new-password" value={clientSecret} onChange={e=>setClientSecret(e.target.value)} placeholder="Deixe vazio para manter o segredo atual"/></Field>
   </div>
   <button disabled={busy||(!clientId.trim()&&!clientSecret)} onClick={()=>void saveCreds()}>Salvar no cofre</button>

   {data?.last_error&&<div className="inline-warning" role="alert"><strong>Problema na integração:</strong><p>{data.last_error}</p></div>}
   {error&&<p role="alert" className="inline-error">{error}</p>}

   <div className="list-row" style={{marginTop:16}}>
    <div><strong>Estado</strong><small>{config?.enabled?'Integração ativa':'Integração desativada'}</small></div>
    {config?.enabled
     ?<button className="danger" disabled={busy} onClick={()=>void call('livepix.disable',{})}><Square size={15}/>Desativar</button>
     :<button className="primary" disabled={busy} onClick={()=>void enable()}><Play size={15}/>Ativar</button>}
   </div>

   {activeGoal&&data&&<div style={{margin:'12px 0'}}>
    <p className="help" style={{margin:0}}><strong>{activeGoal.title}</strong> — {fmt(data.current||0)} / {fmt(activeGoal.target)} BRL ({target>0?Math.round((data.current||0)*100/target):0}%)</p>
   </div>}
  </Card>

  <Card title="Estúdio do overlay" action={<span className={'lp-save lp-save--'+saveState}>{saveState==='saving'?'Salvando…':saveState==='saved'?'Salvo':saveState==='error'?'Não salvou':'Alterações salvam sozinhas'}</span>}>
   <p className="help">Escolha um modelo e ajuste cores, bordas, fontes e efeitos. A prévia é o quadro de 1920×1080 do OBS em miniatura — o que você vê é o que o OBS desenha, e toda alteração chega ao vivo sem recarregar a fonte.</p>
   {designError&&<p role="alert" className="inline-error">{designError}</p>}

   <div className="lp-studio">
    <div className="lp-preview">
     <div className="lp-stagebar">
      <div className="lp-segrow" role="group" aria-label="Fundo da prévia">
       {BGS.map(([v,l])=><button key={v} type="button" className={'lp-seg'+(bg===v?' selected':'')} aria-pressed={bg===v} onClick={()=>setBg(v)}>{l}</button>)}
      </div>
      <div className="lp-segrow">
       <button type="button" className="lp-seg" onClick={simulate}>Simular doação</button>
       <button type="button" className="lp-seg" onClick={celebrate}>Celebração</button>
       {sim!==null&&<button type="button" className="text-button" onClick={()=>setSim(null)}>usar dados reais</button>}
      </div>
     </div>

     <div className="lp-stagebox">
      <OverlayPreview design={design} model={{title,current,target}} bg={bg} gainKey={gainKey} celebrateKey={celebrateKey}/>
      {ensaio&&<span className="lp-ensaio">Ensaio</span>}
     </div>

     <p className="help">Prévia em escala de {pct}% sobre {fmt(target)} BRL{!showReal&&' — dados de exemplo enquanto a integração está desligada'}.</p>
    </div>

    <div className="lp-controls">
     <div className="lp-group">
      <h4>Modelo</h4>
      <div className="lp-tplrow">
       {TEMPLATE_IDS.map(id=>{
        const t=TEMPLATES[id];
        return <button key={id} type="button" className={'lp-tpl'+(design.template===id?' selected':'')} aria-pressed={design.template===id}
         onClick={()=>setDesign(d=>({...t.design,soundDonate:d.soundDonate,soundCelebrate:d.soundCelebrate}))}>
         <span className="lp-tpl-swatch" style={{background:`linear-gradient(90deg,${t.design.fillStart},${t.design.fillEnd})`}}></span>
         <strong>{t.name}</strong><small>{t.hint}</small>
        </button>;
       })}
      </div>
     </div>

     <div className="lp-group">
      <h4>Composição</h4>
      <Seg label="Formato" value={design.layout} options={LAYOUTS} onChange={v=>set({layout:v})}/>
      <Seg label="Alinhamento horizontal" value={design.align} options={ALIGNS} onChange={v=>set({align:v})}/>
      <Seg label="Altura na tela" value={design.valign} options={VALIGNS} onChange={v=>set({valign:v})}/>
      <RangeRow label="Largura" min={20} max={100} step={1} value={design.width} fmt={v=>Math.round(v)+'%'} onChange={v=>set({width:v})}/>
     </div>

     <div className="lp-group">
      <h4>Cores</h4>
      <div className="lp-colors">
       <ColorRow label="Gradiente A" value={rgbOf(design.fillStart)} onChange={v=>set({fillStart:v})}/>
       <ColorRow label="Gradiente B" value={rgbOf(design.fillEnd)} onChange={v=>set({fillEnd:v})}/>
       <ColorRow label="Trilho" value={rgbOf(design.track)} onChange={v=>set({track:v})}/>
       <ColorRow label="Texto" value={rgbOf(design.text)} onChange={v=>set({text:v})}/>
       <ColorRow label="Texto secundário" value={rgbOf(design.muted)} onChange={v=>set({muted:v})}/>
       <ColorRow label="Cor da borda" value={rgbOf(design.border)} onChange={v=>set({border:v})}/>
       <ColorRow label="Fundo do painel" value={rgbOf(design.panel)} onChange={v=>set({panel:withAlpha(v,alphaOf(design.panel))})}/>
      </div>
      <RangeRow label="Opacidade do painel" min={0} max={100} step={1} value={Math.round(alphaOf(design.panel)*100/255)}
       fmt={v=>v+'%'} onChange={v=>set({panel:withAlpha(design.panel,Math.round(v*2.55))})}/>
     </div>

     <div className="lp-group">
      <h4>Forma</h4>
      <RangeRow label="Cantos arredondados" min={0} max={48} step={1} value={design.radius} fmt={v=>v+'px'} onChange={v=>set({radius:v})}/>
      <RangeRow label="Altura da barra" min={4} max={64} step={1} value={design.barHeight} fmt={v=>v+'px'} onChange={v=>set({barHeight:v})}/>
      <RangeRow label="Espessura da borda" min={0} max={6} step={0.5} value={design.borderWidth} fmt={v=>v+'px'} onChange={v=>set({borderWidth:v})}/>
     </div>

     <div className="lp-group">
      <h4>Tipografia</h4>
      <div className="lp-ctrl">
       <span className="lp-ctrl-label">Fonte</span>
       <select value={design.font} onChange={e=>set({font:e.target.value})}>
        {fontGroups.map(g=><optgroup key={g.label} label={g.label}>{g.fonts.map(f=><option key={f} value={f}>{f}</option>)}</optgroup>)}
       </select>
       <small className="lp-note">Google Fonts exigem internet no computador do OBS; sem internet a barra usa a fonte do sistema.</small>
      </div>
      <RangeRow label="Tamanho" min={10} max={48} step={1} value={design.fontSize} fmt={v=>v+'px'} onChange={v=>set({fontSize:v})}/>
      <RangeRow label="Espaçamento das letras" min={-2} max={10} step={0.5} value={design.tracking} fmt={v=>v+'px'} onChange={v=>set({tracking:v})}/>
      <div className="lp-ctrl">
       <span className="lp-ctrl-label">Peso</span>
       <select value={design.fontWeight} onChange={e=>set({fontWeight:Number(e.target.value)})}>
        {WEIGHTS.map(w=><option key={w} value={w}>{w}</option>)}
       </select>
      </div>
      <Toggle label="Caixa alta" checked={design.uppercase} onChange={v=>set({uppercase:v})}/>
     </div>

     <div className="lp-group">
      <h4>Efeitos</h4>
      <RangeRow label="Brilho (glow)" min={0} max={100} step={1} value={design.glow} fmt={v=>v+'%'} onChange={v=>set({glow:v})}/>
      <RangeRow label="Sombra" min={0} max={100} step={1} value={design.shadow} fmt={v=>v+'%'} onChange={v=>set({shadow:v})}/>
      <div className="lp-toggles">
       <Toggle label="Gradiente animado" checked={design.animate} onChange={v=>set({animate:v})}/>
       <Toggle label="Pulso ao receber doação" checked={design.pulse} onChange={v=>set({pulse:v})}/>
       <Toggle label="Celebração ao completar 100%" checked={design.celebrate} onChange={v=>set({celebrate:v})}/>
       <Toggle label="Explosão de emotes ao completar" checked={burstMeta} onChange={v=>void saveBurstMeta(v)}/>
      </div>
      <small className="lp-note">A explosão solta os seus emotes da Twitch na camada de meta quando a barra chega a 100%. A ligação geral, a quantidade e o tamanho ficam na tela Alertas.</small>
      <div className="lp-ctrl">
       <span className="lp-ctrl-label">Som de doação</span>
       <select value={design.soundDonate} aria-label="Som de doação" onChange={e=>set({soundDonate:e.target.value})}>
        <option value="">Sem som</option>
        {sounds.map(s=><option key={s.id} value={s.id}>{s.name}</option>)}
       </select>
       <small className="lp-note">Toca na saída escolhida na tela Áudio a cada doação que sobe a barra. Envie áudios na tela Sons de Live.</small>
      </div>
      <div className="lp-ctrl">
       <span className="lp-ctrl-label">Som de celebração</span>
       <select value={design.soundCelebrate} aria-label="Som de celebração" onChange={e=>set({soundCelebrate:e.target.value})}>
        <option value="">Sem som</option>
        {sounds.map(s=><option key={s.id} value={s.id}>{s.name}</option>)}
       </select>
       <small className="lp-note">Toca no lugar do som de doação quando a meta chega a 100%. Os botões Simular doação e Celebração ouvem a escolha.</small>
      </div>
     </div>
    </div>
   </div>
  </Card>

  <Card title="Metas">
   <p className="help">Cada meta tem título e valor-alvo em reais. A aparência — cores, fonte, bordas — fica no Estúdio acima. A meta ativa é consultada a cada 15 segundos. Excluir uma meta pede confirmação: o progresso dela não pode ser recuperado.</p>
   {config?.goals&&Object.entries(config.goals).map(([gid,g])=>
    <div key={gid} className="list-row">
     <div><strong>{g.title}</strong><small>{fmt(g.target)} BRL</small></div>
     <div className="row">
      {config.activeGoal===gid
       ?<span className="status-pill on"><span className="status-dot" aria-hidden="true"></span>Ativa</span>
       :<button disabled={busy||!config.enabled} onClick={()=>void call('livepix.set_active_goal',{goalId:gid})}>Ativar</button>}
      <button disabled={busy} onClick={()=>setEditing(editing===gid?'':gid)}><Pencil size={14}/>Editar</button>
      <button className="icon-button danger" aria-label={'Excluir meta '+g.title} disabled={busy} onClick={()=>{
       if(window.confirm(`Excluir a meta “${g.title}”? O progresso dela não pode ser recuperado.`))void call('livepix.remove_goal',{goalId:gid});
      }}><Trash2 size={15}/></button>
     </div>
     {editing===gid&&<EditGoal key={gid} initial={g} disabled={busy} onCancel={()=>setEditing('')} onSave={async(title,target)=>{
      await call('livepix.update_goal',{goalId:gid,title,target});setEditing('');
     }}/>}
    </div>
   )}
   {config?.enabled&&active&&<div className="row">
    <button disabled={busy} onClick={()=>{
     if(window.confirm('Reiniciar a contagem da meta ativa? O valor acumulado volta a zero.'))void call('livepix.reset_counter',{});
    }}><RefreshCw size={14}/>Reiniciar contador</button>
   </div>}
   <AddGoal onAdd={g=>{void call('livepix.add_goal',{title:g.title,target:g.target})}} disabled={busy}/>
  </Card>

  <Card title="No OBS" action={info?<span className="status-pill on"><span className="status-dot" aria-hidden="true"></span>Pronto</span>:undefined}>
   {info?<>
    <p className="help">Este é o <strong>endereço da camada da meta</strong> (o parâmetro <code>mode=goal</code> deixa só a meta nesta fonte). Cada camada — meta, alertas e chat — fica na sua própria Fonte de navegador, com posição e tamanho independentes. O endereço é estável: não muda ao reabrir o BotLive e não precisa de arquivo local.</p>
    <div className="lp-url">
     <code>{layerUrl()}</code>
     <button className="primary" onClick={()=>void copyUrl()}>{copied?<Check size={15}/>:<Copy size={15}/>}{copied?'Copiado':'Copiar endereço'}</button>
    </div>
    <ol className="lp-steps">
     <li>No OBS: <strong>Fontes → + → Navegador</strong>.</li>
     <li>Cole o endereço acima (o <code>#</code> traz a chave e o perfil).</li>
     <li>Defina <strong>Largura 1920</strong> e <strong>Altura 1080</strong> e posicione a fonte onde quiser.</li>
     <li>Deixe <strong>Atualizar navegador quando a cena ficar ativa</strong> desmarcado: o overlay reconecta sozinho.</li>
    </ol>
    <p className="help">A chave fica no trecho após o <code>#</code>, que navegadores nunca enviam a servidores externos. As alterações do Estúdio chegam ao OBS na hora. Fontes do Google Fonts exigem internet no computador que roda o OBS; fontes do sistema funcionam offline.</p>
   </>:<>
    <p className="help">{desktop?'Gerando o endereço do overlay…':'O endereço do overlay é gerado pelo aplicativo desktop. Abra o BotLive no computador da live para copiar o endereço da Fonte Navegador.'}</p>
    <p className="help">Enquanto isso, o arquivo <code>examples/goal-overlay.html</code> do repositório aceita o mesmo overlay com o fragmento <code>#token=…&profile={profileId}&port=9876</code>.</p>
   </>}
  </Card>
 </div>
}

/** Grupo de botões lado a lado (formato, alinhamento, fundo). */
function Seg<T extends string>({label,value,options,onChange}:{label:string;value:T;options:[T,string][];onChange:(v:T)=>void}){
 return <div className="lp-ctrl">
  <span className="lp-ctrl-label">{label}</span>
  <div className="lp-segrow" role="group" aria-label={label}>
   {options.map(([v,l])=><button key={v} type="button" className={'lp-seg'+(value===v?' selected':'')} aria-pressed={value===v} onClick={()=>onChange(v)}>{l}</button>)}
  </div>
 </div>;
}

function RangeRow({label,min,max,step,value,fmt,onChange}:{label:string;min:number;max:number;step:number;value:number;fmt:(v:number)=>string;onChange:(v:number)=>void}){
 return <label className="lp-range">
  <span className="lp-range-label">{label}<output>{fmt(value)}</output></span>
  <input type="range" min={min} max={max} step={step} value={value} onChange={e=>onChange(Number(e.target.value))}/>
 </label>;
}

function ColorRow({label,value,onChange}:{label:string;value:string;onChange:(v:string)=>void}){
 return <label className="lp-color"><span>{label}</span><input type="color" value={value} aria-label={label} onChange={e=>onChange(e.target.value)}/><code aria-hidden="true">{value.toUpperCase()}</code></label>;
}

function Toggle({label,checked,onChange}:{label:string;checked:boolean;onChange:(v:boolean)=>void}){
 return <label className="lp-toggle"><input type="checkbox" checked={checked} onChange={e=>onChange(e.target.checked)}/><span>{label}</span></label>;
}

/** Edição de meta existente: título e alvo, com o mesmo formato do AddGoal. */
function EditGoal({initial,disabled,onSave,onCancel}:{initial:GoalDef;disabled:boolean;onSave:(title:string,target:number)=>void;onCancel:()=>void}){
 const [title,setTitle]=useState(initial.title);
 const [target,setTarget]=useState(String(initial.target/100));
 const cents=parseBrl(target);
 const ok=title.trim()&&cents>0;
 return <div className="form-grid" style={{marginTop:8}}>
  <Field label="Título da meta"><input value={title} onChange={e=>setTitle(e.target.value)}/></Field>
  <Field label="Valor-alvo (R$)" hint="Aceita vírgula ou ponto: 500,00 ou 500."><input inputMode="decimal" value={target} onChange={e=>setTarget(e.target.value)}/></Field>
  <div className="row">
   <button className="primary" disabled={disabled||!ok} onClick={()=>onSave(title.trim(),cents)}>Salvar meta</button>
   <button disabled={disabled} onClick={onCancel}>Cancelar</button>
  </div>
 </div>;
}

/** "500,00", "1.500,00" e "500" → centavos. Com vírgula, ponto é milhar. */
function parseBrl(v:string):number{
 const s=String(v).trim();
 const normalized=s.includes(',')?s.replace(/\./g,'').replace(',','.'):s;
 const n=Number(normalized);
 return Number.isFinite(n)?Math.round(n*100):0;
}

function AddGoal({onAdd,disabled}:{onAdd:(g:{title:string;target:number})=>void;disabled:boolean}){
 const [title,setTitle]=useState('');
 const [target,setTarget]=useState('');
 const cents=parseBrl(target);
 const ok=title.trim()&&cents>0;
 return <details style={{marginTop:12}}><summary><Plus size={14} style={{verticalAlign:'middle'}}/> Nova meta</summary>
  <div className="form-grid" style={{marginTop:8}}>
   <Field label="Título"><input value={title} onChange={e=>setTitle(e.target.value)} placeholder="Ex.: Meta do mês"/></Field>
   <Field label="Valor-alvo (R$)" hint="Aceita vírgula ou ponto: 500,00 ou 500."><input inputMode="decimal" value={target} onChange={e=>setTarget(e.target.value)} placeholder="Ex.: 500"/></Field>
  </div>
  <button disabled={disabled||!ok} onClick={()=>{onAdd({title:title.trim(),target:cents});setTitle('');setTarget('')}}>Adicionar meta</button>
 </details>;
}
