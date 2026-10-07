import {variableCatalog,type VariableChoice} from './variableCatalog';

/** A busca entende maiúsculas e acentos: "custo" acha "Custo", "inscricao" acha "inscrição". */
export function normalizeQuery(s:string){return s.toLowerCase().normalize('NFD').replace(/[\u0300-\u036f]/g,'').trim()}

export type PickerGroup={group:string;items:VariableChoice[]};

/** Filtra o catálogo pelo nome compreensível, pela categoria ou pelo identificador técnico. */
export function filterVariables(query:string,list:VariableChoice[]=variableCatalog):VariableChoice[]{
 const q=normalizeQuery(query);
 if(!q)return [...list];
 return list.filter(v=>normalizeQuery(v.label).includes(q)||normalizeQuery(v.group).includes(q)||normalizeQuery(v.example||'').includes(q)||v.key.toLowerCase().includes(q));
}

/** Mantém os resultados agrupados na mesma ordem das categorias do catálogo. */
export function groupVariables(list:VariableChoice[]):PickerGroup[]{
 const groups:PickerGroup[]=[];
 for(const v of list){
  const found=groups.find(g=>g.group===v.group);
  if(found)found.items.push(v);else groups.push({group:v.group,items:[v]});
 }
 return groups;
}

/**
 * Resultado do seletor: o valor salvo na etapa que não está mais no catálogo
 * (chave antiga ou personalizada) aparece no topo, para nunca sumir da configuração.
 */
export function searchVariables(query:string,value:string):{orphan:VariableChoice|null;groups:PickerGroup[]}{
 const list=filterVariables(query);
 const q=normalizeQuery(query);
 const inCatalog=list.some(v=>v.key===value);
 const orphan=value&&!inCatalog&&(!q||normalizeQuery(value).includes(q))?{key:value,label:value,group:'Valor salvo'}:null;
 return {orphan,groups:groupVariables(list)};
}
