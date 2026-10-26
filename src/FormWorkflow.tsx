/** Contrato visível de configuração: salvar não comprova resultado na live. */
export default function FormWorkflow({save,test,requirements}:{save:string;test:string;requirements:string}){
 return <div className="form-workflow" role="note" aria-label="Salvar e testar esta configuração">
  <p><strong>Salvar:</strong> {save}</p>
  <p><strong>Testar:</strong> {test}</p>
  <p><strong>Para funcionar na live:</strong> {requirements}</p>
 </div>;
}
