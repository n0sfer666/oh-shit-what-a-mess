#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

require_cargo() {
  if ! command -v cargo >/dev/null 2>&1; then
    echo "❌ cargo не найден. Установи Rust: https://rustup.rs" >&2
    exit 1
  fi
}

case "${1:-install}" in
  install)
    require_cargo
    echo "▸ Сборка и установка oswam (release)…"
    cargo install --path oswam-cli --locked --force
    BIN="$(command -v oswam || echo "$HOME/.cargo/bin/oswam")"
    echo "✓ Установлено: $BIN"
    case ":$PATH:" in
      *":$HOME/.cargo/bin:"*) ;;
      *) echo "⚠ Добавь в PATH: export PATH=\"\$HOME/.cargo/bin:\$PATH\"" ;;
    esac
    echo "Запуск:  oswam   ·   oswam scan   ·   oswam clean --safe --dry-run"
    ;;
  uninstall)
    require_cargo
    cargo uninstall oswam-cli
    echo "✓ Удалено"
    ;;
  *)
    echo "Использование: ./install.sh [install|uninstall]" >&2
    exit 1
    ;;
esac
