/// Comandos naturais no chat (0.3): streamer e moderadores escrevem por
/// extenso e o bot executa o mesmo que `!setgame`/`!settitle`. Parser
/// determinístico e offline, falha fechada: só entra no que o vocabulário
/// cobre; qualquer dúvida vira Ignore e a mensagem segue o chat normal.
/// Exige a palavra de ativação do Controle por voz, igual à fala.
#[derive(Debug,Clone,PartialEq)]
pub enum Talk{
 /// Não parece comando: segue o processamento normal do chat.
 Ignore,
 /// Intenção clara sem o valor: responde a dica de uso.
 Incomplete(&'static str),
 /// Categoria com alternativas na ordem, como "Minecraft ou AION2".
 Category(Vec<String>),
 /// Título novo, texto único.
 Title(String),
}
/// Verbos de troca: sem nenhum deles a frase não é comando.
const VERBS:&[&str]=&["troca","troque","trocar","trocando","muda","mude","mudar","mudando","altera","altere","alterar","alterando","coloca","coloque","colocar","bota","bote","botar","ponha","poe","define","defina","definir","seta","setar","mexe","mexer","ajusta","ajuste","ajustar","atualiza","atualize","atualizar","passa","passe","passar"];
/// O que identifica cada intenção; as duas no mesmo pedido não executam.
const CATEGORY:&[&str]=&["categoria","jogo","jogos","game","games"];
const TITLE:&[&str]=&["titulo","title"];
/// O valor vem depois da primeira destas palavras após a intenção.
const SEPARATORS:&[&str]=&["para","pra","pro","por"];
/// Valor de pergunta, não de nome: silêncio, sem resposta no chat.
const QUESTIONS:&[&str]=&["que","qual","quais","quem","onde","quando","como","ai","aqui","ali","isso","esse","essa","aquilo","qualquer"];
/// Palavras de ligação que sobram nas bordas da categoria.
const EDGE_START:&[&str]=&["o","a","os","as","um","uma","do","da","de","dos","das","e","jogo","jogos","categoria"];
const EDGE_END:&[&str]=&["por","favor","agora","ai","de","do","da"];
const HINT_CATEGORY:&str="Para trocar a categoria, escreva o nome: mude a categoria para Minecraft. Dá para usar \"X ou Y\" e o bot tenta na ordem.";
const HINT_TITLE:&str="Para trocar o título, escreva o novo texto: mude o título para Ao vivo com a comunidade.";
/// Minúsculas e sem acento, um caractere por caractere: os índices valem no
/// texto original, então o valor sai da mensagem como foi escrita.
fn fold_char(c:char)->char{
 match c{
  'á'|'à'|'ã'|'â'|'ä'=>'a','é'|'è'|'ê'|'ë'=>'e','í'|'ì'|'î'|'ï'=>'i',
  'ó'|'ò'|'õ'|'ô'|'ö'=>'o','ú'|'ù'|'û'|'ü'=>'u','ç'=>'c',
  'Á'|'À'|'Ã'|'Â'|'Ä'=>'a','É'|'È'|'Ê'|'Ë'=>'e','Í'|'Ì'|'Î'|'Ï'=>'i',
  'Ó'|'Ò'|'Õ'|'Ô'|'Ö'=>'o','Ú'|'Ù'|'Û'|'Ü'=>'u','Ç'=>'c',
  other=>other.to_ascii_lowercase(),
 }
}
fn folded(s:&str)->String{s.chars().map(fold_char).collect()}
fn is_word(c:char)->bool{c.is_alphanumeric()||c=='_'||c=='-'}
/// Extensões [início,fim) das palavras em caracteres, para fatiar o original.
fn words(f:&[char])->Vec<(usize,usize)>{
 let mut out=Vec::new();let mut i=0;
 while i<f.len(){
  if is_word(f[i]){let start=i;while i<f.len()&&is_word(f[i]){i+=1}out.push((start,i));}
  else{i+=1}
 }
 out
}
fn at(chars:&[char],span:(usize,usize))->String{chars[span.0..span.1].iter().collect()}
fn has(list:&'static[&str],word:&str)->bool{list.contains(&word)}
fn clean_edges(s:&str)->String{
 s.trim().trim_matches(|c:char|matches!(c,'.'|','|'!'|'?'|';'|':'|'…'|'“'|'”'|'"'|'\'')).trim().to_owned()
}
/// Divide o valor pelo "ou" entre palavras: alternativas na ordem e a
/// primeira que existir na Twitch vale. Sem "ou" devolve o valor inteiro
/// como está; com mais de cinco nomes não parece alternativa e volta inteiro.
pub(crate) fn split_ou(value:&str)->Vec<String>{
 let chars:Vec<char>=value.chars().collect();
 let f:Vec<char>=folded(value).chars().collect();
 let mut pieces:Vec<String>=Vec::new();
 let mut last=0;let mut saw=false;let mut i=0;
 while i<chars.len(){
  if is_word(f[i]){
   let start=i;while i<chars.len()&&is_word(f[i]){i+=1}
   if f[start..i].iter().collect::<String>()=="ou"{
    saw=true;
    pieces.push(clean_edges(&chars[last..start].iter().collect::<String>()));
    last=i;
   }
  }else{i+=1}
 }
 if !saw{return vec![value.to_owned()]}
 pieces.push(clean_edges(&chars[last..].iter().collect::<String>()));
 pieces.retain(|p|!p.is_empty());
 if pieces.is_empty()||pieces.len()>5{return vec![clean_edges(value)]}
 pieces
}
/// Bordas da categoria: "para o jogo Minecraft" perde "o jogo" e fica
/// "Minecraft". A caixa original de cada palavra é mantida para a busca.
fn edge_trim(value:&str)->String{
 let chars:Vec<char>=value.chars().collect();
 let spans=words(&folded(value).chars().collect::<Vec<char>>());
 let word=|span|folded(&at(&chars,span));
 let mut start=0;let mut end=spans.len();
 while start<end&&has(EDGE_START,&word(spans[start])){start+=1}
 while end>start&&has(EDGE_END,&word(spans[end-1])){end-=1}
 spans[start..end].iter().map(|&sp|at(&chars,sp)).collect::<Vec<_>>().join(" ")
}
fn is_question(name:&str)->bool{
 let chars:Vec<char>=name.chars().collect();
 let spans=words(&folded(name).chars().collect::<Vec<char>>());
 spans.len()==1&&has(QUESTIONS,&folded(&at(&chars,spans[0])))
}
/// Lê uma mensagem de chat como comando natural. A ordem é sempre a mesma:
/// ativação, verbo, intenção, valor; o que não passa em uma etapa cai em
/// Ignore e a mensagem segue o fluxo normal do chat.
pub fn parse(message:&str,activation:&str)->Talk{
 // Sem palavra cadastrada não há comando natural no chat.
 let stops:Vec<String>=activation.split(',').map(str::trim).filter(|s|!s.is_empty()).map(|s|folded(s)).collect();
 if stops.is_empty(){return Talk::Ignore}
 let orig:Vec<char>=message.chars().collect();
 let f:Vec<char>=orig.iter().map(|&c|fold_char(c)).collect();
 let mentioned=stops.iter().any(|stop|{
  let needle:Vec<char>=stop.chars().collect();
  !needle.is_empty()&&f.windows(needle.len()).any(|w|w==needle.as_slice())
 });
 if !mentioned{return Talk::Ignore}
 let spans=words(&f);
 let tokens:Vec<String>=spans.iter().map(|&sp|at(&f,sp)).collect();
 if !tokens.iter().any(|w|has(VERBS,w)){return Talk::Ignore}
 let Some(kw_idx)=tokens.iter().position(|w|has(CATEGORY,w)||has(TITLE,w)) else{return Talk::Ignore};
 let in_category=tokens.iter().any(|w|has(CATEGORY,w));
 let in_title=tokens.iter().any(|w|has(TITLE,w));
 if in_category==in_title{return Talk::Ignore}// nem um, ou os dois no mesmo pedido
 let is_title=in_title;
 let Some(sep)=spans.iter().enumerate().find(|(i,_)|*i>kw_idx&&has(SEPARATORS,&tokens[*i])).map(|(_,sp)|*sp)
 else{return Talk::Incomplete(if is_title{HINT_TITLE}else{HINT_CATEGORY})};
 let value:String=orig[sep.1..].iter().collect();
 let vf:Vec<char>=folded(&value).chars().collect();
 if words(&vf).is_empty(){return Talk::Incomplete(if is_title{HINT_TITLE}else{HINT_CATEGORY})}
 if is_title{
  if is_question(value.trim()){return Talk::Ignore}
  return Talk::Title(value.trim().to_owned())
 }
 let names:Vec<String>=split_ou(&value).into_iter().map(|piece|edge_trim(&piece)).filter(|n|!n.is_empty()).collect();
 if names.is_empty(){return Talk::Incomplete(HINT_CATEGORY)}
 if is_question(&names[0]){return Talk::Ignore}
 if names.iter().any(|n|n.chars().count()>120){return Talk::Ignore}
 Talk::Category(names)
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn o_exemplo_completo_vira_categoria_com_plano_b(){
  assert_eq!(parse("Arroba, altere a categoria do jogo para Minecraft ou AION2","arroba"),Talk::Category(vec!["Minecraft".into(),"AION2".into()]));
 }
 #[test] fn intenções_comuns_são_lidas_com_e_sem_acento(){
  assert_eq!(parse("arroba, mude o jogo pra valorant","arroba"),Talk::Category(vec!["valorant".into()]));
  assert_eq!(parse("ÁRROBA! muda o titulo para Ranked com viewers","arroba"),Talk::Title("Ranked com viewers".into()));
  assert_eq!(parse("arroba, troque a categoria por Counter-Strike 2","arroba"),Talk::Category(vec!["Counter-Strike 2".into()]));
 }
 #[test] fn sem_ativacao_a_mensagem_e_chat_normal(){
  assert_eq!(parse("mude a categoria para Minecraft","arroba"),Talk::Ignore);
  assert_eq!(parse("arroba, mude a categoria para Minecraft",""),Talk::Ignore);
  assert_eq!(parse("arroba, mude a categoria para Minecraft","botlive"),Talk::Ignore);
 }
 #[test] fn sem_verbo_ou_com_as_duas_intencoes_nao_executa(){
  assert_eq!(parse("arroba, Minecraft é o melhor jogo","arroba"),Talk::Ignore);
  assert_eq!(parse("arroba, mude o jogo e o título para X","arroba"),Talk::Ignore);
  assert_eq!(parse("arroba, você muda o jogo?","arroba"),Talk::Incomplete(HINT_CATEGORY));
 }
 #[test] fn intencao_clara_sem_nome_responde_o_modo_de_usar(){
  assert_eq!(parse("arroba, mude a categoria","arroba"),Talk::Incomplete(HINT_CATEGORY));
  assert_eq!(parse("arroba, mude o título para","arroba"),Talk::Incomplete(HINT_TITLE));
  assert_eq!(parse("arroba, mude o jogo para ...","arroba"),Talk::Incomplete(HINT_CATEGORY));
 }
 #[test] fn pergunta_de_valor_fica_em_silencio(){
  assert_eq!(parse("arroba, o jogo muda para quê?","arroba"),Talk::Ignore);
  assert_eq!(parse("arroba, mude o título para quê?","arroba"),Talk::Ignore);
 }
 #[test] fn o_titulo_guarda_pontuacao_e_o_titulo_com_ou_nao_vira_alternativa(){
  assert_eq!(parse("arroba, mude o título para Ao vivo às 21h!","arroba"),Talk::Title("Ao vivo às 21h!".into()));
  assert_eq!(parse("arroba, mude o título para RPG ou horror","arroba"),Talk::Title("RPG ou horror".into()));
 }
 #[test] fn a_categoria_tira_palavras_de_ligacao_e_pontuacao_das_bordas(){
  assert_eq!(parse("arroba, por favor mude a categoria para o jogo Minecraft!","arroba"),Talk::Category(vec!["Minecraft".into()]));
  assert_eq!(parse("arroba, altere a categoria do jogo para MINECRAFT","arroba"),Talk::Category(vec!["MINECRAFT".into()]));
 }
 #[test] fn alternativas_da_twitch_comecam_pelo_primeiro_nome(){
  assert_eq!(split_ou("Minecraft ou AION2"),vec!["Minecraft".to_owned(),"AION2".to_owned()]);
  assert_eq!(split_ou("Minecraft OU AION2"),vec!["Minecraft".to_owned(),"AION2".to_owned()]);
  assert_eq!(split_ou("Counter-Strike 2"),vec!["Counter-Strike 2".to_owned()]);
  assert_eq!(split_ou("RPG ou"),vec!["RPG".to_owned()]);
  assert_eq!(split_ou("a ou b ou c ou d ou e ou f"),vec!["a ou b ou c ou d ou e ou f".to_owned()]);
 }
}