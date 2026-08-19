# Carriage Run release budgets

The release gate is intentionally measured from the publisher output rather
than from debug artifacts.

| Artifact | Budget | Baseline measurement |
| --- | ---: | ---: |
| WebGL carriage_run.wasm | 2.5 MiB | 1.79 MiB |
| Windows executable | 8 MiB | 1.70 MiB |
| WebGL package | 16 MiB | 11.73 MiB |
| Windows package | 16 MiB | 9.74 MiB |

Run the project publisher to regenerate dist/, then record any changed
measured values here before a release. The CI workflow enforces the WebGL
binary budget; the publisher output and this table cover the packaged asset
budget. PNGs are kept lossless because the atlases contain pixel-art edges and
alpha masks.
