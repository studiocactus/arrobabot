import type {Flow} from './types';
/** Só apresenta o formulário simples quando ele representa a sequência inteira. */
export function obsRecipeMatches(f:Flow):boolean{
 if(f.layout)return false; // Um desenho personalizado continua no editor que representa seus blocos.
 if(f.audio||f.counter||(f.sendType&&f.sendType!=='chat')||f.actions.some(a=>a.condition||a.enabled===false))return false;
 const kinds=f.actions.map(a=>a.kind).join(',');
 if(f.fnId==='recipe:obsText')return f.trigger.kind==='command'&&kinds==='condition,obs,chat'&&f.actions[0].condVar==='rawInput'&&f.actions[0].condOp==='is_not_empty'&&f.actions[0].condFalse==='stop'&&f.actions[1].obsOp==='set_text';
 if(f.fnId==='recipe:obsImage')return f.trigger.kind==='redemption'&&kinds==='obs,chat'&&f.actions[0].obsOp==='show';
 if(f.fnId==='recipe:obsFilter')return f.trigger.kind==='redemption'&&kinds==='obs,chat'&&['filter_enable','filter_disable','filter_toggle'].includes(f.actions[0].obsOp||'')&&!(f.actions[0].obsDuration||0);
 return false;
}
