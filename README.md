# PureVox

一款基于 Tauri 2 + Vue 3 的桌面音乐播放器，通过 Bilibili 搜索与播放音乐，支持个性化推荐、歌单管理、最近播放与主题自定义。

## 功能特性

- **Bilibili 音乐搜索** —— 输入歌曲/艺术家/歌单关键字，后台聚合候选结果
- **智能个性化推荐** —— 基于 LLM 过滤与排序，每天自动推荐一次，支持手动"换一批"
- **本地数据持久化** —— 播放历史、收藏、歌单、用户画像等以 JSON 形式保存在 `data/` 目录
- **AES-256-GCM 加密** —— LLM API Key 等敏感信息使用机器唯一 ID + 固定盐派生密钥加密
- **最近播放** —— 完整记录与一键清空/播放全部
- **歌单管理** —— 创建、编辑、删除歌单，添加/移除歌曲
- **MV 播放** —— 内置视频播放器，打开即自动播放
- **Glassmorphism 玻璃 UI** —— 支持自定义强调色、玻璃透明度、模糊半径与饱和度
- **最近播放/双击播放** —— 符合 Windows 操作习惯
- **底部加载动画** —— 网络请求期间非阻塞提示

## 技术栈

- 后端：Rust + Tauri 2
- 前端：Vue 3 + TypeScript + Vite
- 样式：Tailwind CSS
- 包管理：pnpm

## 项目结构

```
PureVox/
├── src-frontend/          # Vue 前端源码
│   ├── src/
│   │   ├── components/    # 布局/播放器/UI/视频组件
│   │   ├── stores/        # Pinia 状态管理
│   │   ├── views/         # 页面视图
│   │   ├── api/           # Bilibili / 推荐接口封装
│   │   └── styles/        # 全局样式与设计令牌
│   └── index.html
├── src-tauri/             # Rust 后端源码
│   ├── src/
│   │   ├── bili.rs        # Bilibili 搜索与解析
│   │   ├── llm.rs         # LLM 过滤推荐
│   │   ├── rank.rs        # 个性化排序
│   │   ├── storage.rs     # 本地文件读写
│   │   └── main.rs
│   └── Cargo.toml
├── scripts/               # 构建与图标生成脚本
└── README.md
```

## 开发环境

- [Node.js](https://nodejs.org/) LTS
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)

## 安装与运行

```bash
# 进入前端目录
cd src-frontend

# 安装依赖
pnpm install

# 启动开发服务器
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```

构建完成后，可执行文件位于 `src-tauri/target/release/purevox.exe`。

## 配置说明

首次运行时，程序会在可执行文件同级目录创建 `data/` 文件夹，用于存储：

- `settings.json` —— LLM Base URL、模型、API Key（加密）等
- `theme.json` —— 用户自定义主题
- `play-history.json` —— 最近播放
- `favorites.json` —— 收藏歌曲
- `playlists.json` —— 用户歌单
- `recommend-cache.json` / `user-profile.json` —— 推荐缓存与用户画像

> 注意：`data/` 目录属于运行时数据，不会被提交到 Git。

## 发布 Release（一键部署）

本项目提供一键发布脚本，自动完成构建并上传安装包到 GitHub Release。

### 前置条件

1. 已安装 [Node.js](https://nodejs.org/)、[pnpm](https://pnpm.io/)、[Rust](https://www.rust-lang.org/tools/install)
2. 已安装 [GitHub CLI](https://cli.github.com/) (`gh`) 且已登录：

```powershell
gh auth login
```

### 一键发布

在仓库根目录执行：

```powershell
.\scripts\release.ps1
```

脚本会依次完成：

1. 检查 `gh` 登录状态
2. 从 `src-tauri/Cargo.toml` 读取版本号
3. 确认发布
4. 结束正在运行的 `purevox.exe`（避免文件锁定）
5. 执行 `pnpm install` 和 `pnpm tauri build`
6. 检查构建产物
7. 创建 GitHub Release 并上传：
   - `purevox.exe` —— 绿色版可执行文件
   - `PureVox_x.x.x_x64_en-US.msi` —— Windows 安装包（MSI）
   - `PureVox_x.x.x_x64-setup.exe` —— Windows 安装包（NSIS）

发布完成后，终端会输出 Release 访问地址。

### 手动发布

如果不使用脚本，也可以在 GitHub 网页操作：

1. 本地执行 `pnpm tauri build` 完成构建
2. 打开仓库 Releases 页面 → `Draft a new release`
3. 输入 Tag（如 `v0.1.0`）和标题
4. 上传 `src-tauri/target/release/` 下的 `purevox.exe` 以及 `bundle/` 目录中的安装包
5. 点击 `Publish release`

## 许可证

[MIT](./LICENSE)
