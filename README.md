# l476-rs
This is a template repository for developing with the STM32L476RG using Rust.

![ss](https://github.com/user-attachments/assets/b04effe1-3a6f-4d57-81a5-ec35e3c9a9c8)

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
ビルドする
```
cargo build --release
```
これでマイコンに書き込むバイナリが生成されているので確認する。

ここでは何も変更していなければ以下のフォルダ内に「l476-rs」という名前で生成される。

ここで「thumbv7em-none-eabihf」とは今回のターゲットCPU（Cortex-M4F）に対するライブラリ名である
```
target/thumbv7em-none-eabihf/release
```

スクリプト（必要な操作をすべてまとめたもの）に権限を付与する
```
chmod +x ./load.sh
```
スクリプトを実行してマイコンにコードを書き込む
スクリプトの引数としてバイナリの場所を入力する必要がある
```
./load.sh [バイナリの場所]
```
今回だとこんな感じ
```
./load.sh target/thumbv7em-none-eabihf/release/l476-rs
```
