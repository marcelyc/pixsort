# Pixsort

Rust 桌面图片分类工具，支持 JPG、PNG、BMP。

安装 Rust 后执行 `cargo run --release`。选择待整理文件夹，在右侧为数字 1–9 设置目标文件夹：可以点“选择”，也可以直接输入已存在文件夹的绝对路径，然后按 Enter 或点“保存”。左右方向键浏览图片；按数字键移动当前图片；按 Ctrl+Z（macOS 为 Command+Z）撤销。配置自动保存。

同名图片会自动追加编号，不覆盖已有文件。目标文件夹计数仅包含顶层图片。

## Windows 打包

在安装了 Rust 和 Windows SDK 的 Windows 电脑上执行 `cargo build --release`，生成的程序为 `target\release\arp.exe`。EXE 的“属性 → 详细信息”按示例显示 arp、1.0.0.0、英语（美国）和 arp.exe。文件大小和修改日期由构建结果决定。

Windows 构建使用图形界面子系统，直接双击 `arp.exe` 不会弹出终端窗口。若通过 `cargo run` 启动，运行 Cargo 的终端仍会保持打开，直到程序退出。

应用图标位于 `assets/icon.png`，Windows EXE 图标使用多尺寸的 `assets/icon.ico`。重新执行 `cargo build --release` 后，新生成的 EXE 会包含该图标。
