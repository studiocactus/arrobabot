import {useEffect,useState} from 'react';
import {Field} from './components';
import {errorText} from './api';
export default function AudioOutput({value,onChange}:{value:string;onChange:(id:string)=>void}){
 const [devices,setDevices]=useState<MediaDeviceInfo[]>([]);const [error,setError]=useState('');
 const supported=typeof HTMLMediaElement.prototype.setSinkId==='function';
 const media=navigator.mediaDevices as MediaDevices&{selectAudioOutput?:()=>Promise<MediaDeviceInfo>};
 async function choose(){try{const selected=await media.selectAudioOutput!();onChange(selected.deviceId);await refresh()}catch(e){setError(errorText(e))}}
 async function refresh(){try{setDevices((await navigator.mediaDevices.enumerateDevices()).filter(d=>d.kind==='audiooutput'&&d.deviceId&&d.deviceId!=='default'));setError('')}catch(e){setError(errorText(e))}}
 useEffect(()=>{void refresh();navigator.mediaDevices?.addEventListener('devicechange',refresh);return()=>navigator.mediaDevices?.removeEventListener('devicechange',refresh)},[]);
 return <><Field label="Dispositivo de saída dos sons" hint="Escolha a mesma saída que será capturada no OBS. A seleção vale para este perfil e não altera a voz de texto para fala."><select value={value||''} onChange={e=>onChange(e.target.value)} disabled={!supported}><option value="">Padrão do Windows</option>{value&&!devices.some(d=>d.deviceId===value)&&<option value={value}>Dispositivo salvo — confira se está conectado</option>}{devices.map((d,i)=><option key={d.deviceId} value={d.deviceId}>{d.label||'Saída de áudio '+(i+1)}</option>)}</select></Field><button onClick={()=>void refresh()}>Atualizar dispositivos</button>{supported&&media?.selectAudioOutput&&<button onClick={()=>void choose()}>Autorizar outra saída</button>}{(!supported||devices.length===0)&&<p className="notice">Se sua saída não aparecer, escolha a saída do BotLive no Mixer de volume do Windows e mantenha Padrão do Windows neste painel.</p>}{error&&<p role="alert">Não foi possível selecionar ou listar os dispositivos: {error}</p>}<p className="help">Se a saída escolhida for desconectada, novos sons serão recusados com um aviso. Escolha outra saída e salve. Dispositivos disponíveis dependem do Windows e das permissões do aplicativo.</p></>
}
