param(
  [int]$Port = 17462,
  [string]$RepoRoot = ""
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
  $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
} else {
  $RepoRoot = $RepoRoot.Trim().Trim('"')
}
$RepoRoot = [IO.Path]::GetFullPath($RepoRoot).TrimEnd([char[]]"\/")

function Stop-ProcessTree([int]$ProcessId) {
  & taskkill.exe /PID $ProcessId /T /F *> $null
}

$staleNodeProcesses = Get-CimInstance Win32_Process -Filter "Name = 'node.exe'" -ErrorAction SilentlyContinue | Where-Object {
  $commandLine = [string]$_.CommandLine
  $commandLine -like "*$RepoRoot*" -and (
    $commandLine -match "@tauri-apps[\\/]cli[\\/]tauri\.js" -or
    $commandLine -match "node_modules[\\/]vite" -or
    $commandLine -match "(?:^|\s)vite(?:\s|$)"
  )
}

foreach ($process in $staleNodeProcesses) {
  Write-Host "Chiusura sessione _davSHARE precedente (PID $($process.ProcessId))..."
  Stop-ProcessTree $process.ProcessId
}

foreach ($processName in @('davshare','_davSHARE')) {
  Get-Process $processName -ErrorAction SilentlyContinue | ForEach-Object {
    try {
      if ($_.Path -and ([IO.Path]::GetFullPath($_.Path)).StartsWith($RepoRoot, [StringComparison]::OrdinalIgnoreCase)) {
        Write-Host "Chiusura vecchia istanza _davSHARE (PID $($_.Id))..."
        Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
      }
    } catch {}
  }
}

Start-Sleep -Milliseconds 650

$listeners = @(Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue)
if ($listeners.Count -gt 0) {
  $ownerIds = $listeners | Select-Object -ExpandProperty OwningProcess -Unique
  foreach ($ownerId in $ownerIds) {
    $owner = Get-CimInstance Win32_Process -Filter "ProcessId = $ownerId" -ErrorAction SilentlyContinue
    $name = if ($owner) { $owner.Name } else { "processo sconosciuto" }
    Write-Host "ERRORE: la porta $Port e ancora occupata da $name (PID $ownerId)." -ForegroundColor Red
    Write-Host "Chiudi quel processo oppure la relativa applicazione e riprova." -ForegroundColor Yellow
  }
  exit 20
}

exit 0
