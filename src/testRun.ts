/** Estados de uma execução de teste, em português, e regras puras do acompanhamento.
 * Só existe o que o backend devolve: sem duração, percentual ou resultado calculado aqui. */
type TestStep={index:number;kind:string;status:string;durationMs?:number|null};
export type TestRun={id:string;flow:string;status:string;test:boolean;steps:TestStep[];error?:string|null;startedAt?:number;durationMs?:number;cancelRequested?:boolean;interruptionNote?:string;context?:{eventKind:string;eventRef:string|null;mode:string}|null};

const RUN_STATUS:Record<string,string>={RUNNING:'Executando',WAITING:'Em espera',COMPLETED:'Concluída',FAILED:'Falhou',CANCELLED:'Cancelada',INTERRUPTED:'Interrompida',STOPPED_BY_CONDITION:'Parada por condição'};
const STEP_STATUS:Record<string,string>={PENDING:'Aguardando execução',INTERRUPTED:'Resultado não confirmado',NOT_EXECUTED:'Não iniciada',RUNNING:'Executando',WAITING:'Em espera',SUCCESS:'Concluída',SKIPPED:'Pulada',FAILED:'Falhou',CANCELLED:'Cancelada',TRUE:'Condição verdadeira',FALSE:'Condição falsa'};
const STEP_ICON:Record<string,string>={SUCCESS:'✓',FAILED:'✕',CANCELLED:'■',SKIPPED:'–',RUNNING:'▶',WAITING:'⏳',PENDING:'○',TRUE:'✓',FALSE:'○'};

/** Só estados finais encerram o acompanhamento; parada por condição é conclusão, não falha. */
export const TERMINAL_RUNS=['COMPLETED','FAILED','CANCELLED','STOPPED_BY_CONDITION','INTERRUPTED'] as const;
export const isTerminalRun=(status:string)=>TERMINAL_RUNS.includes(status as typeof TERMINAL_RUNS[number]);
export const runStatusLabel=(status:string)=>RUN_STATUS[status]||status;
export const stepStatusLabel=(status:string)=>STEP_STATUS[status]||status;
export const stepIcon=(status:string)=>STEP_ICON[status]||'?';

/** Condição falsa é resultado da regra, nunca erro técnico: explica parar ou pular. */
export function conditionNote(status:string,condFalse?:string):string{
 if(status==='TRUE')return 'Condição verdadeira: a sequência continuou.';
 if(status!=='FALSE')return '';
 return condFalse==='skip'?'Condição falsa: a próxima etapa foi pulada. Não é uma falha.':'Condição falsa: o fluxo parou aqui. Não é uma falha.';
}

/** Valores de exemplo por variável, usados só quando o fluxo cita a variável. */
const TEST_SAMPLES:Record<string,string>={'viewer.name':'TesteViewer',user:'Teste',message:'!teste','reward.title':'Flashbang',rewardTitle:'Flashbang','reward.cost':'5000',rewardCost:'5000','redemption.input':'hello',userInput:'hello',redeemer:'TesteViewer','bits.amount':'500','raid.viewers':'42','raider.name':'TesteRaider','subscription.tier':'1000','gifter.name':'TesteGifter'};
export function initialSamples(keys:string[]):Record<string,string>{
 return Object.fromEntries(keys.map(k=>[k,TEST_SAMPLES[k]||'']));
}

export function runDuration(ms?:number|null){return ms==null?"Não registrado":ms<1000?`${Math.round(ms)} ms`:`${(ms/1000).toFixed(1)} s`}

export function runModeLabel(run:TestRun){
 const labels:Record<string,string>={live:'Execução da live',test:'Teste',rehearsal:'Ensaio',simulation:'Simulação'};
 return run.context?labels[run.context.mode]||'Modo não reconhecido':run.test?'Teste':'Execução da live';
}
export function eventKindLabel(kind:string){
 const labels:Record<string,string>={chat:'Mensagem no chat',timer:'Timer',follow:'Novo seguidor',subscription:'Inscrição',resub:'Reinscrição',gift:'Inscrição de presente',cheer:'Bits / Super Chat',raid:'Raid',redemption:'Resgate de pontos',custom:'Evento externo',voice:'Comando de voz',manual:'Início manual',live:'Início da live',unlive:'Fim da live',other:'Outro evento'};
 return labels[kind]||'Outro evento';
}
