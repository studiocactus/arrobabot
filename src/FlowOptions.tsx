import {useEffect,useState} from 'react';
import {open} from '@tauri-apps/plugin-dialog';
import {api,errorText,desktop} from './api';
import {Field} from './components';
import {playViewerSound} from './viewerSound';
import {sendTypes,sendColors} from './types';

type Asset={id:string;kind:string;name:string};
type Extras={config:{audioDeviceId:string};assets:Asset[]};

const HINT:Record<string,string>={
 chat:'Publica no chat como qualquer outra mensagem do bot.',
 announce:'A Twitch mostra um banner colorido no chat. O bot precisa ser moderador do canal; depois desta atualização autorize as contas novamente para liberar os anúncios.',
 pin:'Comunicado: a mensagem fica fixada no topo do chat por cerca de 20 minutos. Exige bot moderador e autorização nova.',
 shoutout:'A Twitch destaca outro canal no seu chat. Escreva na mensagem apenas o nome do canal de destino, como outrocanal ou {{user}}. Só funciona com o canal ao vivo e tem limite de destaques por hora.'
};

export function SendPicker({value,color,platform,onChange,onColor}:{value:string;color:string;platform:string;onChange:(v:string)=>void;onColor:(v:string)=>void}){
 const twitch=platform==='twitch';
 const type=sendTypes[value]||sendTypes.chat;
 return <>
 <div className="form-grid">
  <Field label="Como enviar na Twitch" hint={twitch?(HINT[value]||HINT.chat):'Somente a Twitch oferece anúncio, fixação e destaque. Nesta plataforma tudo é publicado como mensagem comum.'}>
   <select value={value} onChange={e=>onChange(e.target.value)} disabled={!twitch}>{Object.entries(sendTypes).map(([k,n])=><option key={k} value={k}>{n}</option>)}</select>
  </Field>
  {value==='announce'&&<Field label="Cor do anúncio"><select value={color} onChange={e=>onColor(e.target.value)} disabled={!twitch}>{Object.entries(sendColors).map(([k,n])=><option key={k} value={k}>{n}</option>)}</select></Field>}
 </div>
 {twitch&&value!=='chat'&&<p className="help">Envio escolhido: {type}. Confira no Histórico se a Twitch aceitou; quando falta permissão, o aplicativo diz o que falta corrigir.</p>}
 </>
}

export function AudioPicker({profileId,value,volume,onChange,onVolume}:{profileId:string;value:string;volume:number;onChange:(v:string)=>void;onVolume:(v:number)=>void}){
 const [assets,setAssets]=useState<Asset[]>([]);
 const [device,setDevice]=useState('');
 const [error,setError]=useState('');
 const [busy,setBusy]=useState(false);
 useEffect(()=>{let live=true;if(!desktop){setAssets([]);return}
  api<Extras>('chatExtras.get',{profileId}).then(v=>{if(live){setAssets(v.assets.filter(a=>a.kind==='sound'));setDevice(v.config.audioDeviceId||'')}}).catch(e=>{if(live)setError(errorText(e))});
  return()=>{live=false}},[profileId]);
 async function run(fn:()=>Promise<void>){setBusy(true);setError('');try{await fn()}catch(e){setError(errorText(e))}finally{setBusy(false)}}
 const choose=()=>run(async()=>{
  const path=await open({multiple:false,filters:[{name:'Áudio',extensions:['wav','mp3','ogg']}]});
  if(typeof path!=='string')return;
  const asset=await api<Asset>('chatExtras.import',{profileId,kind:'sound',path});
  setAssets(list=>list.concat(asset));onChange(asset.id);
 });
 const test=()=>run(async()=>{await playViewerSound({profileId,asset:value,volume,deviceId:device})});
 return <>
 <div className="form-grid">
  <Field label="Tocar áudio ao disparar" hint="Escolha um som já importado em Respostas e sons. Ele toca na saída de áudio do BotLive quando o comando, o timer ou a automação executar; capture essa saída no OBS para a live ouvir.">
   <select value={value} onChange={e=>onChange(e.target.value)}>
    <option value="">Nenhum áudio</option>
    {assets.map(a=><option key={a.id} value={a.id}>{a.name}</option>)}
    {value&&!assets.some(a=>a.id===value)&&<option value={value}>Áudio não encontrado nesta biblioteca</option>}
   </select>
  </Field>
  {value&&<Field label={'Volume: '+Math.round(volume*100)+'%'}><input type="range" min="0" max="1" step="0.05" value={volume} onChange={e=>onVolume(Number(e.target.value))}/></Field>}
 </div>
 <div className="row wrap">
  <button type="button" disabled={!desktop||busy} onClick={()=>void choose()}>Escolher som</button>
  <button type="button" disabled={!desktop||busy||!value} onClick={()=>void test()}>Testar som</button>
 </div>
 {!desktop&&<p className="notice">A prévia mostra a escolha. Importar e testar arquivos de áudio fica no aplicativo desktop.</p>}
 {error&&<p role="alert" className="inline-error">{error}</p>}
 </>
}
