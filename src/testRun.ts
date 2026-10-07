/** Estados de uma execução de teste, em português, e regras puras do acompanhamento.
 * Só existe o que o backend devolve: sem duração, percentual ou resultado calculado aqui. */
export type TestStep={index:number;kind:string;status:string};
export type TestRun={id:string;flow:string;status:string;test:boolean;steps:TestStep[];error?:string|null};

export const RUN_STATUS:Record<string,string>={RUNNING:'Executando',WAITING:'Em espera',COMPLETED:'Concluída',FAILED:'Falhou',CANCELLED:'Cancelada',STOPPED_BY_CONDITION:'Parada por condição'};
export const STEP_STATUS:Record<string,string>={PENDING:'Aguardando execução',RUNNING:'Executando',WAITING:'Em espera',SUCCESS:'Concluída',SKIPPED:'Pulada',FAILED:'Falhou',CANCELLED:'Cancelada',TRUE:'Condição verdadeira',FALSE:'Condição falsa'};
export const STEP_ICON:Record<string,string>={SUCCESS:'✓',FAILED:'✕',CANCELLED:'■',SKIPPED:'–',RUNNING:'▶',WAITING:'⏳',PENDING:'○',TRUE:'✓',FALSE:'○'};

/** Só estados finais encerram o acompanhamento; parada por condição é conclusão, não falha. */
export const TERMINAL_RUNS=['COMPLETED','FAILED','CANCELLED','STOPPED_BY_CONDITION'] as const;
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
export const TEST_SAMPLES:Record<string,string>={'viewer.name':'TesteViewer',user:'Teste',message:'!teste','reward.title':'Flashbang',rewardTitle:'Flashbang','reward.cost':'5000',rewardCost:'5000','redemption.input':'hello',userInput:'hello',redeemer:'TesteViewer','bits.amount':'500','raid.viewers':'42','raider.name':'TesteRaider','subscription.tier':'1000','gifter.name':'TesteGifter'};
export function initialSamples(keys:string[]):Record<string,string>{
 return Object.fromEntries(keys.map(k=>[k,TEST_SAMPLES[k]||'']));
}
