import {api} from './api';
export type ViewerSound={profileId:string;asset:string;nickname?:string;volume:number;deviceId?:string};
let queue=Promise.resolve();let pending=0;
export function playViewerSound(sound:ViewerSound):Promise<void>{
 if(pending>=5)return Promise.reject(Error('Fila de sons cheia. Aguarde o áudio atual.'));
 pending++;
 const next=queue.catch(()=>{}).then(async()=>{
  const src=await api<string>('chatExtras.audio',{profileId:sound.profileId,asset:sound.asset});
  await new Promise<void>((resolve,reject)=>{
   const audio=new Audio(src);audio.volume=Math.max(0,Math.min(1,sound.volume));
   let done=false;const finish=(err?:Error)=>{if(done)return;done=true;clearTimeout(timer);audio.pause();audio.removeAttribute('src');audio.load();err?reject(err):resolve()};
   const timer=setTimeout(()=>finish(),15000);audio.onended=()=>finish();audio.onerror=()=>finish(Error('Não foi possível reproduzir este áudio. Tente WAV ou MP3.'));
   void (async()=>{
    if(sound.deviceId){
     if(typeof audio.setSinkId!=='function')throw Error('Seleção de saída não disponível. Escolha Padrão do Windows e configure o Mixer de volume.');
     try{await audio.setSinkId(sound.deviceId)}catch{throw Error('Saída de áudio indisponível. Conecte o dispositivo ou escolha outra saída e salve.');}
    }
    if(!done)await audio.play();
   })().catch(e=>finish(e instanceof Error?e:Error('Áudio bloqueado. Clique em Testar som e confira a saída de áudio do Windows.')));
  });
 });
 queue=next.finally(()=>{pending--});queue.catch(()=>{});return next;
}
