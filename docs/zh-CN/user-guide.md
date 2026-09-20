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

每次构建都会预制完整的 6 份数据集（小学、初中、高中、CET-4、CET-6、IELTS）。USB 连接 iPhone 后执行：

```bash
pnpm tauri ios run
pnpm tauri ios build --open
```

两个命令都会重新生成并打包与 macOS 相同的完整 `content.db`。安装并打开新构建后，App 会自动把新增预制词条补入旧数据目录，保留已有学习记录和自建数据集，无需删除数据库。公开分发前需核实完整 CSV 的上游再分发授权。

## 使用

- 查词：搜索单词，点击扬声器播放发音，查看音标、释义、例句和学习统计；可直接练习或加入“我的词汇”。
- 首页/数据集：选择未学、错题或全部，再选 10/20/50 个词开始；每次题目和答案位置都会变化。答题后点击或轻触任意非操作区域即可继续，发音和例句按钮仍只播放声音；结束后可练习本次错词。
- 数据集：在 macOS 或 iPhone 上拖动右侧手柄调整顺序，也支持键盘方向键；导入/更新 CSV 时选择“仅新增 / 更新已有 / 替换数据集”；管理区可导出、重命名、删除。必填列：`lemma`、`quiz_prompt_zh`；模板：[`data/examples/dataset-template.csv`](../../data/examples/dataset-template.csv)。
- 听力：导入 UTF-8 `.txt`/`.md`，Markdown 会按文档显示；可英译中，并复制英文、中文或双语全文。宽屏左右双栏，iPhone 上下排列。翻译模型已随 App 内置，无需联网。选择声音与 0.5×/1×/1.5×/2×；选中英文单词可查词并加入“我的词汇”。
- 错题：选择筛选条件，点击“练习这个集合”。
- 设置：切换语言、外观、声音、语速，并导出/导入备份。

答题快捷键：`1`–`4` 选择，`Space` 下一题，`R` 读单词，`S` 读例句，`Esc` 退出。

## 备份

设置 → 数据 →“导出备份”；恢复时点“导入备份”。恢复前会自动备份当前数据。应用离线运行，不提供账号或云同步。

## macOS 与 iPhone 同步

进入“设置 → 设备同步”。先在 macOS 导出变更，通过隔空投送、“文件”或 iCloud Drive 把 `.polarbear-vocab-sync` 文件传到 iPhone 并导入；再在 iPhone 导出变更并回到 macOS 导入，这样才完成一次双向合并。任一设备新增内容后，重复这两个方向即可。

同步会合并自建数据集、文章及翻译、“我的词汇”和答题历史。预制数据集已随两端 App 提供，不重复传输。重复导入同一个包不会重复写入；两端同时修改同一数据集或文章时，会保留带“冲突”标记的副本。每次导入前还会生成可恢复版本。“备份/恢复”会整体替换数据，只用于灾难恢复，不用于设备同步。

## 启动失败

- 缺少 pnpm：执行上面的 Corepack 命令。
- Rust/macOS 链接错误：执行 `xcode-select --install`。
- 端口 1420 被占用：停止旧的 Vite/Tauri 进程后重试。
- iPhone 导入：通过系统文稿选择器选择 CSV/TXT/Markdown；“我的 iPhone”中的文件会自动复制进 App 沙盒。
