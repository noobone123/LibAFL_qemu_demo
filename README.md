# Prepare
You can download prebuilt rootfs for aarch64 from [https://drive.google.com/file/d/1K5Gb-S6vpLb6xmYicPKAvMJPtJ5J9xSN/view?usp=drive_link], extract the `rootfs` and put it in the current directory.

# Build
1. run `cargo make clean` to clean up all build
2. <Optional> Modify the libafl path in `Cargo.toml`, now my libafl version is `commit d8460d14a2872d1281ac0eb55797d0dc63a2d144` 
3. run `cargo make x86_64` to build the fuzzer and target binaries.

# Run
## Simple Manager for testing and Debugging AsanModule
Run without asan_module
```
RUST_BACKTRACE=full RUST_LOG=info ./build/h1k0_qemu_launcher \
    --input ./corpus \
    --output ./output \
    --log ./output/log.txt \
    --cores 0 -r ./corpus/minisblack-1c-16b.tiff -- \ 
    ./build/bin/tiffinfo -Dcjrsw ./corpus/minisblack-1c-16b.tiff
```

Run with asan_module
```
RUST_BACKTRACE=full RUST_LOG=info ./build/h1k0_qemu_launcher \
    --input ./corpus \
    --output ./output \
    --log ./output/log.txt \
    --cores 0 --asan-cores 0 -r ./corpus/minisblack-1c-16b.tiff -- \
    ./build/bin/tiffinfo -Dcjrsw ./corpus/minisblack-1c-16b.tiff
```

Only `./corpus/logluv-3c-16b.tiff` can run without unexpected input.

## (Deprecated) Testing the crashes
1. Build with asan `ENABLE_ASAN=true cargo make x86_64`
2. Run crashes for testing.