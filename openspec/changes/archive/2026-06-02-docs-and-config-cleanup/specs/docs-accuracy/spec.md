## Overview

Documentation accuracy requirements for XiaoLin project files.

## Requirements

- README.md must only reference binaries that exist in the workspace
- MANUAL.md config examples must match actual `XiaoLinConfig` struct fields
- MANUAL.md WebSocket protocol must match actual `xiaolin-ws/2` implementation
- config/default.json must not contain developer-specific paths
- Version numbers must be consistent across all files
- Docker/K8s configurations must build and deploy successfully, or be clearly marked as WIP
