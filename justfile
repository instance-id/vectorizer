#!/usr/bin/env -S just --justfile

# Powershell is just better/more flexible for this kind 
# of stuff than bash IMO Sorry to anyone who is offended.

# --| Cross platform shebang ----------
# --|----------------------------------
shebang := if os() == 'windows' {
  'pwsh.exe'
} else {
  '/usr/bin/env -S pwsh -noprofile -nologo'
}

# set shell := ["/usr/bin/env", "pwsh" ,"-noprofile", "-nologo", "-c"]
set windows-shell := ["pwsh.exe","-NoLogo", "-noprofile", "-c"]

build := './scripts/build.ps1'

# --| Actions -------------------------
# --|----------------------------------

torch source='term': 
  just _torch-{{os()}} {{source}}

_torch-linux source:
  #!{{shebang}}
  . {{build}}
  GetTorch {{source}} 

# --| Test ------------------
# --|------------------------

test run debug='no': 
  just _test-{{os()}} {{run}} {{debug}}

_test-linux run debug:
  #!{{shebang}}
  cargo test

_test-windows run debug:
  # Do Windows Things

# --| Build -----------------
# --|------------------------

build source='term': 
  just torch {{source}}
  just _build-{{os()}} {{source}}

_build-linux source:
  #!{{shebang}}
  . {{build}}
  RunBuild {{source}}
 
_build-windows source:
  # Do Windows Things

#!{{shebang}}
# . {{build_steps}}
# RunBuild {{run}}

# --| Run -------------------
# --|------------------------

run source='term': 
  just build {{source}}
  just _run-{{os()}} {{source}}

_run-linux source:
  #!{{shebang}}
  if (Test-Path -Path $HOME/libtorch) {
    $LIBTORCH = $env:LIBTORCH = "$HOME/libtorch"
    $env:LD_LIBRARY_PATH="${LIBTORCH}/lib:$env:LD_LIBRARY_PATH"
    ./target/debug/vectorizer -p /mnt/x/GitHub/fubark/cyber/test -e=cy --index
  } 


on-save file:
  ./target/debug/vectorizer -p {{file}} --upload
  write-host "Upload Complete."
