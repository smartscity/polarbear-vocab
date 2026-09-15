# Polarbear Vocab 用户手册

## 运行

macOS 需要 Node.js 20.19+、pnpm 11、Rust 1.85+ 和 Xcode Command Line Tools。
在项目根目录执行：

```bash
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

- 查词：搜索单词，查看音标、释义、例句和学习统计；可直接练习或加入“我的词汇”。
- 首页/数据集：选择未学、错题或全部，再选 10/20/50 个词开始；结束后可练习本次错词。
- 数据集：导入/更新 CSV 时选择“仅新增 / 更新已有 / 替换数据集”；管理区可导出、重命名、删除。必填列：`lemma`、`quiz_prompt_zh`；模板：[`data/examples/dataset-template.csv`](../../data/examples/dataset-template.csv)。
- 听力：导入 UTF-8 `.txt`/`.md`，选择声音与 0.5×/1×/1.5×/2×；选中文章单词即可查词并加入“我的词汇”。
- 错题：选择筛选条件，点击“练习这个集合”。
- 设置：切换语言、外观、声音、语速，并导出/导入备份。

答题快捷键：`1`–`4` 选择，`Space` 下一题，`R` 读单词，`S` 读例句，`Esc` 退出。

## 备份

设置 → 数据 →“导出备份”；恢复时点“导入备份”。恢复前会自动备份当前数据。应用离线运行，不提供账号或云同步。

## 启动失败

- 缺少 pnpm：执行上面的 Corepack 命令。
- Rust/macOS 链接错误：执行 `xcode-select --install`。
- 端口 1420 被占用：停止旧的 Vite/Tauri 进程后重试。
