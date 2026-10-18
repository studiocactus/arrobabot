import {ArrowLeft} from 'lucide-react';
import {Card} from './components';
import VoiceCommands from './VoiceCommands';

/**
 * “Comandos por voz”: área própria do reconhecimento da sua fala, acessível a
 * partir de Comandos. Fica separada das trocas de voz do Voicemod — usar
 * resgates e comandos de voz não exige escuta contínua, servidor de
 * transcrição, palavra de ativação nem moderação.
 */
export default function VoiceCommandsPage({profileId,notify,onBack}:{profileId:string;notify:(s:string)=>void;onBack:()=>void}){
 return <div>
  <Card title="Comandos por voz">
   <p className="help">Aqui mora o reconhecimento da sua fala: escuta contínua, gravação de comando e criação rápida. Quando a escuta está ligada, o microfone é o <strong>físico</strong> (antes de qualquer efeito de voz). Nada disso é necessário para as trocas de voz do Voicemod funcionarem.</p>
   <VoiceCommands profileId={profileId} notify={notify}/>
  </Card>
  <div className="form-footer">
   <span className="help">Fala e áudio ficam nesta máquina: nenhum áudio sai do computador.</span>
   <button className="secondary" onClick={onBack}><ArrowLeft size={14}/> Voltar para Comandos</button>
  </div>
 </div>;
}
