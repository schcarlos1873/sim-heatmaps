<#
.SYNOPSIS
  Compila e empacota o Sim Heatmaps para Windows (MSI e/ou instalador NSIS).

.DESCRIPTION
  Etapas: pré-requisitos -> dependências npm -> testes -> tauri build -> cópia dos
  instaladores para release\<versão>\windows-<arch>\ com SHA256SUMS.txt.
  Compatível com Windows PowerShell 5.1 e PowerShell 7+.

.PARAMETER DryRun
  Mostra os comandos sem executá-los.
.PARAMETER SkipTests
  Não roda cargo test / typecheck.
.PARAMETER SkipInstall
  Não roda npm ci.
.PARAMETER Clean
  Remove builds anteriores (cargo clean, dist\) antes de compilar.
.PARAMETER DebugBuild
  Build de depuração (mais rápido, sem otimizações).
.PARAMETER Target
  Alvo Rust (ex.: x86_64-pc-windows-msvc, aarch64-pc-windows-msvc).
.PARAMETER Bundles
  Formatos separados por vírgula: msi,nsis (padrão: ambos).
.PARAMETER Out
  Pasta de saída (padrão: release\).

.EXAMPLE
  .\scripts\build.ps1 -DryRun
.EXAMPLE
  .\scripts\build.ps1 -Bundles nsis
.EXAMPLE
  .\scripts\build.ps1 -Target aarch64-pc-windows-msvc

.NOTES
  Assinatura de código (opcional): configure bundle.windows.certificateThumbprint no
  tauri.conf.json ou um signCommand. Atualizador: TAURI_SIGNING_PRIVATE_KEY.
#>
[CmdletBinding()]
param(
  [switch]$DryRun,
  [switch]$SkipTests,
  [switch]$SkipInstall,
  [switch]$Clean,
  [switch]$DebugBuild,
  [string]$Target = "",
  [string]$Bundles = "",
  [string]$Out = ""
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version 3

$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root
if (-not $Out) { $Out = Join-Path $Root "release" }

# ------------------------------------------------------------- cores ANSI
# Windows 10+ suporta sequências ANSI no console; NO_COLOR desativa.
$UseColor = (-not $env:NO_COLOR) -and (-not [Console]::IsOutputRedirected)
$E = [char]27
function C([string]$code, [string]$text) { if ($UseColor) { "$E[$($code)m$text$E[0m" } else { $text } }

function Step([string]$msg) { Write-Host ""; Write-Host ((C "34" "==>") + " " + (C "1" $msg)) }
function Ok([string]$msg)   { Write-Host ("  " + (C "32" "v") + " $msg") }
function Warn([string]$msg) { Write-Host ("  " + (C "33" "!") + " $msg") }
function Die([string]$msg)  { Write-Host ((C "31" "erro:") + " $msg"); exit 1 }

# Executa (ou apenas mostra, em -DryRun) um comando externo e falha se o código de saída != 0.
function Run([string]$exe, [string[]]$arguments = @()) {
  Write-Host ("  " + (C "2" ("$ " + $exe + " " + ($arguments -join " "))))
  if ($DryRun) { return }
  & $exe @arguments
  if ($LASTEXITCODE -ne 0) { Die "'$exe' terminou com código $LASTEXITCODE" }
}

function Has([string]$cmd) { [bool](Get-Command $cmd -ErrorAction SilentlyContinue) }

# ------------------------------------------------------------- validação
$ValidBundles = @("msi", "nsis")
$BundleList = @()
if ($Bundles) {
  $BundleList = $Bundles.Split(",") | ForEach-Object { $_.Trim().ToLower() } | Where-Object { $_ }
  foreach ($b in $BundleList) {
    if ($ValidBundles -notcontains $b) { Die "formato '$b' inválido para Windows (use: msi,nsis)" }
  }
}

if ($Target) {
  $Arch = $Target.Split("-")[0]
} else {
  $Arch = switch ($env:PROCESSOR_ARCHITECTURE) { "ARM64" { "aarch64" } "x86" { "i686" } default { "x86_64" } }
}

if ($DryRun) { Warn "modo -DryRun: nenhum comando será executado" }

# ------------------------------------------------------------- pré-requisitos
Step "Verificando pré-requisitos (windows/$Arch)"
foreach ($cmd in @("node", "npm", "cargo", "rustc")) {
  if (-not (Has $cmd)) { Die "'$cmd' não encontrado no PATH" }
  $ver = (& $cmd --version 2>$null | Select-Object -First 1)
  Ok "$cmd $ver"
}

$nodeMajor = [int](node -p "process.versions.node.split('.')[0]")
if ($nodeMajor -lt 20) { Die "Node 20+ é necessário (encontrado $(node --version))" }

$rustHost = (rustc -vV | Select-String "^host:").ToString().Split(" ")[1]
if ($rustHost -notmatch "msvc") {
  Die "o Tauri exige o toolchain MSVC do Rust (host atual: $rustHost). Rode: rustup default stable-msvc"
}
Ok "toolchain Rust $rustHost"

# Build Tools do Visual Studio (link.exe) — detectado via vswhere quando disponível.
$vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vswhere) {
  $vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property displayName
  if ($vs) { Ok "Visual Studio: $vs" } else { Warn "Build Tools C++ (MSVC) não encontrados: instale 'Desktop development with C++'" }
} else {
  Warn "vswhere não encontrado; verifique se o Visual Studio Build Tools (C++) está instalado"
}

