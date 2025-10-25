#!/bin/bash
set -e

cargo build --release

base_command="cargo run --release --bin "
haystack="${1:-haystacks/opensubtitles/en-huge.txt}"
needle="hello"
warmup=3
command_name="memchr crate with --haystack='${haystack}' --needle='${needle}'"
command="${base_command} memchr_haystacks '${haystack}' '${needle}'"
command_name1="memchr_libc with --haystack='${haystack}' --needle='${needle}'"
command1="${base_command} memchr_libc_haystacks '${haystack}' '${needle}'"
command_name2="simdx86_64 with --haystack='${haystack}' --needle='${needle}'"
command2="${base_command} simdx86_64_haystacks '${haystack}' '${needle}'"
command_name3="simd with --haystack='${haystack}' --needle='${needle}'"
command3="${base_command} simd_haystacks '${haystack}' '${needle}'"
command_name4="simd_mmap_finder with --haystack='${haystack}' --needle='${needle}'"
command4="${base_command} simd_mmap_finder_haystacks '${haystack}' '${needle}'"

if [ "$(uname -m)" = "x86_64" ]; then 
    echo "Running on x86_64 architecture, including simdx86_64 benchmark."
    hyperfine \
    -N \
    --warmup="${warmup}" \
    --time-unit=millisecond \
    --command-name="${command_name}" "${command}" \
    --command-name="${command_name1}" "${command1}" \
    --command-name="${command_name2}" "${command2}" \
    --command-name="${command_name3}" "${command3}" \
    --command-name="${command_name4}" "${command4}"
else
    echo "Not running on x86_64 architecture, excluding simdx86_64 benchmark."
    hyperfine \
    -N \
    --warmup="${warmup}" \
    --time-unit=millisecond \
    --command-name="${command_name}" "${command}" \
    --command-name="${command_name1}" "${command1}" \
    --command-name="${command_name3}" "${command3}" \
    --command-name="${command_name4}" "${command4}"
fi
 