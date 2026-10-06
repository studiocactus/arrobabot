import {useCallback,useEffect,useState} from 'react';
import {MonitorPlay,RefreshCw,Save} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {Card,Field} from './components';
type Cfg={enabled:boolean;host:string;port:number};
type Status={online:boolean;obsVersion:string;wsVersion:string;scene:string};
type Disco={scenes:string[];current:string;inputs:string[];sources:string[]};
export default function Obs({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [cfg,setCfg]=useState<Cfg>({enabled:false,host:'',port:4455});
 const [hasPassword,setHasPassword]=useState(false);
 const [password,setPassword]=useState('');
 const [st,setSt]=useState<Status|null>(null);
 const [disco,setDisco]=useState<Disco|null>(null);
 const [busy,setBusy]=useState(false);
 const [error,setError]=useState('');
 const [testOp,setTestOp]=useState('mute');
 const [testTarget,setTestTarget]=useState('');
 const [testOut,setTestOut]=useState('');
 const load=useCallback(async()=>{
  try{
   const v=await api<{config:Cfg;hasPassword:boolean}>('obs.get',{profileId});
   setCfg({enabled:v.config.enabled,host:v.config.host,port:v.config.port||4455});
   setHasPassword(v.hasPassword);
  }catch(e){setError(errorText(e))}
 },[profileId]);
 useEffect(()=>{if(desktop)void load()},[load]);
 async function run(fn:()=>Promise<void>){setBusy(true);setError('');try{await fn()}catch(e){setError(errorText(e))}finally{setBusy(false)}}
 async function save(){
  await run(async()=>{
   await api('obs.save',{profileId,config:cfg});
   if(password){await api('obs.password',{profileId,value:password});setPassword('');}
   await load();notify('OBS salvo.');
  });
 }
 async function check(){await run(async()=>{setSt(await api<Status>('obs.status',{profileId}));setDisco(await api<Disco>('obs.discover',{profileId}))})}
 async function test(){
  setTestOut('');
  await run(async()=>{
   const done=await api<string>('obs.execute',{profileId,op:testOp,target:testTarget,num:0,secs:0});
   setTestOut(String(done));notify('Ação executada no OBS.');
  });
 }
 let pill='status-pill off';
 let pillText='Parado';
 let statusText='Confira com o botão abaixo.';
 if(st){if(st.online){pill='status-pill on';pillText='Conectado';statusText='Conectado · OBS '+st.obsVersion+' · WS '+st.wsVersion+' · cena '+st.scene;}else{statusText='Desconectado';}}
 return <div>
 <Card title="OBS Studio local">
  <div className="list-row"><div><strong>Estado</strong><small>{statusText}</small></div><span className={pill}><span className="status-dot" aria-hidden="true"></span><span>{pillText}</span></span></div>
  <div className="row">
   <button disabled={!desktop||busy} onClick={()=>void check()}><RefreshCw size={15}/>Testar conexão</button>
  </div>
  {error&&<p role="alert" className="inline-error">{error}</p>}
  {disco&&<p className="help">Cenas: {disco.scenes.join(', ')||'—'} · Entradas: {disco.inputs.join(', ')||'—'} · Fontes da cena atual: {disco.sources.join(', ')||'—'}</p>}
 </Card>
 <Card title="Conexão">
  <Field label="Ativar integração OBS"><select value={cfg.enabled?'on':'off'} onChange={e=>setCfg({...cfg,enabled:e.target.value==='on'})}><option value="off">Desativada</option><option value="on">Ativada</option></select></Field>
  <div className="row end">
   <Field label="Endereço"><input value={cfg.host} placeholder="127.0.0.1" onChange={e=>setCfg({...cfg,host:e.target.value})}/></Field>
   <Field label="Porta"><input type="number" min={1} max={65535} value={cfg.port} onChange={e=>setCfg({...cfg,port:Number(e.target.value)||4455})}/></Field>
  </div>
  <Field label="Senha do WebSocket" hint={hasPassword?'Senha guardada no cofre. Preencha para trocar.':'Sem senha salva.'}><input type="password" autoComplete="new-password" value={password} placeholder="Só se o OBS pedir" onChange={e=>setPassword(e.target.value)}/></Field>
  <div className="row"><button className="primary" disabled={!desktop||busy} onClick={()=>void save()}><Save size={15}/>Salvar OBS</button></div>
  {!desktop&&<p className="help">Configuração disponível no aplicativo desktop.</p>}
 </Card>
 <Card title="Testar ação">
  <div className="row end">
   <Field label="Operação"><select value={testOp} onChange={e=>setTestOp(e.target.value)}><option value="mute">Mutar entrada</option><option value="unmute">Desmutar entrada</option><option value="toggle_mute">Alternar mudo</option><option value="show">Mostrar fonte</option><option value="hide">Esconder fonte</option><option value="toggle_item">Alternar fonte</option><option value="scene">Trocar de cena</option></select></Field>
   <Field label="Alvo"><input value={testTarget} placeholder="Microfone" onChange={e=>setTestTarget(e.target.value)}/></Field>
  </div>
  <div className="row"><button disabled={!desktop||busy||!testTarget.trim()} onClick={()=>void test()}><MonitorPlay size={15}/>Executar no OBS</button></div>
  {testOut&&<p className="help">{testOut}</p>}
  <p className="help">Executa direto, sem resgate. Com fio: o OBS desta máquina, nunca nuvem.</p>
 </Card>
 </div>;
}
