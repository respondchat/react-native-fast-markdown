export IPHONEOS_DEPLOYMENT_TARGET=16.0
export RUSTFLAGS="-C panic=unwind -C debuginfo=2 "
NDK=/Users/user/Library/Android/sdk/ndk/28.0.13004108/toolchains/llvm/prebuilt/darwin-x86_64/bin
export PATH=$NDK:$PATH

cargo +nightly b --target aarch64-apple-ios
cargo +nightly b --target aarch64-apple-ios-macabi
cargo +nightly b --target aarch64-apple-ios-sim
cargo +nightly b --target x86_64-apple-ios
cargo +nightly b --target x86_64-linux-android
cargo +nightly b --target aarch64-linux-android

# strip -x ./target/aarch64-apple-ios/debug/libreact_native_fast_markdown.a
# strip -x ./target/aarch64-apple-ios-sim/debug/libreact_native_fast_markdown.a
# strip -x ./target/x86_64-apple-ios/debug/libreact_native_fast_markdown.a
# strip -x ./target/x86_64-linux-android/debug/libreact_native_fast_markdown.a
# strip -x ./target/aarch64-linux-android/debug/libreact_native_fast_markdown.a

$NDK/aarch64-linux-android24-clang++ -shared -o ./target/aarch64-linux-android/debug/libreact_native_fast_markdown.so -Wl,--whole-archive ./target/aarch64-linux-android/debug/libreact_native_fast_markdown.a -Wl,--no-whole-archive -llog -lc -lm -static-libstdc++ -fexceptions -frtti

$NDK/x86_64-linux-android24-clang++ -shared -o ./target/x86_64-linux-android/debug/libreact_native_fast_markdown.so -Wl,--whole-archive ./target/x86_64-linux-android/debug/libreact_native_fast_markdown.a -Wl,--no-whole-archive -llog -lc -lm -static-libstdc++ -fexceptions -frtti

cp ./target/aarch64-linux-android/debug/libreact_native_fast_markdown.so ../../app/client/android/app/src/debug/jniLibs/arm64-v8a/libfast-markdown.so
cp ./target/x86_64-linux-android/debug/libreact_native_fast_markdown.so ../../app/client/android/app/src/debug/jniLibs/x86_64/libfast-markdown.so


cp ./target/aarch64-apple-ios/debug/libreact_native_fast_markdown.a ../../app/client/ios/libfast-markdown.a
cp ./target/aarch64-apple-ios-sim/debug/libreact_native_fast_markdown.a ../../app/client/ios/libfast-markdown-sim.a
cp ./target/x86_64-apple-ios/debug/libreact_native_fast_markdown.a ../../app/client/ios/libfast-markdown-sim-x86.a
cp ./target/aarch64-apple-ios-macabi/debug/libreact_native_fast_markdown.a ../../app/client/ios/libfast-markdown-catalyst.a
cd ../../app/client/ios/
lipo libfast-markdown-sim.a libfast-markdown-sim-x86.a -create -output libfast-markdown-sim.a
rm libfast-markdown-sim-x86.a

