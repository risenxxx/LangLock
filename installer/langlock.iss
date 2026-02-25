; LangLock Inno Setup Script
; Modern installer for LangLock - Caps Lock Language Switcher

#define MyAppName "LangLock"
#define MyAppPublisher "LangLock Contributors"
#define MyAppURL "https://github.com/risenxxx/langlock"
#define MyAppExeName "langlock.exe"

[Setup]
; Unique AppId - DO NOT CHANGE after first release
AppId={{B5E7F8A9-C0D1-4E2F-A3B4-567890ABCDEF}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
; No license page
LicenseFile=
; Allow user to skip Start Menu folder
AllowNoIcons=yes
; Output settings
OutputDir=..\target\installer
OutputBaseFilename=langlock-setup-{#MyAppVersion}
; Modern visual style
WizardStyle=modern
; Compression
Compression=lzma2
SolidCompression=yes
; Require admin for Program Files installation
PrivilegesRequired=admin
; 64-bit only
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
; Uninstall settings
UninstallDisplayIcon={app}\{#MyAppExeName}
UninstallDisplayName={#MyAppName}
; Allows upgrading without uninstalling
UsePreviousAppDir=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[CustomMessages]
english.RunAtStartup=Run at Windows startup
english.StartupGroup=Startup:
russian.RunAtStartup=Запускать при входе в Windows
russian.StartupGroup=Автозапуск:

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startup"; Description: "{cm:RunAtStartup}"; GroupDescription: "{cm:StartupGroup}"; Flags: unchecked

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
; Create scheduled task for startup (runs with admin since installer is elevated)
Filename: "schtasks"; Parameters: "/create /tn ""LangLock"" /tr """"""{app}\{#MyAppExeName}"""""" /sc onlogon /rl highest /f"; Flags: runhidden; Tasks: startup
; Launch after installation (user can uncheck)
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; Remove scheduled task on uninstall
Filename: "schtasks"; Parameters: "/delete /tn ""LangLock"" /f"; Flags: runhidden

[Code]
// Close running instance before install/uninstall
function InitializeSetup(): Boolean;
var
  ResultCode: Integer;
begin
  Result := True;
  // Try to close running instance gracefully
  Exec('taskkill', '/f /im langlock.exe', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
end;

function InitializeUninstall(): Boolean;
var
  ResultCode: Integer;
begin
  Result := True;
  // Close running instance before uninstall
  Exec('taskkill', '/f /im langlock.exe', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
end;
