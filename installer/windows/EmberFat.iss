#define MyAppName "EmberFat"
#ifndef MyAppVersion
  #error "Build with scripts/package-windows.ps1 so the version matches the plugin."
#endif
#define MyPublisher "eM4"
#define MyVst3Bundle "EmberFat.vst3"

[Setup]
AppId={{B9C4D18B-8A9B-4F5A-A2A2-8DDA4E9B6F10}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyPublisher}
DefaultDirName={commoncf}\VST3
DisableDirPage=no
PrivilegesRequired=admin
OutputDir=..\..\dist
OutputBaseFilename=EmberFat-Windows-Setup
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
Uninstallable=yes
Compression=lzma2
SolidCompression=yes

[Files]
Source: "..\..\dist\{#MyVst3Bundle}\*"; DestDir: "{app}\{#MyVst3Bundle}"; Flags: recursesubdirs createallsubdirs ignoreversion

[Messages]
WelcomeLabel2=This installs the EmberFat VST3 effect for compatible 64-bit audio applications.

[Code]
function InitializeSetup(): Boolean;
begin
  Result := IsWin64;
  if not Result then
    MsgBox('EmberFat requires 64-bit Windows.', mbError, MB_OK);
end;
