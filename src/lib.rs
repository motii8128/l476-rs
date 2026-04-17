// マイコン上で動かすため、stdライブラリを使わない宣言
#![no_std]
// メイン関数がないことを宣言
#![no_main]

use core::sync::atomic::{AtomicI16, Ordering};

pub struct CanData
{
    pub position : [AtomicI16; 3],
    pub velocity : [AtomicI16; 3],
    pub current : [AtomicI16; 3],
}

impl CanData
{
    pub const fn new()->Self
    {
        let pos = [AtomicI16::new(-1), AtomicI16::new(-1), AtomicI16::new(-1)];
        let vel = [AtomicI16::new(0), AtomicI16::new(0), AtomicI16::new(0)];
        let cur = [AtomicI16::new(0), AtomicI16::new(0), AtomicI16::new(0)];
        Self { position: pos, velocity: vel, current: cur }
    }

    pub fn set_data(&self, id: u16, pos:i16, vel:i16, cur:i16)
    {
        let index = (id-1) as usize;
        self.position[index].store(pos, Ordering::Relaxed);
        self.velocity[index].store(vel, Ordering::Relaxed);
        self.current[index].store(cur, Ordering::Relaxed);
    }

    pub fn get_data(&self, id: u16)->(i16, i16, i16)
    {
        let index = (id-1) as usize;
        let pos = self.position[index].load(Ordering::Relaxed);
        let vel = self.velocity[index].load(Ordering::Relaxed);
        let cur = self.current[index].load(Ordering::Relaxed);
        (pos, vel, cur)
    }
}