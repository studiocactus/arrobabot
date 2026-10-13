/** Configuração da Central de áudio — espelhada em src-tauri/src/audio.rs.
 *
 * Duas saídas possíveis por origem de som: a **linha de áudio** (o aplicativo
 * toca no aparelho escolhido, que você aponta para um cabo virtual e captura
 * no OBS) e o **navegador** (a fonte do overlay toca dentro do painel do OBS).
 * Cada origem tem seus próprios interruptores.
 */
export type AudioConfig={
  peopleLine:boolean;      // sons de espectador na linha do aplicativo
  peopleBrowser:boolean;   // ... e no navegador (fonte do overlay)
  flowLine:boolean;        // sons de fluxo na linha do aplicativo
  flowBrowser:boolean;     // ... e no navegador
  alertLine:boolean;       // som dos alertas na linha do aplicativo
  alertBrowser:boolean;    // ... e no navegador (como sempre foi)
  device:string;           // aparelho de saída (setSinkId); vazio = padrão do Windows
};

/** Espelhada em audio.rs: sons na linha, alertas no navegador. */
export const DEFAULT_AUDIO:AudioConfig={
  peopleLine:true,peopleBrowser:false,flowLine:true,flowBrowser:false,
  alertLine:false,alertBrowser:true,device:'',
};

export function normalizeAudio(v:unknown):AudioConfig{
  const o=(v&&typeof v==='object'?v:{}) as Record<string,unknown>;
  type Flag=Exclude<keyof AudioConfig,'device'>;
  const b=(k:Flag)=>typeof o[k]==='boolean'?o[k]===true:DEFAULT_AUDIO[k];
  return {
    peopleLine:b('peopleLine'),peopleBrowser:b('peopleBrowser'),
    flowLine:b('flowLine'),flowBrowser:b('flowBrowser'),
    alertLine:b('alertLine'),alertBrowser:b('alertBrowser'),
    device:typeof o.device==='string'?o.device.slice(0,512):'',
  };
}

/** Bipe curto (WAV 8 kHz mono) para o botão "Testar linha" da Central. */
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
