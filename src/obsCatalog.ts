/** Operações compartilhadas pelo editor de automações e pelo teste de conexão. */
export const OBS_OPS:Record<string,string>={mute:'Mutar entrada',unmute:'Desmutar entrada',toggle_mute:'Alternar mudo',volume:'Volume da entrada',show:'Mostrar fonte',hide:'Esconder fonte',toggle_item:'Alternar fonte',scene:'Trocar de cena',media_play:'Reproduzir mídia',media_pause:'Pausar mídia',media_stop:'Parar mídia',media_restart:'Reiniciar mídia',media_next:'Próxima mídia',media_previous:'Mídia anterior',rotate:'Girar fonte',flip_horizontal:'Espelhar horizontalmente',flip_vertical:'Espelhar verticalmente'};
export const OBS_TEMP=['mute','unmute','volume','show','hide'];
export const obsNumeric=(op:string)=>op==='volume'||op==='rotate';
export const obsInput=(op:string)=>['mute','unmute','toggle_mute','volume'].includes(op)||op.startsWith('media_');
