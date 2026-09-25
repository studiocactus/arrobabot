; =============================================================================
; Hooks NSIS do BotLive
;
; Fonte: referenciado por bundle.windows.nsi.installerHooks em
;        src-tauri/tauri.conf.json. A Tauri insere estes macros no script
;        gerado nos pontos NSIS_HOOK_* (pre/post de instalacao e desinstalacao).
;
; ATENCAO: os hooks rodam DEPOIS da pagina de reinstalacao, entao eles nao
; conseguem impedir a desinstalacao. Isso esta resolvido no template proprio
; (src-tauri/installer.nsi). Aqui fica apenas a limpeza pos-instalacao.
; =============================================================================

; Depois que todos os arquivos e atalhos foram gravados: regenera o cache de
; icones do Explorer. Ao sobrescrever o executavel o Windows mantem o cache
; antigo e pode exibir o icone em branco ate o cache ser invalidado.
!macro NSIS_HOOK_POSTINSTALL
  ${If} ${FileExists} "$SYSDIR\ie4uinit.exe"
    ExecWait '"$SYSDIR\ie4uinit.exe" -show'
  ${EndIf}
!macroend
