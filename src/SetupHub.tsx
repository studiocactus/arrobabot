import {useState} from 'react';
import {ArrowRight} from 'lucide-react';
import {Card,Tag,Jump} from './components';
import {setupJourneys} from './navigation';

export default function SetupHub({hasProfile,onGo,onCreateProfile}:{hasProfile:boolean;onGo:(page:string)=>void;onCreateProfile:()=>void}){
 const [selected,setSelected]=useState('');
 const journey=setupJourneys.find(item=>item.id===selected);
 return <Card title="O que você quer configurar?" className="setup-hub" action={<Jump onClick={()=>hasProfile?onGo('flows'):onCreateProfile()}>Criar uma automação</Jump>}>
  <Tag>CONFIGURE NO SEU RITMO</Tag>
  <p className="help">Escolha um objetivo para ver o caminho. Você pode voltar às mesmas telas pelo menu lateral.</p>
  <div className="setup-journeys">
   {setupJourneys.map(item=><button type="button" key={item.id} aria-pressed={selected===item.id} aria-controls="setup-journey-steps" onClick={()=>setSelected(item.id)}>
    <strong>{item.title}</strong><span>{item.description}</span>
   </button>)}
  </div>
  <div id="setup-journey-steps">
   {journey&&<section className="setup-journey-detail" aria-label={journey.title}>
    <h3>{journey.title}</h3>
    <p className="help">Abra a etapa que precisa ajustar. Este roteiro não indica que a configuração já foi testada.</p>
    {!hasProfile&&<div className="notice"><p className="help">Crie um perfil para configurar os recursos do seu canal.</p><button type="button" className="primary" onClick={onCreateProfile}>Criar perfil para começar</button></div>}
    <ol>{journey.steps.map(step=><li key={step.page}><button type="button" disabled={!hasProfile&&step.page!=='profiles'} onClick={()=>onGo(step.page)}><span>{step.label}</span><ArrowRight size={16}/></button></li>)}</ol>
   </section>}
  </div>
 </Card>;
}
