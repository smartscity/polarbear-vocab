# Polarbear Vocab 用户手册

## 运行

macOS 需要 Node.js 20.19+、pnpm 11、Rust 1.85+ 和 Xcode Command Line Tools。

```bash
cd /Users/liyunlong/git/smartscity/VocabProbe
pnpm install
pnpm tauri dev
```

没有 pnpm 时执行：

```bash
corepack enable
corepack prepare pnpm@11.19.0 --activate
```

## 构建

```bash
pnpm tauri build
```

产物位于 `target/release/bundle/`。

## 使用

- 首页：切换数据集，查看统计，点击“继续”学习。
- 数据集：创建名称 → 选择数据集 → 导入 CSV。必填列为 `lemma`、`quiz_prompt_zh`；模板见 [`data/examples/dataset-template.csv`](../../data/examples/dataset-template.csv)。
- 错题：选择筛选条件，点击“练习这个集合”。
- 设置：切换 English / 简体中文和系统 / 浅色 / 深色外观。

答题快捷键：`1`–`4` 选择，`Space` 下一题，`R` 读单词，`S` 读例句，`Esc` 退出。

## 数据

数据位于 `~/Library/Application Support/com.polarbear.vocab/`：

- `content.db`：数据集和词汇内容。
- `user.db`：答题历史、统计、会话和设置。

应用离线运行，不提供远程导入、账号或云同步。备份时请同时复制两个数据库。

## 启动失败

- 缺少 pnpm：执行上面的 Corepack 命令。
- Rust/macOS 链接错误：执行 `xcode-select --install`。
- 端口 1420 被占用：停止旧的 Vite/Tauri 进程后重试。
