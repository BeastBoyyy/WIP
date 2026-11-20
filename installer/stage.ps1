# Builds target\stage: rice-studio.exe plus clean copies of the bundled tools, ready for rice-studio.iss.
# Clean means: no personal data from this PC (Flow history and settings, Windhawk's profile id, symbol caches,
# logs) and no Windhawk compiler (the mods ship prebuilt, which saves 543 MB).
# Usage: powershell -File installer\stage.ps1 [-Apps D:\VagabondRice\apps]
param([string]$Apps = 'D:\VagabondRice\apps')
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot
$out = Join-Path $root 'target\stage'
if (Test-Path $out) { Remove-Item $out -Recurse -Force }
New-Item -ItemType Directory "$out\apps" | Out-Null

function Copy-Tree($from, $to, [string[]]$skipDirs = @(), [string[]]$skipFiles = @()) {
  $a = @($from, $to, '/E', '/NFL', '/NDL', '/NJH', '/NJS', '/NP')
  if ($skipDirs) { $a += '/XD'; $a += $skipDirs }
  if ($skipFiles) { $a += '/XF'; $a += $skipFiles }
  robocopy @a | Out-Null
  if ($LASTEXITCODE -ge 8) { throw "copy failed: $from" }
}

Copy-Item "$root\target\release\rice-studio.exe" $out
Copy-Item "$PSScriptRoot\NOTICES.txt" $out

Copy-Tree "$Apps\zebar" "$out\apps\zebar"
Copy-Tree "$Apps\leopardwm" "$out\apps\leopardwm"

# Flow: the program only. An empty UserData folder keeps it portable (settings stay in this folder, and
# never mix with a Flow the user may already have). packages\ only feeds Flow's own updater.
$flowApp = (Get-ChildItem "$Apps\flow" -Directory -Filter 'app-*' | Sort-Object Name | Select-Object -Last 1).Name
Copy-Tree "$Apps\flow" "$out\apps\flow" @("$Apps\flow\packages", "$Apps\flow\$flowApp\UserData")
New-Item -ItemType Directory "$out\apps\flow\$flowApp\UserData" | Out-Null

# Windhawk: engine, UI and the three prebuilt mods. Each mod keeps its [Mod] section; its [Settings] are
# left out so the mod starts at its defaults until a theme writes them.
$wh = "$Apps\windhawk"
Copy-Tree $wh "$out\apps\windhawk" @("$wh\Compiler", "$wh\AppData")
$ad = "$out\apps\windhawk\AppData"
Copy-Tree "$wh\AppData\Engine\Mods" "$ad\Engine\Mods" @() @('*.ini')
Copy-Tree "$wh\AppData\ModsSource" "$ad\ModsSource"
foreach ($ini in Get-ChildItem "$wh\AppData\Engine\Mods" -Filter *.ini) {
  $keep = @(); $on = $false
  foreach ($l in Get-Content $ini.FullName) {
    if ($l -match '^\[(.+)\]') { $on = $Matches[1] -eq 'Mod' }
    if ($on -and $l -notmatch '^SettingsChangeTime=') { $keep += $l }
  }
  Set-Content "$ad\Engine\Mods\$($ini.Name)" $keep -Encoding Unicode
}
# anti-cheat safety: Windhawk never injects into games or their launchers
Set-Content "$ad\Engine\settings.ini" -Encoding Unicode @'
[Settings]
LoggingVerbosity=0
Include=
Exclude=*\Riot Games\*|*\Riot Vanguard\*|*\League of Legends.exe|*\LeagueClient*.exe|*\RiotClient*.exe|*\Riot Client.exe|*\vgc.exe|*\vgtray.exe|*\VALORANT*.exe|*\EasyAntiCheat*|*\BattlEye\*|*\BEService*.exe|*\FACEIT*|*\steamapps\common\*|*\Epic Games\*
ThreadAttachExempt=
InjectIntoCriticalProcesses=0
InjectIntoIncompatiblePrograms=0
InjectIntoGames=0
'@
Set-Content "$ad\settings.ini" -Encoding Unicode @'
[Settings]
Language=en
LoggingVerbosity=0
HideTrayIcon=0
AlwaysCompileModsLocally=0
DontAutoShowToolkit=1
SafeMode=0
'@

# nothing personal may leave this PC
$leaks = Get-ChildItem $out -Recurse -File | Where-Object { $_.Name -match '^(History|UserSelectedRecord|userprofile)\.json|\.log$' -or $_.FullName -match '\\(Logs|UIData|EditorWorkspace|ModsWritable)\\' }
if ($leaks) { throw "personal files in the stage: $($leaks.FullName -join ', ')" }
$size = (Get-ChildItem $out -Recurse -File | Measure-Object Length -Sum).Sum
"staged $([math]::Round($size / 1MB)) MB in $out"
exit 0
