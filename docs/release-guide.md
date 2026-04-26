# FastClaw 发版指南

## 一、前置准备（一次性配置）

### 1.1 签名密钥

更新系统使用 minisign 密钥对来验证更新包的完整性。密钥已生成并存放在：

| 文件 | 路径 | 用途 |
|------|------|------|
| 私钥 | `~/.tauri/fastclaw.key` | 构建时签名，**不可泄露** |
| 公钥 | `~/.tauri/fastclaw.key.pub` | 内嵌到 `tauri.conf.json`，客户端验签 |

> 如需重新生成密钥：
> ```bash
> npx @tauri-apps/cli@latest signer generate --write-keys ~/.tauri/fastclaw.key --force -p ""
> ```
> 重新生成后需更新 `tauri.conf.json` 中的 `plugins.updater.pubkey`。

### 1.2 GitHub Secrets 配置

在仓库 Settings -> Secrets and variables -> Actions 中配置：

| Secret 名称 | 值 | 说明 |
|-------------|------|------|
| `TAURI_SIGNING_PRIVATE_KEY` | `~/.tauri/fastclaw.key` 的完整内容 | 构建签名用 |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 私钥密码（如无密码留空字符串） | 解密私钥 |

```bash
# 获取私钥内容（复制到 GitHub Secret）
cat ~/.tauri/fastclaw.key
```

### 1.3 更新端点配置

`tauri.conf.json` 中已配置：

```json
{
  "plugins": {
    "updater": {
      "pubkey": "<公钥内容>",
      "endpoints": [
        "https://github.com/<owner>/FastClaw/releases/latest/download/latest.json"
      ],
      "windows": {
        "installMode": "passive"
      }
    }
  }
}
```

> **注意**：`endpoints` 中的 URL 需要替换 `<owner>` 为实际的 GitHub 用户名/组织名。

---

## 二、发版流程

### 2.1 版本号规范

