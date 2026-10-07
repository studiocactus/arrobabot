export type EditorPrefs={minimap?:boolean};
/** Preferências locais do editor no mesmo formato defensivo de botlive-preview: JSON em localStorage, silencioso quando ausente. */
export function parsePrefs(raw:string|null):EditorPrefs{
 try{const v:unknown=JSON.parse(raw||'{}');return v&&typeof v==='object'&&!Array.isArray(v)?v as EditorPrefs:{}}catch{return {}}
}
export function readPrefs():EditorPrefs{try{return parsePrefs(localStorage.getItem('botlive-editor'))}catch{return {}}}
export function writePrefs(patch:EditorPrefs):EditorPrefs{
 const next={...readPrefs(),...patch};
 try{localStorage.setItem('botlive-editor',JSON.stringify(next))}catch{}
 return next;
}
