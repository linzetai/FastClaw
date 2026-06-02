## Overview

Global floating mini-prompt bar (Spotlight/Raycast style) for quick assistant interaction.

## Requirements

- Activated by configurable global shortcut (default: Ctrl+Shift+L)
- Independent frameless, always-on-top, transparent Tauri window
- Auto-focused text input on activation
- Enter sends input → creates new or appends to current conversation (via `chat.send` WebSocket message to Gateway)
- Esc or blur hides the window
- Smooth show/hide animation (<100ms)
- Works when main window is hidden or closed
- Pre-created hidden window for instant activation
- After sending, the main window is shown and focused on the active conversation
- Empty input is ignored (no message sent, bar remains open)
