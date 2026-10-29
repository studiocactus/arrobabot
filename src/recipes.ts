import {newAction,newFlow,newTimer,type Flow} from './types';
import {followAgeFlow,followersCountFlow,nowPlayingFlow,donationFlow,clippersFlow,FOLLOW_SAY_OK_DEFAULT,FOLLOW_SAY_WARN_DEFAULT} from './commandModels';

export const recipes=[
 {id:'obsText',title:'Comando atualiza texto no OBS',group:'OBS',when:'Um moderador usa !tela seguido do texto.',result:'Atualiza uma fonte de texto e confirma no chat depois que o OBS aceitar.',requirements:'Escolha uma fonte Texto no OBS; autorize o bot para responder no chat.'},
 {id:'obsImage',title:'Resgate mostra uma imagem',group:'OBS',when:'A recompensa da Twitch escolhida é resgatada.',result:'Mostra uma fonte existente por dez segundos, restaura a visibilidade anterior e agradece no chat.',requirements:'Crie a fonte Imagem no OBS e escolha a recompensa e a fonte. Bot e canal autorizados; OBS conectado.'},
 {id:'obsFilter',title:'Resgate ativa um filtro no OBS',group:'OBS',when:'A recompensa da Twitch escolhida é resgatada.',result:'Ativa um filtro já existente e confirma no chat. O filtro fica ligado até outra ação desligar.',requirements:'Escolha recompensa, fonte e filtro criado no OBS. OBS conectado e contas autorizadas.'},
 {id:'welcome',title:'Boas-vindas por comando',group:'Chat',when:'Alguém usa !oi.',result:'Responde no chat com o nome da pessoa.',requirements:'Perfil conectado e conta do bot autorizada.'},
 {id:'followAge',title:'Tempo de follow',group:'Chat',when:'Alguém usa !followers ou !followage.',result:'Consulta a data real de follow e responde com os dados da Twitch.',requirements:'Contas autorizadas e permissões de leitura da Twitch.'},
 {id:'followers',title:'Total de seguidores',group:'Chat',when:'Alguém usa !seguidores.',result:'Mostra o total de seguidores do canal.',requirements:'Conta do canal autorizada na Twitch.'},
 {id:'song',title:'Música atual',group:'Chat',when:'Alguém usa !musica.',result:'Responde com a música tocando no Spotify.',requirements:'Spotify configurado e conectado neste perfil.'},
 {id:'donation',title:'Link de doação',group:'Chat',when:'Alguém usa !pix.',result:'Gera uma frase com IA e inclui seu link de doação.',requirements:'Informe seu link e configure o provedor de IA.'},
 {id:'clippers',title:'Clipadores',group:'Chat',when:'Alguém usa !clip ou !clipe.',result:'Cria um clipe e atualiza a contagem de quem pediu.',requirements:'Canal ao vivo e permissão para criar clipes na Twitch.'},
 {id:'counter',title:'Contador de mortes',group:'Chat',when:'Um moderador usa !mortes.',result:'Soma um uso e informa a contagem no chat.',requirements:'Perfil conectado; por padrão, só moderadores e streamer.'},
 {id:'conversation',title:'Resenha com IA',group:'Automação',when:'Uma mensagem contém a expressão que você escolher.',result:'Gera uma resposta contextual e envia ao chat.',requirements:'Configure a IA e revise a expressão e os intervalos.'},
 {id:'timer',title:'Lembrete de água',group:'Automação',when:'A cada 15 minutos, enquanto o perfil está conectado.',result:'Envia um lembrete de hidratação.',requirements:'Salve, revise e ative o timer na lista de Timers.'},
 {id:'sound',title:'Som por resgate',group:'Áudio e voz',when:'A recompensa da Twitch escolhida é resgatada.',result:'Toca o som escolhido e agradece no chat.',requirements:'Escolha uma recompensa específica e um som; teste Áudio no OBS.'},
 {id:'raid',title:'Agradecer raid',group:'Automação',when:'Outro canal envia uma raid.',result:'Configura mensagem e destaque no painel existente Shoutout e ganks.',requirements:'Usa a configuração atual da Comunidade; não cria outra regra de raid.'},
 {id:'voice',title:'Voz por resgate',group:'Áudio e voz',when:'Uma recompensa aciona uma voz do Voicemod.',result:'Abre a Etapa 3 do Voicemod, com as regras e fila existentes.',requirements:'Conecte o Voicemod e autorize o canal; confira o áudio no OBS.'},
] as const;
export type RecipeId=typeof recipes[number]['id'];
export function recipeDraft(id:RecipeId,profileId:string):Flow|null{
 let flow:Flow;
 switch(id){
  case 'obsText':{const base=newFlow(profileId);flow={...base,name:'Texto no OBS pelo chat',trigger:{...base.trigger,pattern:'!tela',permission:'moderator'},actions:[{...newAction('condition'),condVar:'rawInput',condOp:'is_not_empty',condValue:'',condFalse:'stop'},{...newAction('obs'),obsOp:'set_text',text:'{{rawInput}}'},{...newAction(),text:'Texto atualizado no OBS.'}]};break;}
  case 'obsImage':flow={...newFlow(profileId),name:'Imagem por resgate',trigger:{kind:'redemption',pattern:'',permission:'everyone',cooldown:15,userCooldown:0},actions:[{...newAction('obs'),obsOp:'show',obsDuration:10},{...newAction(),text:'{{user}}, sua imagem apareceu na live!'}]};break;
  case 'obsFilter':flow={...newFlow(profileId),name:'Filtro por resgate',trigger:{kind:'redemption',pattern:'',permission:'everyone',cooldown:15,userCooldown:0},actions:[{...newAction('obs'),obsOp:'filter_enable'},{...newAction(),text:'{{user}}, filtro ativado no OBS.'}]};break;
  case 'followAge':flow=followAgeFlow(profileId,{command:'!followers',alias:'!followage',mention:true,ai:false,ok:'',sayOk:FOLLOW_SAY_OK_DEFAULT,sayWarn:FOLLOW_SAY_WARN_DEFAULT});break;
  case 'followers':flow=followersCountFlow(profileId,{command:'!seguidores',text:''});break;
  case 'song':flow=nowPlayingFlow(profileId,{command:'!musica',text:''});break;
  case 'donation':flow=donationFlow(profileId,{command:'!pix',link:''});break;
  case 'clippers':flow=clippersFlow(profileId,{command:'!clip, !clipe'});break;
  case 'welcome':flow={...newFlow(profileId),name:'Boas-vindas',actions:[{...newAction(),text:'Olá, {{user}}! Bem-vindo ao canal.'}]};break;
  case 'counter':{const base=newFlow(profileId);flow={...base,name:'Contador de mortes',counter:true,trigger:{...base.trigger,pattern:'!mortes',permission:'moderator'},actions:[{...newAction(),text:'Mortes: {{commandCount}}'}]};break}
  case 'timer':flow={...newTimer(profileId),name:'Pausa para água',timerSeconds:900,actions:[{...newAction(),text:'Hora de beber água! Cuide de você e faça uma pausa.'}]};break;
  case 'sound':flow={...newFlow(profileId),name:'Som por resgate',trigger:{kind:'redemption',pattern:'',permission:'everyone',cooldown:0,userCooldown:0},actions:[{...newAction(),text:'{{user}}, valeu pelo resgate!'}]};break;
  default:return null;
 }
 return {...flow,enabled:false,fnId:flow.fnId||'recipe:'+id,fnVersion:flow.fnVersion||1};
}
/** Identidade explícita ou mesmo comando; nunca reescreve passos por semelhança. */
export function recipeExisting(draft:Flow,flows:Flow[]):Flow|undefined{
 const own=flows.filter(flow=>flow.profileId===draft.profileId);
 if(draft.fnId){const found=own.find(flow=>flow.fnId===draft.fnId);if(found)return found}
 if(draft.trigger.kind==='command'){
  const wanted=draft.trigger.pattern.split(',').map(value=>value.trim().toLowerCase()).filter(Boolean);
  return own.find(flow=>flow.trigger.kind==='command'&&flow.trigger.pattern.split(',').some(value=>wanted.includes(value.trim().toLowerCase())));
 }
 return own.find(flow=>flow.name===draft.name&&flow.trigger.kind===draft.trigger.kind);
}

/** Resenha mantém o seu formulário de criação e o fluxo já salvo para edição. */
export function recipeConfiguration(id:RecipeId,profileId:string,flows:Flow[]):Flow|undefined{
 const draft=recipeDraft(id,profileId);
 if(draft)return recipeExisting(draft,flows);
 if(id==='conversation')return flows.find(flow=>flow.profileId===profileId&&flow.fnId==='recipe:conversation')||flows.find(flow=>flow.profileId===profileId&&flow.name==='Resenha do chat'&&flow.trigger.kind==='contains');
 return undefined;
}
