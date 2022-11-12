#![no_std]

use stm32f3xx_hal::i2c;

pub struct Pioled<I2C> {
    i2c: I2C
}
impl<I2C, Bus> Pioled<I2C> where I2C: i2c::Instance<Bus = Bus> {
    pub fn new(i2c: I2C) -> Self {


        Pioled { i2c }
    }
}