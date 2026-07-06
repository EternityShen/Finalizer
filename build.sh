cargo build --target aarch64-linux-android --release
cp ./target/aarch64-linux-android/release/finalizer ./mode/system/bin/finalizer
cd ./mode/
zip -r ../build/finalizer.zip *
