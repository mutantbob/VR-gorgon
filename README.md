# prerequisites
`cargo install cargo-apk`

`cargo install cargo-ndk`

https://developer.oculus.com/downloads/package/oculus-openxr-mobile-sdk/


# building


Build, install, and run on the connected android device (tested on Oculus Quest 2)
```
ANDROID_HOME=~/Android/Sdk/ \
 ANDROID_NDK_ROOT=~/Android/Sdk/ndk/25.2.9519653/ \
 OPENXR_LIBDIR=~/vendor/ovr_openxr_mobile_sdk/OpenXR/Libs/Android/arm64-v8a/Debug/ \
    cargo apk run
```
Run clippy to detect opportunities to improve code idioms
```
ANDROID_HOME=~/Android/Sdk/ \
 ANDROID_NDK_ROOT=~/Android/Sdk/ndk/25.2.9519653/ \
 OPENXR_LIBDIR=~/vendor/ovr_openxr_mobile_sdk/OpenXR/Libs/Android/arm64-v8a/Debug/ \
    cargo ndk -t arm64-v8a -o app/src/main/jniLibs/  clippy
```
Use `adb` to install the `.apk` file and start the app (if you don't want to use `cargo apk run`)
```
(adb uninstall rust.vr_gorgon
adb install -r ./vr-gorgon/target/debug/apk/vr-gorgon.apk ) &&
adb shell am start -n rust.vr_gorgon/android.app.NativeActivity     \
-a android.intent.action.MAIN -c android.intent.category.LAUNCHER
```
