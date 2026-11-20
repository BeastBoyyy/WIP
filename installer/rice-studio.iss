; Rice Studio installer. Per user, no admin prompt, the user picks the folder (any drive).
; Build: powershell -File installer\stage.ps1, then ISCC installer\rice-studio.iss
; Output: target\installer\RiceStudio-Setup-<version>.exe
#define AppVer "0.1.0"
#define Stage "..\target\stage"

[Setup]
AppId={{A99BFE58-9570-444C-B163-C68593569E37}
AppName=Rice Studio
AppVersion={#AppVer}
AppPublisher=Rice Studio contributors
DefaultDirName={localappdata}\Programs\Rice Studio
DisableDirPage=no
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
OutputDir=..\target\installer
OutputBaseFilename=RiceStudio-Setup-{#AppVer}
Compression=lzma2/max
SolidCompression=yes
LZMAUseSeparateProcess=yes
WizardStyle=modern
UninstallDisplayIcon={app}\rice-studio.exe
UninstallDisplayName=Rice Studio
; Windhawk's mods live inside Explorer; never let the Restart Manager try to close Explorer
CloseApplications=no
; ponytail: upgrading over a running copy is not handled yet (stop Rice Studio first); Phase 6 adds it

[Files]
Source: "{#Stage}\*"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs ignoreversion
; createallsubdirs also brings the empty Flow UserData folder, which keeps Flow portable

[Icons]
Name: "{autoprograms}\Rice Studio"; Filename: "{app}\rice-studio.exe"
Name: "{autodesktop}\Rice Studio"; Filename: "{app}\rice-studio.exe"; Tasks: desktopicon

[Tasks]
Name: desktopicon; Description: "Put Rice Studio on the desktop"; Flags: unchecked

[Run]
Filename: "{app}\rice-studio.exe"; Description: "Open Rice Studio"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; puts Windows back the way it was first: stops everything, restores the original wallpaper and bar
; settings, removes the fonts it installed and its start-up entry
Filename: "{app}\rice-studio.exe"; Parameters: "--uninstall"; Flags: runhidden waituntilterminated; RunOnceId: "RestoreWindows"

[UninstallDelete]
; only what Rice Studio created; the folder itself goes only if nothing else is in it
Type: filesandordirs; Name: "{app}\apps"
Type: filesandordirs; Name: "{app}\data"
Type: dirifempty; Name: "{app}"

