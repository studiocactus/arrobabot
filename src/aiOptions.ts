import type {Action,AIConfig,KnowledgeEntry} from './types';

/** Perfis salvos antes desta versão não têm os campos novos; completa sem quebrar a tela. */
export function normalizeAI(ai:AIConfig):AIConfig {
  const src=(ai||{}) as AIConfig;
  const num=(v:unknown,d:number)=>typeof v==='number'&&isFinite(v)?v:d;
  const str=(v:unknown,d:string)=>typeof v==='string'?v:d;
  return {...src,
    provider:str(src.provider,'ollama') as AIConfig['provider'],
    endpoint:str(src.endpoint,'http://localhost:11434'),
    model:str(src.model,''),
    personality:str(src.personality,''),
    temperature:num(src.temperature,0.7),
    fallback:str(src.fallback,''),
    remember:!!src.remember,
    knowledge:src.knowledge!==false,
    knowledgeNicho:src.knowledgeNicho||'',
    knowledgeDepth:src.knowledgeDepth||'standard',
    knowledgeOff:Array.isArray(src.knowledgeOff)?src.knowledgeOff:[],
    knowledgeSource:src.knowledgeSource||'',
    anchor:src.anchor||'',
    answerLength:src.answerLength||'',
    noRepeat:!!src.noRepeat,
  };
}

/** O que vale de fato quando uma ação não escolhe nada: o padrão do perfil. */
export const resolveAnchor=(action:Action,ai:AIConfig)=>action.aiAnchor||ai.anchor||'all';
export const resolveLength=(action:Action,ai:AIConfig)=>action.aiLength||ai.answerLength||'';
export const resolveKnowledge=(action:Action,ai:AIConfig)=>action.aiKnowledge==='on'||(action.aiKnowledge!=='off'&&ai.knowledge);
export const resolveNoRepeat=(action:Action,ai:AIConfig)=>action.aiNoRepeat===null||action.aiNoRepeat===undefined?ai.noRepeat:action.aiNoRepeat;

export const isExcluded=(ai:AIConfig,path:string)=>(ai.knowledgeOff||[]).includes(path);
export const toggleKnowledgeFile=(ai:AIConfig,path:string):AIConfig=>{
  const off=ai.knowledgeOff||[];
  return {...ai,knowledgeOff:off.includes(path)?off.filter(p=>p!==path):off.concat(path)};
};

export type KnowledgeGroup={category:string;files:{entry:KnowledgeEntry;included:boolean}[]};
/** Lista da base agrupada por pasta, na ordem em que a IA lê: primeiro o que estiver ligado. */
export function knowledgeGroups(items:KnowledgeEntry[],ai:AIConfig):KnowledgeGroup[] {
  const groups=new Map<string,{entry:KnowledgeEntry;included:boolean}[]>();
  for(const entry of [...items].sort((a,b)=>a.path.localeCompare(b.path))) {
    const key=entry.category||'geral';
    if(!groups.has(key))groups.set(key,[]);
    groups.get(key)!.push({entry,included:!isExcluded(ai,entry.path)});
  }
  return [...groups].map(([category,files])=>({category,files}));
}
