#!/usr/bin/env sh

set -eu

TARGET_DIR="${1:-}"

if [ -z "$TARGET_DIR" ]; then
  echo "target directory is not defined"
  echo "Usage: configure.sh <directory>"
  exit 1
fi

# Expand ~
case "$TARGET_DIR" in
  "~"/*)
    TARGET_DIR="$HOME/${TARGET_DIR#\~/}"
    ;;
  "~")
    TARGET_DIR="$HOME"
    ;;
esac

# Resolve absolute path if possible
if command -v realpath >/dev/null 2>&1; then
  TARGET_DIR="$(realpath "$TARGET_DIR")"
fi

if [ ! -d "$TARGET_DIR" ]; then
  echo "Directory does not exist: $TARGET_DIR"
  exit 1
fi

OS="$(uname -s)"
CURRENT_SHELL="$(basename "${SHELL:-}")"

echo "Detected OS: $OS"
echo "Detected shell: $CURRENT_SHELL"

add_line_if_missing() {
  file="$1"
  line="$2"

  mkdir -p "$(dirname "$file")"
  touch "$file"

  if grep -Fqs "$line" "$file"; then
    echo "PATH already configured in $file"
  else
    printf '\n%s\n' "$line" >> "$file"
    echo "Updated $file"
  fi
}

configure_fish() {
  fish_config="$HOME/.config/fish/config.fish"
  line="fish_add_path \"$TARGET_DIR\""

  add_line_if_missing "$fish_config" "$line"

  echo
  echo "Apply changes with:"
  echo "  source \"$fish_config\""
}

configure_posix_shell() {
  rc_file="$1"
  line="export PATH=\"$TARGET_DIR:\$PATH\""

  add_line_if_missing "$rc_file" "$line"

  echo
  echo "Apply changes with:"
  echo "  source \"$rc_file\""
}

case "$CURRENT_SHELL" in
  bash)
    if [ "$OS" = "Darwin" ]; then
      configure_posix_shell "$HOME/.bash_profile"
    else
      configure_posix_shell "$HOME/.bashrc"
    fi
    ;;

  zsh)
    configure_posix_shell "$HOME/.zshrc"
    ;;

  fish)
    configure_fish
    ;;

  sh)
    configure_posix_shell "$HOME/.profile"
    ;;

  *)
    echo "Unknown shell: $CURRENT_SHELL"
    echo "Using ~/.profile"
    configure_posix_shell "$HOME/.profile"
    ;;
esac

echo
echo "Done."
echo "Added to PATH:"
echo "  $TARGET_DIR"
