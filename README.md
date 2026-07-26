SizeOf
====
Reports the size of a given directory (recursively) or file.

```
Usage: sizeof.exe [OPTIONS] <ROOT>

Arguments:
  <ROOT>

Options:
  -x, --exclude <EXCLUDE>  Exclude files that match any of these patterns
  -i, --include <INCLUDE>  Only include files that match any of these patterns
  -r, --raw                Print raw size in bytes
  -v, --verbose            Print files as they are scanned
  -p, --progress           Show progress bar
      --human              Use humanized size units (GB -> GiB, MB -> MiB, etc.)
      --ignore-errors      Instead of panicking at errors, skip them
  -h, --help               Print help
  -V, --version            Print version
```