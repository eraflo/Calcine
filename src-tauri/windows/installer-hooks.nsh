; Lets terminals find calcine-cli. calcine-cli changes the user's PATH itself:
; NSIS strings stop at 1024 characters and would cut a longer PATH short.

!macro NSIS_HOOK_POSTINSTALL
  nsExec::Exec '"$INSTDIR\calcine-cli.exe" path add'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::Exec '"$INSTDIR\calcine-cli.exe" path remove'
!macroend
