
[Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidUsingCmdletAliases', '')]
[Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSUseDeclaredVarsMoreThanAssignments', '')]

param()

$current = $PSScriptRoot
# . "${current}/color.ps1"

$redCode   = '#FD1C35'
$greenCode = '#92B55F'
$textCode  = '#969696'
$yellowCode= '#E8DA5E'

$buildSource = 'term'

function output() {
  param([string]$text, [switch]$isError)

  if ($source -eq 'term'){ 
    Write-HostColor $text -ForegroundColor $($isError ? $redCode : $greenCode)
  }
  else { Write-Host $text }
}

# --| Get Torch ------------------
# --|-----------------------------
function GetTorch() {
  param(
    [string]$source
  )
  $buildSource = $source
  $canContinue = $true

  $libTorch = "${HOME}/libtorch"
  $libTorchLib = "${libTorch}/lib"
  $libTorchInclude = "${libTorch}/include"

  output "Current: ${current}"

  if (!(Test-Path $libTorchLib)) {
    & "${current}/setup.sh"
  }

  $env:LIBTORCH = $libTorchLib  
  $env:LIBTORCH_INCLUDE = $libTorchInclude
  $env:LIBTORCH_LIB = $libTorchLib

}

# --| Buildir2d ----------------------
# --|-----------------------------
function RunBuild() {
  param(
    [string]$source
  )
  $buildSource = $source
  $canContinue = $true
 
  try { $output = $(cargo build 2>&1); $canContinue = $? }
  catch { output "Build Failed: ${_}" -isError; $canContinue = $false }

  $errorLines = $output | Select-String -Pattern "error: " -AllMatches -CaseSensitive | Select-Object 

  if($errorLines.Count -gt 0){
    $errorLines | ForEach-Object { output $_ -isError }
    $canContinue = $false
  }

  cp $current/../target/debug/vectorizer $HOME/.local/bin

  if ($canContinue) { output "Compilation Successful" }
  else { output "Failed to compile!" -isError; exit 1 }
}

# --| Helper Functions ----------------
# --|----------------------------------
$Script:defaultColors = @{
  Black  = "#0c0c0c";
  DarkBlue = "#0037da";
  DarkGreen = "#13a10e";
  DarkCyan = "#3a96dd";
  DarkRed = "#c50f1f";
  DarkMagenta = "#891798";
  DarkYellow = "#c19a00";
  Gray  = "#cccccc";
  DarkGray = "#767676";
  Blue = "#3b79ff";
  Green = "#16c60c";
  Cyan = "#61d6d6";
  Red = "#e74856";
  Magenta = "#b4009f";
  Yellow = "#f9f1a5";
  White = "#f2f2f2";
};


function ConvertFrom-Hex {
  param(
      [Parameter(Mandatory = $true, Position = 1)] [string] $Color
      )

  if ($Color -in $Script:defaultColors.Keys) {
    $Color = $Script:defaultColors[$Color];
  }

  if ($Color -notmatch "^#[0-9A-F]{6}$") {
    throw "Hex color $Color is not valid!";
  }

# Remove # symbol
  $Color = $Color.Remove(0, 1);

  $red    = $Color.Remove(2, 4);
  $green  = $Color.Remove(4, 2).Remove(0, 2);
  $blue   = $Color.Remove(0, 4);

  $red    = [System.Convert]::ToInt32($red, 16);
  $green  = [System.Convert]::ToInt32($green, 16);
  $blue   = [System.Convert]::ToInt32($blue, 16);

  return "$red;$green;$blue";
}

function Write-HostColor {
  param(
      [Parameter(Mandatory = $true, Position = 1)] [string] $Value,
      [Parameter(Mandatory = $true, Position = 2)] [string] $ForegroundColor,
      [switch] $NoNewLine
  )

  $ForegroundColor = ConvertFrom-Hex -Color $ForegroundColor;
  Write-Host "`e[38;2;${ForegroundColor}m$Value`e[0m" -NoNewline:$NoNewLine;
}
