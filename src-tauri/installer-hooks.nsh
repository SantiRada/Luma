!macro NSIS_HOOK_POSTINSTALL
  IfFileExists "$INSTDIR\uninstall.exe" 0 +2
    CopyFiles /SILENT "$INSTDIR\uninstall.exe" "$INSTDIR\Uninstall.exe"

  IfFileExists "$INSTDIR\LUMA.exe" 0 +3
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "LUMA" '"$INSTDIR\LUMA.exe"'
    Goto +2
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "LUMA" '"$INSTDIR\luma.exe"'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "LUMA"
  Delete "$INSTDIR\Uninstall.exe"
!macroend
