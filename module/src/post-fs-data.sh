#!/system/bin/sh

MODDIR=${0%/*}
if [ "$ZYGISK_ENABLED" = "1" ]; then
  exit 0
fi

cd "$MODDIR"

if [ "$(which magisk)" ]; then
  for file in ../*; do
    if [ -d "$file" ] && [ -d "$file/zygisk" ] && ! [ -f "$file/disable" ]; then
      if [ -f "$file/post-fs-data.sh" ]; then
        cd "$file"
        log -p i -t "r0z-sh" "Manually trigger post-fs-data.sh for $file"
        sh "$(realpath ./post-fs-data.sh)"
        cd "$MODDIR"
      fi
    fi
  done
fi

export TMP_PATH="$MODDIR"
[ "$DEBUG" = true ] && export RUST_BACKTRACE=1

if command -v resetprop >/dev/null 2>&1; then
  resetprop ro.dalvik.vm.native.bridge libzn_loader.so
fi

./bin/r0zd daemon </dev/null >/dev/null 2>&1 &

# Avoid a race where zygote loads native bridge before daemon socket is ready.
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
  [ -S "$MODDIR/cp64.sock" ] && break
  [ -S "$MODDIR/cp32.sock" ] && break
  sleep 0.2
done