# WebView2 vem no Windows 10/11; o instalador do app também pode baixá-lo.
$wv2 = "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
if (Test-Path $wv2) { Ok "WebView2 Runtime" } else { Warn "WebView2 Runtime não detectado (necessário para executar o app)" }

if ($Target) {
  $installed = rustup target list --installed
  if ($installed -contains $Target) { Ok "alvo Rust $Target" }
  else { Warn "alvo Rust $Target não instalado"; Run "rustup" @("target", "add", $Target) }
}

$conf = Get-Content (Join-Path $Root "src-tauri\tauri.conf.json") -Raw -Encoding UTF8 | ConvertFrom-Json
$Version = $conf.version
Ok "$($conf.productName) $Version"

# ------------------------------------------------------------- limpeza
if ($Clean) {
  Step "Limpando builds anteriores"
  Write-Host ("  " + (C "2" "$ Remove-Item dist -Recurse -Force"))
  if (-not $DryRun -and (Test-Path "dist")) { Remove-Item "dist" -Recurse -Force }
  Run "cargo" @("clean", "--manifest-path", "src-tauri\Cargo.toml")
}

# ------------------------------------------------------------- dependências
if (-not $SkipInstall) {
  Step "Instalando dependências npm"
  if (Test-Path "package-lock.json") { Run "npm" @("ci") } else { Run "npm" @("install") }
}

# ------------------------------------------------------------- testes
if (-not $SkipTests) {
  Step "Rodando testes"
  Run "cargo" @("test", "--workspace", "--manifest-path", "src-tauri\Cargo.toml")
  Run "npm" @("run", "typecheck")
}

# ------------------------------------------------------------- build
Step "Compilando e empacotando"
$buildArgs = @("tauri", "build")
if ($DebugBuild) { $buildArgs += "--debug" }
if ($Target) { $buildArgs += @("--target", $Target) }
if ($BundleList.Count -gt 0) { $buildArgs += @("--bundles", ($BundleList -join ",")) }
Run "npx" $buildArgs

# ------------------------------------------------------------- distribuição
$profileName = if ($DebugBuild) { "debug" } else { "release" }
$bundleDir = if ($Target) { "src-tauri\target\$Target\$profileName\bundle" } else { "src-tauri\target\$profileName\bundle" }
$dest = Join-Path $Out (Join-Path $Version "windows-$Arch")

Step "Copiando instaladores para $dest"
if ($DryRun) {
  Write-Host ("  " + (C "2" "(dry-run) copiaria os arquivos de $bundleDir e geraria SHA256SUMS.txt"))
  exit 0
}
if (-not (Test-Path $bundleDir)) { Die "pasta de bundles não encontrada: $bundleDir" }

if (Test-Path $dest) { Remove-Item $dest -Recurse -Force }
New-Item -ItemType Directory -Path $dest -Force | Out-Null

$files = @()
$files += Get-ChildItem (Join-Path $bundleDir "msi\*.msi") -ErrorAction SilentlyContinue
$files += Get-ChildItem (Join-Path $bundleDir "nsis\*.exe") -ErrorAction SilentlyContinue
if ($files.Count -eq 0) { Die "nenhum instalador encontrado em $bundleDir" }

foreach ($f in $files) {
  Copy-Item $f.FullName $dest
  Ok $f.Name
}

# SHA256SUMS.txt no mesmo formato do sha256sum (hash minúsculo + dois espaços + nome), UTF-8 sem BOM.
$lines = Get-ChildItem $dest -File | ForEach-Object {
  "{0}  {1}" -f (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower(), $_.Name
}
[System.IO.File]::WriteAllLines((Join-Path $dest "SHA256SUMS.txt"), [string[]]$lines, (New-Object System.Text.UTF8Encoding $false))
Ok "SHA256SUMS.txt"

Write-Host ""
Write-Host ((C "32" "concluído:") + " $($files.Count) arquivo(s) em $dest")
