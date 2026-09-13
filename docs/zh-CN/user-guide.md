# Polarbear Vocab 用户手册

## 在 macOS 上从源码运行

请先安装：

- Node.js 20.19 或更高版本。
- pnpm 11。
- 通过 `rustup` 安装 Rust 1.85 或更高版本。
- Xcode Command Line Tools（运行 `xcode-select --install`）。

打开终端，在项目根目录中执行：

```bash
cd /Users/liyunlong/git/smartscity/VocabProbe
pnpm install
pnpm tauri dev
```

`pnpm install` 会安装锁定版本的 JavaScript 依赖。`pnpm tauri dev` 会重建预制的 Polarbear Lexicon 数据库，启动 Vite，编译 Rust 应用，然后打开 Tauri 桌面窗口。首次编译 Rust 可能需要几分钟。

如果系统没有 `pnpm`，可以启用 Corepack 并安装项目锁定的 pnpm 版本：

```bash
corepack enable
corepack prepare pnpm@11.19.0 --activate
```

## 构建可安装应用

执行：

```bash
pnpm tauri build
```

macOS 应用包和安装包会输出到 `target/release/bundle/`。如果只需重建词库和 Web 界面，不打包桌面外壳，可以执行 `pnpm run build`。

## 首次使用

应用打开后会进入首页，并选中一个预制数据集。首页会显示已探索词汇、正确集、错题集、最近 30 天活动和错题快捷入口。可在首页顶部切换数据集，也可从导航进入“数据集”。

学习未做过的内容时，点击“继续”。复习历史内容时，可点击已探索、正确或错误统计，也可打开不同错题阈值。每道题显示一个中文题干和四个英文选项。

答题页快捷键：

| 按键 | 操作 |
| --- | --- |
| `1` / `2` / `3` / `4` | 选择答案。 |
| `Space` | 答题后进入下一题。 |
| `R` | 答题后朗读正确单词。 |
| `S` | 答题后朗读例句。 |
| `Esc` | 结束当前会话并返回首页。 |

语音由 macOS 系统语音在本地实时生成，不会调用云端语音服务。

## 创建和导入数据集

1. 打开“数据集”。
2. 输入任意数据集名称，点击“创建数据集”。
3. 选中新数据集，点击“导入 CSV”。
4. 选择本地 `.csv` 文件。
5. 检查样例数据和校验结果。
6. 修复所有错误后，点击“导入”。

只有 `lemma` 和 `quiz_prompt_zh` 两列必填。完整格式说明见 [数据集 CSV 格式](dataset-csv.md)，可直接使用 [`data/examples/dataset-template.csv`](../../data/examples/dataset-template.csv) 作为模板。导入是原子操作；如果失败，整次导入都会回滚。

数据集详情页还可以开始练习、重命名或删除数据集。删除数据集会删除其成员列表，但会保留已记录的答题历史。

## 错题和统计

打开“错题”后，可按全部错题、错误次数阈值或最近一次答题结果进行筛选。点击“练习这个集合”会使用当前可见集合开始测验。后续答对不会抹掉之前的错题记录。

首页统计只描述已经发生的学习活动。Polarbear Vocab 没有到期时间、每日目标、连续天数、间隔重复计划或学习债务。

## 切换界面语言和外观

打开“设置”，在“语言”中选择跟随系统、English 或简体中文。界面会立即切换，选择会保存到 `user.db`。跟随系统时，macOS 使用受支持的中文语言区域就显示简体中文，其他情况回退到英文。切换界面语言不会翻译数据集内容。

外观可选择跟随系统、浅色或深色，切换会立即生效，并与语言设置一起保存。导航会随窗口宽度调整：宽窗口使用左侧栏，中等窗口使用顶部栏，紧凑窗口使用底部导航；学习时会隐藏全局导航，保持题目区域稳定。

## 本地数据

macOS 上的可写数据位于：

```text
~/Library/Application Support/com.polarbear.vocab/
```

- `content.db` 保存预制数据集、导入数据集、词义、发音和例句。
- `user.db` 保存答题历史、派生统计、学习会话和设置。

手动移动或替换应用数据前，请同时备份这两个文件。正常使用全程在本地完成，应用不提供远程目录、云账号或云同步。

## 常见启动问题

- **`pnpm: command not found`**：按上文命令启用 Corepack，或安装 pnpm 11。
- **`node: command not found`**：安装 Node.js 20.19 或更高版本，然后重新打开终端。
- **Rust 链接或 macOS framework 错误**：执行 `xcode-select --install`，安装完成后重试。
- **1420 端口已占用**：停止先前的 Vite 或 Tauri 开发进程，然后重新执行 `pnpm tauri dev`。
- **首次启动较慢**：首次开发运行需要编译 Rust 和 Tauri 依赖，后续启动会复用编译缓存。
