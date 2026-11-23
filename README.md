# Pixsort

Rust 桌面图片分类工具，支持 JPG、PNG、BMP。

安装 Rust 后执行 `cargo run --release`。选择待整理文件夹，在右侧为数字 1–9 设置目标文件夹。左右方向键浏览图片；按数字键移动当前图片；按 Ctrl+Z（macOS 为 Command+Z）撤销。配置自动保存。

同名图片会自动追加编号，不覆盖已有文件。目标文件夹计数仅包含顶层图片。
