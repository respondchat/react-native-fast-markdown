export IPHONEOS_DEPLOYMENT_TARGET=16.0
export RUSTFLAGS="-Zlocation-detail=none -Cpanic=abort -Cdebuginfo=0 -Cstrip=symbols"
NDK=/Users/user/Library/Android/sdk/ndk/28.0.13004108/toolchains/llvm/prebuilt/darwin-x86_64/bin
export PATH=$NDK:$PATH

cargo +nightly b --release --target aarch64-apple-ios -Z build-std-features="optimize_for_size"
cargo +nightly b --release --target aarch64-apple-ios-sim -Z build-std-features="optimize_for_size"
cargo +nightly b --release --target aarch64-apple-ios-macabi -Z build-std=std,panic_abort -Z build-std-features="optimize_for_size" -Z build-std-features=panic_immediate_abort
cargo +nightly b --release --target x86_64-apple-ios-macabi -Z build-std=std,panic_abort -Z build-std-features="optimize_for_size" -Z build-std-features=panic_immediate_abort
cargo +nightly b --release --target x86_64-apple-ios -Z build-std-features="optimize_for_size"
cargo +nightly b --release --target x86_64-linux-android -Z build-std-features="optimize_for_size"
cargo +nightly b --release --target aarch64-linux-android -Z build-std-features="optimize_for_size"

# strip -x ./target/aarch64-apple-ios/release/libreact_native_fast_markdown.a
# strip -x ./target/aarch64-apple-ios-sim/release/libreact_native_fast_markdown.a
# strip -x ./target/x86_64-apple-ios/release/libreact_native_fast_markdown.a
# strip -x ./target/x86_64-linux-android/release/libreact_native_fast_markdown.a
# strip -x ./target/aarch64-linux-android/release/libreact_native_fast_markdown.a

$NDK/aarch64-linux-android24-clang++ -shared -o ./target/aarch64-linux-android/release/libreact_native_fast_markdown.so -Wl,--whole-archive ./target/aarch64-linux-android/release/libreact_native_fast_markdown.a -Wl,--no-whole-archive -llog -lc -lm -static-libstdc++ -fexceptions -frtti
$NDK/x86_64-linux-android24-clang++ -shared -o ./target/x86_64-linux-android/release/libreact_native_fast_markdown.so -Wl,--whole-archive ./target/x86_64-linux-android/release/libreact_native_fast_markdown.a -Wl,--no-whole-archive -llog -lc -lm -static-libstdc++ -fexceptions -frtti

cp ./target/aarch64-linux-android/release/libreact_native_fast_markdown.so ../../app/client/android/app/src/main/jniLibs/arm64-v8a/libfast-markdown.so
cp ./target/x86_64-linux-android/release/libreact_native_fast_markdown.so ../../app/client/android/app/src/main/jniLibs/x86_64/libfast-markdown.so


cp ./target/aarch64-apple-ios/release/libreact_native_fast_markdown.a ../../app/client/ios/libfast-markdown.a
cp ./target/aarch64-apple-ios-sim/release/libreact_native_fast_markdown.a ../../app/client/ios/libfast-markdown-sim.a
cp ./target/aarch64-apple-ios-macabi/release/libreact_native_fast_markdown.a ../../app/client/ios/libfast-markdown-catalyst.a
cp ./target/x86_64-apple-ios-macabi/release/libreact_native_fast_markdown.a ../../app/client/ios/libfast-markdown-catalyst-x86.a
cp ./target/x86_64-apple-ios/release/libreact_native_fast_markdown.a ../../app/client/ios/libfast-markdown-sim-x86.a
cd ../../app/client/ios/
lipo libfast-markdown-sim.a libfast-markdown-sim-x86.a -create -output libfast-markdown-sim.a
lipo libfast-markdown-catalyst.a libfast-markdown-catalyst-x86.a -create -output libfast-markdown-catalyst.a
rm libfast-markdown-sim-x86.a
rm libfast-markdown-catalyst-x86.a

