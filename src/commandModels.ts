import {newAction,newFlow,type Action,type Flow} from './types';

/** Comandos prontos: um clique cria a automação completa, para a pessoa só
 *  conferir no editor e usar. O primeiro é o de doação (!pix) — resposta de IA
 *  diferente a cada chamada, com o link de doação fixo no fim de toda resposta. */

export type DonationFields={command:string;link:string};

/** Instrução da resposta: curta, animada e nunca igual à anterior. O pedido
 *  proíbe inventar valores e regras — o link certo vem da mensagem fixa abaixo.
 *  A proibição de links é importante: cada resposta do bot entra na memória da
 *  conversa e a IA imita as respostas anteriores — um link escrito por ela se
 *  repete e cresce a cada disparo. */
export const DONATION_PROMPT='Escreva uma frase curta (no máximo 100 caracteres), animada e DIFERENTE a cada resposta, agradecendo e convidando {{user}} a apoiar a live. Não invente valores, prêmios, metas ou regras. Não escreva links nem endereços: o link da doação já entra no fim da mensagem.';

/** O comando de doação: gera a frase variada e responde com o link da pessoa. */
export function donationFlow(profileId:string,{command,link}:DonationFields):Flow{
 const pattern=command.trim();
 const primary=pattern.split(',')[0].trim()||'!pix';
 const ia:Action={...newAction('ai.generate'),text:DONATION_PROMPT,target:'local.aiResponse'};
 const msg:Action={...newAction('chat'),text:`{{local.aiResponse}} 💜 ${link.trim()}`};
 const flow=newFlow(profileId);
 return {...flow,name:`Doação (${primary})`,trigger:{...flow.trigger,kind:'command',pattern,permission:'everyone'},actions:[ia,msg]};
}

/** O que falta no comando: nome vazio não tem como disparar. */
export function donationCommandIssue(command:string):string{
 return command.trim()?'':'Escreva o comando que dispara, como !pix.';
}

/** O que falta no link: sem ele a resposta sai sem o destino da doação. */
export function donationLinkIssue(link:string):string{
 const v=link.trim();
 if(!v)return 'Escreva o link de doação, como livepix.gg/seucanal.';
 if(/\s/.test(v))return 'O link não pode ter espaços.';
 if(!v.includes('.'))return 'Escreva um endereço como livepix.gg/seucanal.';
 return '';
}

export type ClippersFields={command:string};

/** O comando dos clipadores: cria o clipe na hora, soma na contagem de quem
 *  pediu e anuncia o número ("seu 3º clipe!"). A contagem vive em
 *  `{{user.clips}}` — por pessoa, persistente, pronta para ranking e mensagens.
 *  Só conta clipe criado com sucesso: se o clipe falhar, o fluxo para antes. */
export function clippersFlow(profileId:string,{command}:ClippersFields):Flow{
 const pattern=command.trim();
 const primary=pattern.split(',')[0].trim()||'!clip';
 const clip:Action={...newAction('twitch'),twOp:'clip'};
 const count:Action={...newAction('variable.increment'),target:'user.clips',text:'1'};
 const msg:Action={...newAction('chat'),text:'{{user}}, esse é o seu {{user.clips}}º clipe da live! {{local.clipUrl}}'};
 const flow=newFlow(profileId);
 return {...flow,name:`Clipadores (${primary})`,trigger:{...flow.trigger,kind:'command',pattern,permission:'everyone'},actions:[clip,count,msg]};
}

/** O que falta no comando: nome vazio não tem como disparar. */
export function clippersCommandIssue(command:string):string{
 return command.trim()?'':'Escreva o comando que dispara, como !clip.';
}
