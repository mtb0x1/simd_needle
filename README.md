simd_needle
===========

Small utility to find occurrences of a needle in a haystack using a streaming Finder.

Usage examples
--------------

Search a single file:

```powershell
cargo run -- "needle text" path/to/haystack.txt
```

Search multiple haystacks in a directory in parallel:

```powershell
cargo run -- "deadbeef" --hex --haystacks-dir path/to/haystacks_dir
```

When using `--haystacks-dir`, results are printed as `path:offset` for each match.
