/** Configuração de áudio do BotLive — espelhada em src-tauri/src/audio.rs.
 *
 * Todos os sons do bot (espectador, fluxo, resgates e alertas) saem pela
 * **linha de áudio**: o aparelho de saída escolhido aqui, que você captura
 * no OBS como uma única fonte. Não existe mais rota pelo navegador — a fonte
 * do overlay ficou só visual.
 */
export type AudioConfig={
  device:string;  // aparelho de saída (setSinkId); vazio = padrão do Windows
};

/** Espelhada em audio.rs: saída padrão do Windows até você escolher. */
export const DEFAULT_AUDIO:AudioConfig={device:''};

export function normalizeAudio(v:unknown):AudioConfig{
  const o=(v&&typeof v==='object'?v:{}) as Record<string,unknown>;
  return {device:typeof o.device==='string'?o.device.slice(0,512):''};
}

/** Bipe curto (WAV 8 kHz mono) para o botão "Testar saída" da tela Áudio. */
export function beepDataUri(hz=440,seconds=0.35):string{
  const rate=8000;const n=Math.round(rate*seconds);
  const bytes=new Uint8Array(44+n*2);const view=new DataView(bytes.buffer);
  const str=(off:number,s:string)=>{for(let i=0;i<s.length;i++)bytes[off+i]=s.charCodeAt(i)};
  str(0,'RIFF');view.setUint32(4,36+n*2,true);str(8,'WAVE');str(12,'fmt ');
  view.setUint32(16,16,true);view.setUint16(20,1,true);view.setUint16(22,1,true);
  view.setUint32(24,rate,true);view.setUint32(28,rate*2,true);view.setUint16(32,2,true);view.setUint16(34,16,true);
  str(36,'data');view.setUint32(40,n*2,true);
  for(let i=0;i<n;i++){
    const fade=Math.min(1,(n-i)/(rate*0.08));
    view.setInt16(44+i*2,Math.round(Math.sin(2*Math.PI*hz*(i/rate))*9000*fade),true);
  }
  let bin='';for(const b of bytes)bin+=String.fromCharCode(b);
  return 'data:audio/wav;base64,'+btoa(bin);
}
