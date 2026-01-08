NIGHTLY="${PINNED_CI_NIGHTLY:=nightly-2025-09-27}"

case $TARGET in
    "riscv64-linux-android" | "riscv64gc-unknown-linux-gnu") 
        NO_STD="1" ;;
    "wasm32-unknown-unknown")
        NO_STD="2" ;;
    *) 
        NO_STD="0" ;;
esac

if [[ "$TYPE" = "static" ]] || [[ $NO_STD > 0 ]]; then
    rustup toolchain install --no-self-update $NIGHTLY --component rust-src
    rustup target add $TARGET --toolchain $NIGHTLY
else
    rustup target add $TARGET
fi

# Explanation of flags:
# -Zunstable-options: enables other unstable flags
# -Cpanic=immediate-abort: removes unwind machinery and associated Debug impls
# --config=profile.release.codegen-units=1: generate the code in a single process to enable more opportunities for optimization
# -Zbuild-std=std,panic_abort: rebuild the standard library with panic-abort behavior and our RUSTFLAGS

if [[ $NO_STD > 0 ]]; then
    RUSTFLAGS="-Zunstable-options -Cpanic=immediate-abort $RUSTFLAGS"
fi

cargo \
    $( ([[ $TYPE == "static" ]] || [[ $NO_STD > 0 ]]) && echo "+$NIGHTLY" ) \
    "rustc" \
    "--manifest-path=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )/Cargo.toml" \
    "--crate-type=$( [[ "$TYPE" == "static" ]] && echo "staticlib" || echo "cdylib" )" \
    "--release" \
    "--config=profile.release.panic=\"abort\"" \
    "--config=profile.release.codegen-units=1" \
    "--no-default-features" \
    "--features=$( [[ $NO_STD == 1 ]] && echo "libc_alloc,looping_panic_handler" || echo "simple_logger"),$FEATURES" \
    $( ( [[ $TYPE = "dynamic" ]] && [[ $NO_STD = 1 ]] ) && echo "-Zbuild-std=core,alloc") \
    $( ( [[ $TYPE = "static" ]] || [[ $NO_STD > 0 ]] ) && echo "-Zbuild-std=std,panic_abort" ) \
    "--target=$TARGET" \
    "--" \
    "--emit" "link=$OUT"
