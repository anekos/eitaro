#!/bin/bash

set -euC
# set -o pipefail

# exec 5>> /tmp/xmosh/shell-script-debug.out
# BASH_XTRACEFD="5"
# PS4='$LINENO: '
# set -x


function t() {
  local expect="$1"
  shift
  local command=("$@")

  local actual

  actual="$(cargo run --quiet -- "${command[@]}" 2>&1)"

  echo "${command[*]}"
  echo -n '  → '

  if [ "$expect" = "$actual" ]
  then
    echo 'OK'
  else
    printf "NG: expect = %q, actual = %q\n" "$expect" "$actual"
    exit 1
  fi
}

t "cat-o'-nine-tails" lemmatize "cat-o'-nine-tails"
t 'monster' lemmatize 'monster'
t 'monster' lemmatize 'monsters'
t 'sconce' lemmatize 'sconces'
