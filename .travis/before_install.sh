#!/usr/bin/env bash
set -ev

# Install Rust
if [ -z ${TRAVIS_RUST_VERSION+w} ]; then
  curl https://sh.rustup.rs -sSf | bash -s -- -y
fi

if [ $TRAVIS_OS_NAME == "osx" ]; then
  if [ $PYTHON_TESTS == true ]; then
    # install pyenv
    git clone --depth 1 https://github.com/pyenv/pyenv ~/.pyenv
    PYENV_ROOT="$HOME/.pyenv"
    PATH="$PYENV_ROOT/bin:$PATH"
    eval "$(pyenv init -)"

    case "${TOXENV}" in
      "py27")
        pyenv install 2.7.14
        pyenv global 2.7.14
        ;;
      "py36")
        pyenv install 3.6.1
        pyenv global 3.6.1
        ;;
    esac
    pyenv rehash

    # A manual check that the correct version of Python is running.
    python --version
  fi

  if [ "${IOS_SWIFT_TESTS}" == "true" ]; then
    PATH="$HOME/.cargo/bin:$PATH"
    rustup target install x86_64-apple-ios
  fi
fi
