/** Traduz erros técnicos vindos da API local, do provedor ou do sistema operacional
 * para a linguagem das telas. Mensagens que já estão em português são mantidas como estão
 * e o texto original só aparece em "Detalhes técnicos". */
export type FriendlyError={message:string;detail:string};

const TECHNICAL:[RegExp,string][]=[
 [/panicked|rust_panic|unwrap\(|unreachable/i,'Algo falhou dentro do aplicativo. Feche e abra de novo; se repetir, veja o Histórico.'],
 [/fetch|network|econnrefused|etimedout|enotfound|econnreset|socket|offline|timed? ?out/i,'Não consegui falar com o aplicativo agora. Tente de novo em instantes.'],
 [/invalid type|unknown variant|unknown field|missing field|expected .+line|deserializ|serde|parse error|unexpected token|invalid json/i,'O aplicativo não conseguiu ler os dados salvos. Recarregue a tela e tente de novo.'],
 [/permission denied|eacces|eperm|access is denied/i,'O aplicativo não teve permissão para concluir. Feche o que estiver usando o arquivo e tente de novo.'],
 [/enospc|no space left/i,'Faltou espaço em disco para concluir. Libere espaço e tente de novo.'],
 [/out of memory|heap limit|allocation failed/i,'O aplicativo ficou sem memória para concluir. Feche e abra de novo.'],
 [/enoent|file not found/i,'O arquivo indicado não foi encontrado. Escolha outro e tente de novo.']
];

const PORTUGUESE=/[áàâãéêíóôõúç]|\b(nao|voce|erro|invalido|informe|campo|valor|mensagem|salvar|salvo|fluxo|bloco|gatilho|conecte|adicione|preencha|arquivo|perfil|comando|timer|evento|permissao|intervalo|resposta|aplicativo|dados|sequencia|acao|acoes|contador|canal|servidor|conexao)\b/i;

const GENERIC={message:'Não foi possível concluir. Tente de novo; se continuar, veja o Histórico.',detail:''};

export function friendlyError(e:unknown):FriendlyError{
 const raw=e instanceof Error?e.message:String(e).trim();
 if(!raw||raw==='[object Object]'||raw==='undefined'||raw==='null')return{...GENERIC};
 if(PORTUGUESE.test(raw))return{message:raw,detail:''};
 const mapped=TECHNICAL.find(([pattern])=>pattern.test(raw));
 if(mapped)return{message:mapped[1],detail:raw};
 return{...GENERIC,detail:raw};
}

export function friendlyMessage(e:unknown):string{return friendlyError(e).message}
