# Build
run `cargo make aarch64`
run `cargo make clean` to clean up all build


# Run
## Single Test
```
./target/aarch64/release/h1k0_qemu_launcher \
    --input ./corpus \
    --output ./target/output \
    --log ./target/output/log.txt \
    --cores 0 \
    --verbose \
    -- \
    -L ./rootfs \
    ./target/aarch64/build-tiff/bin/tiffinfo
```