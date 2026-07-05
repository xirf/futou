# Python

Python embeddable packages from [python.org](https://www.python.org/downloads/windows/).

## Versions

| Version | Source |
|---------|--------|
| 3.13.3 | embeddable amd64 |
| 3.12.9 | embeddable amd64 |

## Activation

```bash
futou use python 3.13.3
```

Creates a `.bat` shim for `python.exe`.

::: warning Embeddable Package
The embeddable distribution does **not** include `pip`. To install packages, download `get-pip.py` from [bootstrap.pypa.io](https://bootstrap.pypa.io/get-pip.py) and run:

```bash
python get-pip.py
```
:::

## Package Management

After installing pip, packages install into `Lib\site-packages\` within the runtime directory. Switching Python versions switches the package set — each version has its own isolated environment.
