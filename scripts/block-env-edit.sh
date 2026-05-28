#!/usr/bin/env bash
# PreToolUse hook — block Edit/Write tới .env* files (except .env.example).
# Đầu vào: Claude Code hook spec gửi JSON qua stdin với { "tool_input": { "file_path": "..." } }.
# Fallback: $CLAUDE_TOOL_INPUT env var nếu stdin trống.
# Exit 2 → block tool call. Exit 0 → allow.
set -euo pipefail

# Đọc input
if [ ! -t 0 ]; then
  INPUT=$(cat || echo "")
else
  INPUT="${CLAUDE_TOOL_INPUT:-}"
fi

# Không có input → pass through (hook không có context để check)
if [ -z "$INPUT" ]; then exit 0; fi

# Parse file_path từ JSON
FILE_PATH=$(echo "$INPUT" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    print(data.get('tool_input', {}).get('file_path', ''))
except Exception:
    print('')
" 2>/dev/null || echo "")

# Không có file_path → tool khác (không phải Edit/Write file) → pass
if [ -z "$FILE_PATH" ]; then exit 0; fi

# Basename để check pattern
BASE=$(basename "$FILE_PATH")

# Allowlist: .env.example là template, được phép edit
if [ "$BASE" = ".env.example" ]; then exit 0; fi

# Block .env và .env.* (production, local, staging, etc.)
if echo "$BASE" | grep -qE '^\.env($|\.)'; then
  cat >&2 <<EOF
⛔ BLOCKED: Edit/Write tới $FILE_PATH bị chặn.

Lý do: Project có ALE encryption + secrets production (OpenRouter / PayOS / Telegram bot / Sentry).
KHÔNG sửa .env* trực tiếp qua Claude.

Cách hợp lệ:
  - Sửa .env.example (template, có docs)
  - Nhờ Sếp paste secret thật vào .env tay
  - Sửa qua VPS SSH nếu là production env
EOF
  exit 2
fi

# Mọi file khác → allow
exit 0
