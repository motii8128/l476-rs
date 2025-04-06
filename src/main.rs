// マイコン上で動かすため、stdライブラリを使わない宣言
#![no_std]
// メイン関数がないことを宣言
#![no_main]

// エラーハンドリングを勝手にやってくれる魔法
use panic_halt as _;

// Delay関数など汎用が詰まってる
use cortex_m::asm;

// メイン関数がないためプログラムの始まりを示すエントリーポイントを作る必要あり
use cortex_m_rt::entry;

// STM32L476RG専用ライブラリ
// pacとは Peripheral Access Crateのことで簡単に言えばそのマイコンにアクセスするライブラリを指す
// prelude::*でライブラリのうち汎用的な関数（例えばconstrain）を呼び出している
use stm32l4xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    // stm32L476RG本体みたいな構造体変数
    let peripheral = pac::Peripherals::take().unwrap();

    // Reset and Clock Controlの略。ここに各タイマーレジスタが含まれているやバスが含まれている
    // peripheralに含まれているRCCをconstrain関数によって使いやすい構造体に変換
    let mut rcc = peripheral.RCC.constrain();

    // GPIOのAポートを取得する。
    // マイコンのシステムにはバスと呼ばれる、CPUと各周辺機能をつなぐものがある。
    // ahb2というのはCPUとGPIOをつなぐバスです。
    // split関数でperipheralの中のGPIOのAポートのみ分離して構造体変数として取得
    let mut gpio_a = peripheral.GPIOA.split(&mut rcc.ahb2); 

    // gpio_aが含んでいるpa5にアクセスする
    // pa5はSTM32L476RGの内蔵LEDにつながっているピン
    // 「　let led = gpio_a.pa5　」だとただピンを取得しただけになる
    // なのでinto_push_pull_output関数で出力モードに切り替える
    // gpio_a.moderはピンのモードを設定するレジスタにアクセスする変数
    // gpio_a.otyperはピンの出力タイプを設定するレジスタにアクセスする変数
    // なのでこの２つの変数を可変のポインタとして（&mutをつけている理由）引数に渡している
    let mut internal_led = gpio_a.pa5.into_push_pull_output(&mut gpio_a.moder, &mut gpio_a.otyper);

    loop {
        // ピンをHIGH（5V）にする
        internal_led.set_high();

        // 普通にdelayする関数。1000000μ秒なので１秒止まる
        asm::delay(1000000);

        // ピンをLOWにする
        internal_led.set_low();

        asm::delay(1000000);
    }
}
