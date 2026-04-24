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

use stm32l4xx_hal::pac::NVIC;
// STM32L476RG専用ライブラリ
// pacとは Peripheral Access Crateのことで簡単に言えばそのマイコンにアクセスするライブラリを指す
// prelude::*でライブラリのうち汎用的な関数（例えばconstrain）を呼び出している
use stm32l4xx_hal::pac;
use stm32l4xx_hal::gpio::{Pin, Alternate, PushPull, H8};
use stm32l4xx_hal::pac::interrupt;
use stm32l4xx_hal::{can::Can, prelude::*, serial::{Config, Serial}};
use core::cell::RefCell;
use core::ops::Deref;
use cortex_m::interrupt::Mutex;
use bxcan::{self, Can as BxCan, StandardId};
use core::fmt::Write; // ← これ重要

use l476_rs::CanData;
static CAN_DATA: CanData = CanData::new();

static CAN: Mutex<RefCell<Option<BxCan<Can<pac::CAN1, (Pin<Alternate<PushPull, 9>, H8, 'B', 9>, Pin<Alternate<PushPull, 9>, H8, 'B', 8>)>>>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // stm32L476RG本体みたいな構造体変数
    let peripheral = pac::Peripherals::take().unwrap();


    // Reset and Clock Controlの略。ここに各タイマーレジスタが含まれているやバスが含まれている
    // peripheralに含まれているRCCをconstrain関数によって使いやすい構造体に変換
    let mut rcc = peripheral.RCC.constrain();

    let mut pwr = peripheral.PWR.constrain(&mut rcc.apb1r1);

    let mut flash = peripheral.FLASH.constrain();

    let clocks = rcc.cfgr.sysclk(80.MHz()).pclk1(80.MHz()).freeze(&mut flash.acr, &mut pwr);

    // GPIOのAポートを取得する。
    // マイコンのシステムにはバスと呼ばれる、CPUと各周辺機能をつなぐものがある。
    // ahb2というのはCPUとGPIOをつなぐバスです。
    // split関数でperipheralの中のGPIOのAポートのみ分離して構造体変数として取得
    let mut gpio_a = peripheral.GPIOA.split(&mut rcc.ahb2); 
    let mut gpio_b = peripheral.GPIOB.split(&mut rcc.ahb2);

    let rx = gpio_b.pb8.into_alternate::<9>(&mut gpio_b.moder, &mut gpio_b.otyper, &mut gpio_b.afrh);
    let tx = gpio_b.pb9.into_alternate::<9>(&mut gpio_b.moder, &mut gpio_b.otyper, &mut gpio_b.afrh);

    // Use HAL's CAN wrapper directly instead of calling bxcan::Can::builder
    let can_dp = Can::new(&mut rcc.apb1r1, peripheral.CAN1, (tx, rx));

    // bxcan::Can::builder関数でCANの構造体を作成
    let mut can_ = bxcan::Can::builder(can_dp).set_bit_timing(0x0001_0013).set_loopback(false).enable();

    // すべてのIDを受信するフィルタを設定
    can_.modify_filters().enable_bank(0, bxcan::filter::Mask32::accept_all());
    // 受信割り込み有効化
    can_.enable_interrupt(bxcan::Interrupt::Fifo0MessagePending);

    // NVIC有効化
    unsafe {
        NVIC::unmask(pac::Interrupt::CAN1_RX0);
    }

    // USART2のTXとRXのピンを設定
    let serial_tx = gpio_a.pa2.into_alternate::<7>(&mut gpio_a.moder, &mut gpio_a.otyper, &mut gpio_a.afrl);
    let serial_rx = gpio_a.pa3.into_alternate::<7>(&mut gpio_a.moder, &mut gpio_a.otyper, &mut gpio_a.afrl);
    // USART2の構造体を作成
    let mut serial = Serial::usart2(peripheral.USART2, (serial_tx, serial_rx), Config::default(), clocks, &mut rcc.apb1r1);
    

    // gpio_aが含んでいるpa5にアクセスする
    // pa5はSTM32L476RGの内蔵LEDにつながっているピン
    // 「　let led = gpio_a.pa5　」だとただピンを取得しただけになる
    // なのでinto_push_pull_output関数で出力モードに切り替える
    // gpio_a.moderはピンのモードを設定するレジスタにアクセスする変数
    // gpio_a.otyperはピンの出力タイプを設定するレジスタにアクセスする変数
    // なのでこの２つの変数を可変のポインタとして（&mutをつけている理由）引数に渡している
    let mut internal_led = gpio_a.pa5.into_push_pull_output(&mut gpio_a.moder, &mut gpio_a.otyper);

    internal_led.set_high();
    asm::delay(5000000);
    internal_led.set_low();
    asm::delay(5000000);
    internal_led.set_high();
    asm::delay(5000000);
    internal_led.set_low();
    asm::delay(5000000);

    cortex_m::interrupt::free(|cs| {
        CAN.borrow(cs).replace(Some(can_));
    });

    loop 
    {
        let mut read_buffer = [0_u8; 8];
        let mut count = 0;
        // シリアル通信で受信
        loop {
            match serial.read()
            {
                Ok(b)=>{
                    if b == b'\n'
                    {
                        internal_led.set_high();
                        break;
                    }

                    if count < 8
                    {
                        read_buffer[count] = b;
                        count += 1;
                    }
                    else {
                        internal_led.set_high();
                        asm::delay(5000000);
                        internal_led.set_low();
                        asm::delay(5000000);

                        count = 0;
                    }
                }
                Err(_e) => {
                }
            }
        }

        let mut current = [0_i16; 6];

        for i in 0..3
        {
            current[i] = to_current(read_buffer[i]);
        }

        // for i in 3..6
        // {
        //     current[i] = to_current(read_buffer[i]) * -1;
        // }

        let mut data:[u8;8] = [0_u8;8];
        data[0] = ((current[0] >> 8) & 0xFF) as u8;
        data[1] = (current[0] & 0xFF) as u8;
        data[2] = ((current[1] >> 8) & 0xFF) as u8;
        data[3] = (current[1] & 0xFF) as u8;
        data[4] = ((current[2] >> 8) & 0xFF) as u8;
        data[5] = (current[2] & 0xFF) as u8;

        let frame = bxcan::Frame::new_data(bxcan::Id::Standard(StandardId::new(0x200).unwrap()), data);

        cortex_m::interrupt::free(|cs| {
            if let Some(can) = CAN.borrow(cs).borrow_mut().as_mut() 
            {
                if can.transmit(&frame).is_ok() 
                {
                    let mut encode_data = [0_i16; 9];
                    encode_data[0] = CAN_DATA.position[0].load(core::sync::atomic::Ordering::Relaxed);
                    encode_data[1] = CAN_DATA.velocity[0].load(core::sync::atomic::Ordering::Relaxed);
                    encode_data[2] = CAN_DATA.current[0].load(core::sync::atomic::Ordering::Relaxed);
                    encode_data[3] = CAN_DATA.position[1].load(core::sync::atomic::Ordering::Relaxed);
                    encode_data[4] = CAN_DATA.velocity[1].load(core::sync::atomic::Ordering::Relaxed);
                    encode_data[5] = CAN_DATA.current[1].load(core::sync::atomic::Ordering::Relaxed);
                    encode_data[6] = CAN_DATA.position[2].load(core::sync::atomic::Ordering::Relaxed);
                    encode_data[7] = CAN_DATA.velocity[2].load(core::sync::atomic::Ordering::Relaxed);
                    encode_data[8] = CAN_DATA.current[2].load(core::sync::atomic::Ordering::Relaxed);

                    let mut tx_buf = [0_u8; 18];
                    for i in 0..9
                    {
                        tx_buf[2*i] = (encode_data[i] >> 8) as u8 & 0xFF;
                        tx_buf[2*i+1] = (encode_data[i] & 0xFF) as u8;
                    }

                    for b in tx_buf.iter() {
                        write!(serial, "{} ", b).unwrap();
                    }
                    write!(serial, "\r\n").unwrap();
                    nb::block!(serial.flush()).unwrap();
                }
            }
        });

        // asm::delay(1000000);        
    }
}

