export IPHONEOS_DEPLOYMENT_TARGET=16.0
# export RUSTFLAGS="-Zlocation-detail=none -Cpanic=abort -Cdebuginfo=0 -Cstrip=symbols -Clink-arg=--gc-sections"

cargo +nightly b --release --target aarch64-apple-ios -Z build-std-features="optimize_for_size"
cargo +nightly b --release --target aarch64-apple-ios-sim -Z build-std-features="optimize_for_size"
cargo +nightly b --release --target x86_64-apple-ios -Z build-std-features="optimize_for_size"
cargo +nightly b --release --target x86_64-linux-android -Z build-std-features="optimize_for_size"
cargo +nightly b --release --target aarch64-linux-android -Z build-std-features="optimize_for_size"

# strip -x ./target/aarch64-apple-ios/release/libreact_native_fast_markdown.a
# strip -x ./target/aarch64-apple-ios-sim/release/libreact_native_fast_markdown.a
# strip -x ./target/x86_64-apple-ios/release/libreact_native_fast_markdown.a
# strip -x ./target/x86_64-linux-android/release/libreact_native_fast_markdown.a
# strip -x ./target/aarch64-linux-android/release/libreact_native_fast_markdown.a

cp ./target/aarch64-apple-ios/release/libreact_native_fast_markdown.a ../../app/client/ios/libfast-markdown.a
cp ./target/aarch64-apple-ios-sim/release/libreact_native_fast_markdown.a ../../app/client/ios/libfast-markdown-sim.a
cp ./target/x86_64-apple-ios/release/libreact_native_fast_markdown.a ../../app/client/ios/libfast-markdown-sim-x86.a
cd ../../app/client/ios/
lipo libfast-markdown-sim.a libfast-markdown-sim-x86.a -create -output libfast-markdown-sim.a
rm libfast-markdown-sim-x86.a

