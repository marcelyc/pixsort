# Pixsort

Rust 桌面图片分类工具，支持 JPG、PNG、BMP。

安装 Rust 后执行 `cargo run --release`。选择待整理文件夹，在右侧为数字 1–9 设置目标文件夹：可以点“选择”，也可以直接输入已存在文件夹的绝对路径，然后按 Enter 或点“保存”。左右方向键浏览图片；按数字键移动当前图片；按 Ctrl+Z（macOS 为 Command+Z）撤销。配置自动保存。

同名图片会自动追加编号，不覆盖已有文件。目标文件夹计数仅包含顶层图片。

## Windows 打包

在安装了 Rust 和 Windows SDK 的 Windows 电脑上执行 `cargo build --release`，生成的程序为 `target\release\pixsort.exe`。EXE 的“属性 → 详细信息”包含版本、产品名称、文件说明、版权和原始文件名。版本取自 `Cargo.toml` 的 `[package]`，其他字段可在 `[package.metadata.winresource]` 中修改。文件大小和修改日期由构建结果决定。
