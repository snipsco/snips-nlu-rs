# snips-queries

### Setup

1. Create a virtual env:
```bash
virtualenv venv
```

2. Activate it:
```bash
venv/bin/activate
```

3. Create `venv/pip.conf` file within the virtual env and append:
```bash
[global]
index = https://nexus-repository.snips.ai/repository/pypi-internal/pypi
index-url = https://pypi.python.org/simple/
extra-index-url = https://nexus-repository.snips.ai/repository/pypi-internal/simple
```