遵循 [Semantic Versioning](https://semver.org/)：

- **patch** (0.0.x)：Bug 修复、小优化
- **minor** (0.x.0)：新功能、非破坏性改动
- **major** (x.0.0)：破坏性 API 变更

### 2.2 发版步骤

#### Step 1: 更新版本号

同步修改以下位置的版本号：

```bash
# 1. tauri.conf.json
# 修改 "version": "x.y.z"
```

涉及文件：
- `crates/fastclaw-app/src-tauri/tauri.conf.json` → `version` 字段
- `crates/fastclaw-app/package.json` → `version` 字段（可选，保持一致）
- `Cargo.toml`（workspace）→ 如有全局 version 字段

#### Step 2: 提交并推送

```bash
git add -A
git commit -m "chore: bump version to v0.1.0"
git push origin main
```

#### Step 3: 打 Tag 触发构建

```bash
git tag v0.1.0
git push origin v0.1.0
```

推送 tag 后，GitHub Actions `release.yml` 会自动：
1. 在 Linux / Windows / macOS 三平台并行构建
2. 使用 `TAURI_SIGNING_PRIVATE_KEY` 对产物签名
3. 生成 `latest.json`（包含各平台下载 URL + 签名）
4. 创建 GitHub Release 并上传所有产物

#### Step 4: 验证 Release

检查 GitHub Releases 页面，确认以下文件已上传：

| 平台 | 安装包 | 更新包 | 签名 |
|------|--------|--------|------|
| Linux | `FastClaw_x.y.z_amd64.deb` | `FastClaw_x.y.z_amd64.AppImage.tar.gz` | `.sig` |
| Windows | `FastClaw_x.y.z_x64-setup.exe` | `FastClaw_x.y.z_x64-setup.nsis.zip` | `.sig` |
| macOS | `FastClaw_x.y.z_universal.dmg` | `FastClaw_x.y.z_universal.app.tar.gz` | `.sig` |
| 元数据 | — | `latest.json` | — |

#### Step 5: 验证 latest.json

```bash
curl -sL https://github.com/<owner>/FastClaw/releases/latest/download/latest.json | jq .
```

预期输出：

```json
{
  "version": "0.1.0",
  "notes": "FastClaw v0.1.0",
  "pub_date": "2026-04-26T12:00:00Z",
  "platforms": {
    "linux-x86_64": {
      "url": "https://github.com/.../FastClaw_0.1.0_amd64.AppImage.tar.gz",
      "signature": "..."
    },
    "windows-x86_64": {
      "url": "https://github.com/.../FastClaw_0.1.0_x64-setup.nsis.zip",
      "signature": "..."
    },
    "darwin-universal": {
      "url": "https://github.com/.../FastClaw_0.1.0_universal.app.tar.gz",
      "signature": "..."
    }
  }
}
```

---

## 三、客户端更新机制

### 3.1 自动检测

应用启动后 5 秒自动检查更新，之后每 1 小时轮询一次。检测流程：

1. 前端调用 `@tauri-apps/plugin-updater` 的 `check()` 方法
2. 插件请求 `endpoints` 中的 `latest.json`
3. 比较本地版本与远程版本
4. 若有新版本，展示更新横幅

### 3.2 更新策略

| 平台 | 安装模式 | 用户体验 |
|------|----------|----------|
| Windows | `passive`（静默安装） | 下载后自动安装，无需用户确认弹窗 |
| Linux | AppImage 替换 | 下载后替换，重启生效 |
| macOS | .app 替换 | 下载后替换，重启生效 |

### 3.3 UI 行为

| 组件 | 位置 | 行为 |
|------|------|------|
| `UpdateBanner` | 主窗口 TitleBar 下方 | 有更新时自动显示，可关闭 |
| `AboutTab` 更新区 | 设置 -> 关于 | 手动点击「检查更新」按钮 |

状态流转：`idle` → `checking` → `available` → `downloading` (带进度) → `ready` → 重启

---

## 四、发版前 Checklist

### 代码准备

- [ ] 所有 PR 已合并到 main
- [ ] `cargo clippy --workspace -- -D warnings` 无警告
- [ ] `cargo test --workspace` 全部通过
- [ ] `pnpm build` 前端构建成功
- [ ] `pnpm test` 前端测试通过

### 版本号

- [ ] `tauri.conf.json` 中 `version` 已更新
- [ ] Tag 名称格式正确（`v0.1.0`）

### Secrets

- [ ] `TAURI_SIGNING_PRIVATE_KEY` 已配置在 GitHub Secrets
- [ ] `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` 已配置（无密码也要设为空字符串）

### 配置

- [ ] `tauri.conf.json` 中 `plugins.updater.endpoints` URL 正确
- [ ] `tauri.conf.json` 中 `plugins.updater.pubkey` 与密钥对匹配
- [ ] `bundle.createUpdaterArtifacts` 为 `true`
- [ ] `capabilities/default.json` 包含 `updater:default`、`process:allow-restart`

### 构建产物验证

- [ ] GitHub Actions 三平台构建全部成功
- [ ] Release 页面产物完整（安装包 + 更新包 + .sig + latest.json）
- [ ] `latest.json` 格式正确，platforms 包含所有目标平台
- [ ] 签名文件 (.sig) 与对应包匹配

### 更新功能验证

- [ ] 旧版本客户端能检测到新版本
- [ ] 下载进度正常显示
- [ ] 下载完成后「立即重启」能正常重启并应用更新
- [ ] Windows 静默安装无弹窗
- [ ] 更新后版本号正确显示

---

## 五、本地构建发布（无 CI 环境）

由于无法直接使用 GitHub CI，需要在 Linux 和 Windows 机器上分别构建，然后合并产物发布。

### 5.1 脚本清单

| 脚本 | 运行环境 | 路径 | 用途 |
|------|---------|------|------|
| `build-linux.sh` | Linux 构建机 | `scripts/build-linux.sh` | 构建 + 签名 |
| `build-windows.ps1` | Windows 构建机 | `scripts/build-windows.ps1` | 构建 + 签名 |
| `publish-release.sh` | 任意能访问 GitHub 的机器 | `scripts/publish-release.sh` | 发布 + 生成 latest.json |
| `merge-latest-json.sh` | 任意 | `scripts/merge-latest-json.sh` | 离线合并 latest.json（可选） |

### 5.2 推荐流程（GitHub Releases，两台独立机器）

```
Linux 构建机                     Windows 构建机
    │                                │
    ▼                                ▼
build-linux.sh                 build-windows.ps1
    │                                │
    ▼                                ▼
  dist/                            dist\
  ├─ .AppImage                     ├─ .exe
  ├─ .AppImage.tar.gz              ├─ .nsis.zip
  ├─ .AppImage.tar.gz.sig          ├─ .nsis.zip.sig
  └─ .deb                          └─ (latest.json 可忽略)
    │                                │
    │   gh release upload            │   gh release upload
    └────────────┬───────────────────┘
                 ▼
          GitHub Release (v0.1.0)
                 │
                 ▼
    任意机器: publish-release.sh v0.1.0
    (自动从 Release 下载 .sig, 生成并上传 latest.json)
```

#### Step 1: 更新版本号

修改 `crates/fastclaw-app/src-tauri/tauri.conf.json` 中的 `"version"` 字段，提交到 main。

#### Step 2: 在 Linux 机器上构建

```bash
git pull origin main
./scripts/build-linux.sh
# 产物输出到 ./dist/
```

#### Step 3: 在 Windows 机器上构建

```powershell
git pull origin main
.\scripts\build-windows.ps1
# 产物输出到 .\dist\
```

#### Step 4: 上传产物到 GitHub Release

**在 Linux 机器上：**
```bash
# 先创建 Release（只需在一台机器上做一次）
gh release create v0.1.0 --title "FastClaw v0.1.0" --generate-notes

# 上传 Linux 产物
gh release upload v0.1.0 dist/*.AppImage dist/*.AppImage.tar.gz dist/*.AppImage.tar.gz.sig dist/*.deb
```

**在 Windows 机器上：**
```powershell
# 上传 Windows 产物（Release 已存在，直接追加文件）
gh release upload v0.1.0 dist\*.exe dist\*.nsis.zip dist\*.nsis.zip.sig
```

> 也可通过 GitHub 网页直接拖拽上传文件到 Release 中。

#### Step 5: 生成并上传 latest.json

在**任意**一台能访问 GitHub 的机器上（Linux/macOS/WSL 均可）：

```bash
./scripts/publish-release.sh v0.1.0

# 脚本会自动:
# 1. 从 GitHub Release 获取文件列表
# 2. 下载所有 .sig 签名文件
# 3. 生成 latest.json（包含所有已上传平台的 URL + 签名）
# 4. 上传 latest.json 到同一个 Release
```

#### Step 6: 验证

```bash
curl -sL https://github.com/<owner>/FastClaw/releases/latest/download/latest.json | python3 -m json.tool
```

### 5.3 替代方案：自建文件服务器

如果不使用 GitHub Releases，可以：

1. 两台机器分别构建（同上 Step 2, 3）
2. 将两台机器的 `dist/` 传到同一台服务器
3. 使用 `merge-latest-json.sh` 合并：
   ```bash
   ./scripts/merge-latest-json.sh ./dist-linux ./dist-windows -o ./release
   ```
4. 编辑 `release/latest.json`，将 `REPLACE_WITH_DOWNLOAD_URL` 改为实际 URL
5. 将所有文件放到 Web 服务器上
6. 确保 `tauri.conf.json` 中 `endpoints` 指向 `latest.json` 的 URL

### 5.3 构建脚本参数说明

**Linux (`build-linux.sh`)**

| 参数 | 说明 |
|------|------|
| `--release` | 构建 + 生成 latest.json |
| `--skip-lint` | 跳过 clippy 检查（加速构建） |

**Windows (`build-windows.ps1`)**

| 参数 | 说明 |
|------|------|
| `-Release` | 构建 + 生成 latest.json |
| `-SkipLint` | 跳过 clippy 检查 |

### 5.4 签名密钥管理

签名密钥 `~/.tauri/fastclaw.key` 需要在所有构建机器上保持一致。

```bash
# 在源机器上导出
cat ~/.tauri/fastclaw.key | base64 > key-backup.b64

# 在目标机器上恢复
mkdir -p ~/.tauri
base64 -d key-backup.b64 > ~/.tauri/fastclaw.key
```

> 密钥传输完成后务必删除 `key-backup.b64`。

---

## 六、故障排查

### 客户端检测不到更新

1. 确认 `latest.json` 可访问：`curl -sL <endpoint-url>`
2. 确认 `latest.json` 中的 `version` 大于当前版本
3. 确认 `platforms` 中包含当前平台的 key

### 签名验证失败

1. 确认构建时使用的私钥与 `tauri.conf.json` 中的公钥是同一对
2. 确认 `.sig` 文件内容完整未被截断
3. 重新生成密钥对并更新所有配置

### Windows 更新安装失败

1. 检查 `installMode` 是否设为 `passive`（静默）或 `basicUi`（显示进度）
2. 确认 NSIS 安装包未被杀毒软件拦截
3. 可选模式：`quiet`（完全静默）、`passive`（显示进度条）、`basicUi`（带取消按钮）

### 更新下载超时

1. GitHub Releases 的下载 URL 可能需要翻墙
2. 可配置自建 CDN 或镜像作为备用 endpoint
3. `endpoints` 支持多个 URL，依次尝试
