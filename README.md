# Prepare
You can download prebuilt rootfs for aarch64 from [https://drive.google.com/file/d/1K5Gb-S6vpLb6xmYicPKAvMJPtJ5J9xSN/view?usp=drive_link], extract the `rootfs` and put it in the current directory.

# Build
1. run `cargo make clean` to clean up all build
2. <Optional> Modify the libafl path in `Cargo.toml`, now my libafl version is `commit ca647f0c30593e9f1670d334a4b2b61000c66e21` 
3. run `cargo make aarch64` to build the fuzzer and target binaries.

# Run
## Simple Manager for testing and Debugging
Now I'm testing my harness using following command.
```
./target/aarch64/release/h1k0_qemu_launcher \
    --input ./corpus \
    --output ./target/output \
    --log ./target/output/log.txt \
    --cores 0 \
    -- \
    -L ./rootfs --strace \
    ./target/aarch64/build-tiff/bin/tiffinfo -Dcjrsw ./infile
```