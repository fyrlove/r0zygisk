MODDIR=${0%/*}/..

export TMP_PATH="$MODDIR"

HIDE_LIST_FILE="$MODDIR/hide_list.conf"

ensure_hide_list() {
  if [ ! -f "$HIDE_LIST_FILE" ]; then
    : >"$HIDE_LIST_FILE"
    chmod 0600 "$HIDE_LIST_FILE" 2>/dev/null || true
  fi
}

validate_package() {
  case "$1" in
    *[!A-Za-z0-9._]*|'')
      return 1
      ;;
  esac
  return 0
}

list_hide_apps() {
  ensure_hide_list
  sed '/^[[:space:]]*$/d; /^[[:space:]]*#/d' "$HIDE_LIST_FILE" 2>/dev/null | sort -u
}

list_installed_apps() {
  pm list packages -3 2>/dev/null | sed 's/^package://' | sed '/^[[:space:]]*$/d' | sort -u
}

add_hide_app() {
  ensure_hide_list
  pkg="$1"
  validate_package "$pkg" || return 1
  if ! grep -Fxq "$pkg" "$HIDE_LIST_FILE" 2>/dev/null; then
    printf '%s\n' "$pkg" >>"$HIDE_LIST_FILE"
  fi
  list_hide_apps >"$HIDE_LIST_FILE.tmp"
  mv "$HIDE_LIST_FILE.tmp" "$HIDE_LIST_FILE"
}

remove_hide_app() {
  ensure_hide_list
  pkg="$1"
  validate_package "$pkg" || return 1
  grep -Fxv "$pkg" "$HIDE_LIST_FILE" 2>/dev/null >"$HIDE_LIST_FILE.tmp" || true
  mv "$HIDE_LIST_FILE.tmp" "$HIDE_LIST_FILE"
}

if [ "$1" = "hide-list" ]; then
  case "$2" in
    list)
      list_hide_apps
      exit 0
      ;;
    apps)
      list_installed_apps
      exit 0
      ;;
    add)
      add_hide_app "$3"
      exit $?
      ;;
    remove)
      remove_hide_app "$3"
      exit $?
      ;;
  esac
fi

exec "$MODDIR/bin/r0zd" "$@"
