#!/bin/bash
set -ev

export PATH="$HOME/.cargo/bin:$PATH"

PYTHON_PATH=$(which python"$PYTHON_VERSION")
COMMIT_ID=$(git rev-parse --short HEAD)
VENV_PATH="/tmp/venv$PYTHON_VERSION-$COMMIT_ID"

warn() {
  echo "$@" >&2
}

die() {
  warn "$@"
  exit 1
}

escape() {
	echo "$1" | sed 's/\([\.\$\*]\)/\\\1/g'
}

has() {
	local item=$1; shift
	echo " $@ " | grep -q " $(escape $item) "
}
