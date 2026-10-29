/** Operações compartilhadas pelo editor de automações e pelo teste de conexão. */
export const OBS_OPS:Record<string,string>={mute:'Mutar entrada',unmute:'Desmutar entrada',toggle_mute:'Alternar mudo',volume:'Volume da entrada',show:'Mostrar fonte',hide:'Esconder fonte',toggle_item:'Alternar fonte',scene:'Trocar de cena',media_play:'Reproduzir mídia',media_pause:'Pausar mídia',media_stop:'Parar mídia',media_restart:'Reiniciar mídia',media_next:'Próxima mídia',media_previous:'Mídia anterior',rotate:'Girar fonte',flip_horizontal:'Espelhar horizontalmente',flip_vertical:'Espelhar verticalmente'};
export const OBS_TEMP=['mute','unmute','volume','show','hide'];
export const obsNumeric=(op:string)=>op==='volume'||op==='rotate';
export const obsInput=(op:string)=>['mute','unmute','toggle_mute','volume'].includes(op)||op.startsWith('media_');
Object.assign(OBS_OPS,{filter_enable:'Ativar filtro',filter_disable:'Desativar filtro',filter_toggle:'Alternar filtro',set_text:'Atualizar texto no OBS',set_image:'Trocar imagem',set_browser_url:'Atualizar endereço de navegador'});
export const obsItem=(op:string)=>['show','hide','toggle_item','rotate','flip_horizontal','flip_vertical'].includes(op);
export const obsContent=(op:string)=>['set_text','set_image','set_browser_url'].includes(op);
export const obsCompatible=(op:string,kind:string)=>op==='set_text'?kind.startsWith('text_gdiplus')||kind.startsWith('text_ft2_source'):op==='set_image'?kind==='image_source':op==='set_browser_url'?kind==='browser_source':true;
export function obsProblem(a:import('./types').Action):string|null{
 const op=a.obsOp||'mute';
 if(!a.obsTarget?.trim())return op==='scene'?'Escolha uma cena':obsInput(op)?'Escolha uma entrada de áudio':'Escolha uma fonte';
 if(op.startsWith('filter_')&&!a.obsFilter?.trim())return 'Escolha o filtro da fonte';
 if(obsNumeric(op)){const n=Number(a.text);if(!a.text.trim()||!Number.isFinite(n)||n<(op==='rotate'?-360:-100)||n>(op==='rotate'?360:30))return op==='rotate'?'Informe a rotação entre -360 e 360 graus':'Informe o volume entre -100 e 30 dB';}
 if(['set_image','set_browser_url'].includes(op)&&!a.text.trim())return op==='set_image'?'Escolha a imagem':'Informe o endereço do navegador';
 if(op==='set_browser_url'&&!a.text.includes('{{')){try{const u=new URL(a.text);if(!['http:','https:'].includes(u.protocol)||!u.hostname||u.username||u.password)return 'Use um endereço HTTP/HTTPS sem usuário ou senha';}catch{return 'Informe um endereço completo com http:// ou https://';}}
 return null;
}
