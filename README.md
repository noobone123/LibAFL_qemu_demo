# Prepare
You can download prebuilt rootfs for aarch64 from [https://drive.google.com/file/d/1K5Gb-S6vpLb6xmYicPKAvMJPtJ5J9xSN/view?usp=drive_link], extract the `rootfs` and put it in the current directory.

# Build
1. run `cargo make clean` to clean up all build
2. run `cargo make aarch64` to build the fuzzer and target binaries.

# Run
## Run the fuzzer
```bash
RUST_LOG=info ./build/h1k0_qemu_launcher \
    --input ./corpus \
    --output ./output \
    --tui \
    --cores 0-2 --asan-cores 0 --cmplog-cores 1 --tokens ./build/tiff.dict -- \
    -L ./rootfs ./build/bin/tiffinfo -Dcjrsw ./corpus/minisblack-1c-16b.tiff
```

## Run the fuzzer with Debugging Client 
```bash
RUST_BACKTRACE=full RUST_LOG=info ./build/h1k0_qemu_launcher \
    --input ./corpus \
    --output ./output \
    --tui \
    --client-stdout-file ./stdout.txt --client-stderr-file ./stderr.txt \
    --cores 0-2 --asan-cores 0 --cmplog-cores 1 --tokens ./build/tiff.dict -- \
    -L ./rootfs ./build/bin/tiffinfo -Dcjrsw ./corpus/minisblack-1c-16b.tiff
```

## Verify Crashes
1. Modify `Cargo.toml`, add `"simplemgr"` in features
2. run following command
    ```bash
    RUST_LOG=info ./build/h1k0_qemu_launcher \
    --input ./corpus \
    --output ./output \
    --log ./output/log.txt \
    --cores 0 --asan-cores 0 -r <input> -- \
    -L ./rootfs ./build/bin/tiffinfo -Dcjrsw <input>
    ```

## Important Arguments
- `--verbose`: Enable verbose output (Output clients' stdout and stderr to console, conflicts with `client_stdout_file` and `client_stderr_file`)
- `--client-stdout-file`: Redirect client stdout to a file (`/dev/null` is also a valid option)
- `--client-stderr-file`: Redirect client stderr to a file (`/dev/null` is also a valid option)
- `--log`: Redirect fuzzer log to a file
- `--tui`: Enable TUI mode (no fuzzer log)
- `RUST_BACKTRACE=full`: Enable backtrace, useful for debugging clients' crashes
- `RUST_LOG=info`: Enable info level log

## (Deprecated) Testing the crashes
1. Build with asan `ENABLE_ASAN=true cargo make x86_64`
2. Run crashes for testing.