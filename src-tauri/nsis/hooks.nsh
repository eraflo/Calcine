; Calcine NSIS installer hooks (merged in by tauri.release.conf.json).
;
; The official GenieX installer is bundled as a resource and run silently when
; GenieX isn't installed yet. It is per-user (no admin), installs to
; %LOCALAPPDATA%\GenieX CLI and adds itself to the user PATH. Upgrades of an
; existing GenieX are handled by the app, which first stops the server: the
; GenieX installer kills any running geniex.exe.

!macro NSIS_HOOK_POSTINSTALL
  IfFileExists "$LOCALAPPDATA\GenieX CLI\geniex.exe" calcine_geniex_present
    DetailPrint "Installing GenieX CLI..."
    ExecWait '"$INSTDIR\geniex\geniex-cli-setup.exe" /VERYSILENT /SUPPRESSMSGBOXES /NORESTART' $0
    DetailPrint "GenieX CLI installer exited with code $0"
    Goto calcine_geniex_done
  calcine_geniex_present:
    DetailPrint "GenieX CLI already installed, leaving it as is"
  calcine_geniex_done:
!macroend
