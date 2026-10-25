import {newAction,newFlow,type Action,type Flow} from './types';

/** Shoutout/gank: o painel de Comunidade monta uma automação de raid de
 *  verdade (trigger "raid") — mesma execução, mesma fila, editável depois em
 *  Automações. Nada de motor paralelo. */

export type GankFields={
 shoutout:boolean;   // destaque nativo para quem mandou a raid
 ai:boolean;         // frase da IA (aleatória) no lugar do texto fixo
 announce:boolean;   // sai como Anúncio da Twitch
 text:string;        // mensagem fixa (quando sem IA)
 prompt:string;      // instrução da IA (quando com IA)
};

export const GANK_TEXT_DEFAULT='Valeu a raid, {{raider.name}}! {{raid.viewers}} pessoas chegaram junto — bem-vindos!';
export const GANK_PROMPT_DEFAULT='Você é o ArrobaSrv, um bot brasileiro bem-humorado. Escreva uma frase curta (máximo 160 caracteres) agradecendo a raid de {{raider.name}}, que trouxe {{raid.viewers}} pessoas. Cite o nome e a quantidade exatamente como vieram. Sempre diferente da anterior, sem gírias de internet.';
/** Reserva determinística: se a IA falhar, a raid ainda é agradecida. */
export const GANK_FALLBACK_DEFAULT='Valeu a raid, {{raider.name}}! {{raid.viewers}} pessoas chegaram junto — bem-vindos!';

/** Monta a automação de gank a partir das escolhas do painel. */
export function gankFlow(profileId:string,f:GankFields,base?:Flow):Flow{
 const flow=base??newFlow(profileId);
 const say:Action=f.ai
  ?{...newAction('ai.generate'),text:f.prompt.trim()||GANK_PROMPT_DEFAULT,target:'local.aiResponse',aiFallback:f.text.trim()||GANK_FALLBACK_DEFAULT}
  :{...newAction('chat'),text:f.text.trim()||GANK_TEXT_DEFAULT};
 const steps:Action[]=[];
 if(f.shoutout)steps.push({...newAction('twitch'),twOp:'shoutout'});
 steps.push(say);
 if(f.ai)steps.push({...newAction('chat'),text:'{{local.aiResponse}}'});
 return {
  ...flow,
  name:'Gank/Raid',
  fnId:'gank',
  fnVersion:1,
  sendType:f.announce?'announce':'chat',
  trigger:{kind:'raid',pattern:'',permission:'everyone',cooldown:0,userCooldown:0},
  actions:steps,
 };
}

/** Lê um fluxo existente de volta para o painel (o que estiver fora do
 *  formulário fica preservado no fluxo; o painel nunca descarta nada). */
export function readGank(flow:Flow):GankFields{
 const a=flow.actions;
 const ai=a.some(x=>x.kind==='ai.generate');
 return {
  shoutout:a.some(x=>x.kind==='twitch'&&x.twOp==='shoutout'),
  ai,
  announce:flow.sendType==='announce',
  text:(ai?a.find(x=>x.kind==='ai.generate')?.aiFallback:a.find(x=>x.kind==='chat')?.text)||'',
  prompt:a.find(x=>x.kind==='ai.generate')?.text||GANK_PROMPT_DEFAULT,
 };
}

/** O formulário simples não pode sobrescrever passos ou condições extras do editor visual. */
export function isGankCustomized(flow:Flow):boolean{
 const expected=gankFlow(flow.profileId,readGank(flow),flow);
 const relevant=(a:Action)=>({kind:a.kind,twOp:a.twOp||'',text:a.text||'',target:a.target||'',aiFallback:a.aiFallback||'',condition:a.condition||'',enabled:a.enabled!==false});
 return flow.sendType!==expected.sendType||JSON.stringify(flow.actions.map(relevant))!==JSON.stringify(expected.actions.map(relevant));
}