#[interrupt]
fn CAN1_RX0() {
    cortex_m::interrupt::free(|cs| {
        if let Some(can) = CAN.borrow(cs).borrow_mut().as_mut() 
        {
            if let Ok(frame) = can.receive() 
            {
                match frame.id() {
                    bxcan::Id::Standard(id) => {
                        let id_raw = id.as_raw() - 0x200;
                        let data = frame.data().unwrap().deref();
                        // ここでencoder更新
                        
                        let angle_data = (data[0] as i16) << 8 | (data[1] as i16);
                        let rpm_data = (data[2] as i16) << 8 | (data[3] as i16);
                        let ampare_data = (data[4] as i16) << 8 | (data[5] as i16);

                        CAN_DATA.position[id_raw as usize -1].store(angle_data, core::sync::atomic::Ordering::Relaxed);
                        CAN_DATA.velocity[id_raw as usize -1].store(rpm_data, core::sync::atomic::Ordering::Relaxed);
                        CAN_DATA.current[id_raw as usize -1].store(ampare_data, core::sync::atomic::Ordering::Relaxed);
                    }
                    _ => {
                        // 処理しないIDの場合はここに来る
                    }
                }
            }
        }
    });
}

fn to_current(b: u8) -> i16 {
    ((b as f32 - 128.0) / 127.0 * 10000.0) as i16
}