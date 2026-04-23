#!/system/bin/sh

DEBUG=@DEBUG@

MODDIR=${0%/*}
if [ "$ZYGISK_ENABLED" = "1" ]; then
  exit 0
fi

cd "$MODDIR"

if [ "$(which magisk)" ]; then
  for file in ../*; do
    if [ -d "$file" ] && [ -d "$file/zygisk" ] && ! [ -f "$file/disable" ]; then
      if [ -f "$file/service.sh" ]; then
        cd "$file"
        log -p i -t "r0z-sh" "Manually trigger service.sh for $file"
        sh "$(realpath ./service.sh)"
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
./bin/r0zd service-stage
