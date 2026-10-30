/** Organização por tarefa; ids estáveis mantêm os destinos existentes. */
export const navigationSections = [
 {label:'SEU CANAL',pages:['dashboard','profiles']},
 {label:'CHAT',pages:['commands','timers','responses','counters']},
 {label:'VISUAL DO OBS',pages:['obs','labels','livepix','alertas','chatobs']},
 {label:'ÁUDIO E VOZ',pages:['sounds','audio','vozfx','spotify']},
 {label:'AUTOMAÇÃO E IA',pages:['flows','ai','memory','presets']},
 {label:'COMUNIDADE',pages:['community','discord']},
 {label:'ACOMPANHAMENTO',pages:['history','stats']},
];
const keywords:Record<string,string>={
 dashboard:'inicio configurar começar tarefas',
 profiles:'conta canal twitch youtube kick conectar autorizar login',
 commands:'comando responder mensagem chat followage',
 timers:'mensagem automatica periodica intervalo lembrete',
 responses:'resposta palavra chave texto txt',
 counters:'contador mortes vitorias placar',
 obs:'cena fonte gravacao conectar websocket filtro texto imagem navegador endereco url grupo espelhar girar midia',
 labels:'rotulo arquivo texto seguidores espectadores',
 livepix:'doacao pix meta arrecadacao overlay',
 alertas:'alerta follow sub bits raid overlay',
 chatobs:'chat tela overlay mensagens',
 sounds:'som importar biblioteca arquivo espectador',
 audio:'som saida fonte ouvir volume testar',
 vozfx:'voz voicemod resgate recompensa efeito microfone',
 spotify:'musica tocando spotify',
 flows:'automacao fluxo gatilho evento resgate recompensa condicao receita modelo pronto obs',
 ai:'ia inteligencia artificial personalidade modelo provedor',
 memory:'memoria conhecimento notas contexto',
 presets:'modelo pronto importar exportar pacote',
 community:'pontos loja sorteio fila jogos comunidade batimentos bpm amazfit relogio pulsoid bluetooth android celular',
 discord:'discord servidor cargos membros',
 history:'erro falha diagnostico historico logs eventos',
 stats:'estatistica numero relatorio atividade',
 settings:'configuracao atualizar backup aparencia tema acesso',
};
const normalize=(value:string)=>value.normalize('NFD').replace(/[̀-ͯ]/g,'').toLowerCase();
export function navigationMatches(id:string,label:string,query:string){
 const text=normalize(label+' '+(keywords[id]||''));
 return normalize(query).trim().split(/\s+/).every(word=>text.includes(word));
}
export const setupJourneys=[
 {id:'channel',title:'Conectar meu canal',description:'Perfil, contas e primeiro comando.',steps:[{page:'profiles',label:'1. Criar perfil e autorizar as contas'},{page:'commands',label:'2. Criar e testar um comando'},{page:'history',label:'3. Conferir o resultado no Histórico'}]},
 {id:'chat',title:'Configurar o chat',description:'Comandos, respostas e mensagens periódicas.',steps:[{page:'commands',label:'1. Criar uma resposta por comando'},{page:'responses',label:'2. Responder a palavras do chat'},{page:'timers',label:'3. Programar mensagens periódicas'}]},
 {id:'visual',title:'Montar o visual da live',description:'Alertas, chat na tela e meta.',steps:[{page:'obs',label:'1. Conferir a integração OBS (opcional)'},{page:'alertas',label:'2. Personalizar e testar alertas'},{page:'chatobs',label:'3. Preparar o chat na tela'},{page:'livepix',label:'4. Configurar a meta'}]},
 {id:'audio',title:'Preparar sons e voz',description:'Sons do bot e efeitos de voz em caminhos próprios.',steps:[{page:'sounds',label:'1. Organizar a biblioteca de sons'},{page:'audio',label:'2. Testar os sons no OBS'},{page:'vozfx',label:'3. Configurar voz e resgates no Voicemod'}]},
 {id:'community',title:'Envolver a comunidade',description:'Pontos, jogos e integração com Discord.',steps:[{page:'community',label:'1. Escolher os recursos da comunidade'},{page:'discord',label:'2. Conectar e configurar o Discord'}]},
 {id:'automation',title:'Criar algo personalizado',description:'Modelos prontos, automações e personalidade.',steps:[{page:'commands',label:'1. Abrir comandos e receitas prontas'},{page:'flows',label:'2. Criar uma automação por evento'},{page:'ai',label:'3. Ajustar a personalidade da IA'},{page:'memory',label:'4. Organizar memórias e conhecimento'}]},
];
