MODDIR=${0%/*}/..

export TMP_PATH="$MODDIR"

exec "$MODDIR/bin/r0zd" "$@"
