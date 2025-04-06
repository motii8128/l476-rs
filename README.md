# l476-rs
This is a template repository for developing with the STM32L476RG using Rust.

![l476rg](https://github.com/user-attachments/assets/00770983-df8b-44d4-a8e2-0d50324add78)

# Usage
## Install
まず必要なツールをインストールします。Rustはもう入っている前提です
#### デバッガと基本ライブラリ
```
sudo apt install openocd binutils-arm-none-eabi gdb-multiarch
```
#### Rust関連ツール
STM32L476RGを含むSTM32CPUのRustライブラリ
```
rustup target add thumbv7em-none-eabihf
```
Rust製のファームウェアやバイナリを解析・検査・デバッグしやすくするためのツールセット
```
cargo install cargo-binutils
```
解析ツール本体
```
rustup component add llvm-tools-preview
```



## Build
リポジトリの取得
```
git clone https://github.com/motii8128/l476-rs.git
```
リポジトリフォルダに移動する
```
cd l476-rs
```
ビルド（コンパイル）する
```
cargo build --release
```
