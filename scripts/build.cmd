@echo off
rem Atalho para scripts\build.ps1 (aceita os mesmos parametros, ex.: build.cmd -DryRun)
setlocal
where pwsh >/dev/null 2>/dev/null && (set "PS=pwsh") || (set "PS=powershell")
%PS% -NoProfile -ExecutionPolicy Bypass -File "%~dp0build.ps1" %*
exit /b %ERRORLEVEL%
