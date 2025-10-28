!macro NSIS_HOOK_PREUNINSTALL
  ; 在默认卸载逻辑之前，强制删除整个安装目录
  RMDir /r "$INSTDIR"
!macroend
