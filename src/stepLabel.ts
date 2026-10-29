import {actions,punishModes,voiceEffects,type Action} from './types';
import {variableCatalog} from './variableCatalog';
import {OBS_OPS} from './obsCatalog';

/** Nome legível de uma variável de condição, preservando o código quando não há entrada. */
function condVarLabel(k:string){const v=variableCatalog.find(v=>v.key===k);return v?v.label:k}
function condOpLabel(op:string){const m:Record<string,string>={equals:'=',not_equals:'≠',greater_or_equal:'≥',greater_than:'>',less_than:'≤',less_or_equal:'≤',contains:'contém',not_contains:'não contém',is_empty:'vazio',is_not_empty:'não vazio'};return m[op]||op}
function durationLabel(ms:number){const v=ms/1000;if(v>=60)return `${Math.round(v/60)} min`;return `${v.toFixed(1)} s`}
function twitchKindLabel(k:string){const m:Record<string,string>={game:'Jogo',title:'Titulo',timeout:'Timeout',ban:'Ban',unban:'Desban',warn:'Warn',vip:'Vip',unvip:'Unvip',shoutout:'Shoutout',mention:'Mencao',followers:'Followers'};return m[k]||k}
function obsOpLabel(op:string){const m:Record<string,string>={mute:'Mutar',unmute:'Desmutar',toggle_mute:'Alternar mudo',volume:'Volume',show:'Mostrar fonte',hide:'Esconder fonte',toggle_item:'Alternar fonte',scene:'Trocar cena'};return m[op]||OBS_OPS[op]||op}
/**
 * Resumo legível da etapa. É o mesmo texto do bloco no canvas: o acompanhamento do
 * teste reutiliza aqui para a sequência ser reconhecível por quem montou o fluxo.
 */
export function stepLabel(a:Action):string{const kind=a.kind;
 if(kind==='condition')return '◆ '+(condVarLabel(a.condVar||'')||'variável')+' '+condOpLabel(a.condOp||'equals')+(a.condValue?' '+a.condValue:'');
 if(kind==='wait')return '◷ '+durationLabel(a.value||0);
 if(kind==='obs')return '◉ '+obsOpLabel(a.obsOp||'mute')+(a.obsTarget?' · '+a.obsTarget:'');
 if(kind==='voiceFx'){const legacy=voiceEffects[a.target||''];const name=a.text&&!voiceEffects[a.text]?a.text:(legacy||a.text||a.target||'Voz');return '🎙 Voz: '+name+(a.value>0?' · '+a.value+'s':' · até cancelar')}
 if(kind==='action.ref')return '↗ '+(a.text||'Etapa reutilizável');
 if(kind==='followAge')return '⏱ Tempo de follow';
 if(kind==='twitch')return '⚡ '+twitchKindLabel(a.twOp||'game');
 if(kind==='punish')return '🔨 '+(punishModes[a.punish||'timeout']||a.punish||'Punir');
 if(kind==='redemption')return '⚡ Resgate';
 if(kind==='script')return '📜 Script';
 return actions[kind]||kind}
