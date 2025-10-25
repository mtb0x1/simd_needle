#!/bin/bash 

cargo build --release
base_command="cargo run --release --bin "
haystack="haystacks/opensubtitles/en-huge.txt"
needle="hello"
warmup=3
command_name="memchr crate with --haystack='${haystack}' --needle='${needle}'"
command="${base_command} memchr_haystacks '${haystack}' '${needle}'" 
command_name1="memchr_libc with --haystack='${haystack}' --needle='${needle}'"
command1="${base_command} memchr_libc_haystacks '${haystack}' '${needle}'"
command_name2="simd with --haystack='${haystack}' --needle='${needle}'"
command2="${base_command} simd_haystacks '${haystack}' '${needle}'"

hyperfine \
    -N \
    --warmup="${warmup}" \
    --time-unit=millisecond \
    --command-name="${command_name}" "${command}" \
    --command-name="${command_name1}" "${command1}" \
    --command-name="${command_name2}" "${command2}" \
