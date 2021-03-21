/* Access a host console via Spike's Host Target Interface (HTIF)
 * 
 * This follows the HTIF API defined here:
 * https://github.com/riscv/riscv-isa-sim/issues/364#issuecomment-607657754
 * 
 * It requires two global symbols to be defined...
 *  tohost
 *  fromhost
 * ...which when written to and read from triggers an API call to the HTIF provider
 * 
 * (c) Chris Williams, 2021.
 *
 * See README and LICENSE for usage and copying.
 */

/* we're on our own here */
#![cfg_attr(not(test), no_std)]
#![allow(dead_code)]

use core::ptr::{write_volatile, read_volatile};

extern "C"
{
    /* symbols required by spike: writing to and reading
       from these memory locations is trapped by the simulator
       and treated as API calls */
    static mut tohost: u64;
    static fromhost: u64;
}

/* total register size is 2 x 8-byte words */
const REG_TOTAL_SIZE: usize = 2 * 8;

const DEVICE_SHIFT:       u64 = 56; /* bits 63-56 contain the device number */
const DEVICE_CHARIO:      u64 = 1;  /* device 1 is the blocking character device */

const COMMAND_SHIFT:      u64 = 48; /* bits 55-48 contain the command number */
const COMMAND_READ_CHAR:  u64 = 0;  /* read a character from the host console */
const COMMAND_WRITE_CHAR: u64 = 1;  /* write a character to the host console */

/* possible error conditions supported at this time */
#[derive(Debug)]
pub enum Fault
{
    Success /* HTIF API calls don't fail */
}

#[derive(Debug)]
pub struct HTIF {}

impl HTIF
{
    pub fn new() -> Result<Self, Fault> { Ok( HTIF {} ) }

    /* return size of this controller's MMIO space in bytes */
    pub fn size(&self) -> usize
    {
        REG_TOTAL_SIZE
    }

    /* centralize reading and writing of API addresses to these unsafe functions */
    fn write_to_host(&self, val: u64)
    {
        unsafe { write_volatile(&mut tohost as *mut u64, val) }

        /* do a delay loop as spike seems to drop characters if we write too fast */
        for _ in 0..100
        {
            unsafe { read_volatile(&tohost); }
        }
    }

    fn read_from_host(&self) -> u64
    {
        unsafe { read_volatile(&fromhost) }
    }

    pub fn send_byte(&self, to_send: u8) -> Result<(), Fault>
    {
        /* write a character to the blocking character IO device */
        let device = DEVICE_CHARIO << DEVICE_SHIFT;
        let command = COMMAND_WRITE_CHAR << COMMAND_SHIFT;
        let byte = (to_send as u64) & 0xff;
        self.write_to_host(device | command | byte);
        Ok(())
    }

    pub fn read_byte(&self) -> Result<u8, Fault>
    {
        /* tell the blocking character IO device we want to read a byte */
        let device = DEVICE_CHARIO << DEVICE_SHIFT;
        let command = COMMAND_READ_CHAR << COMMAND_SHIFT;
        self.write_to_host(device | command);

        /* raad that byte */
        Ok((self.read_from_host() & 0xff) as u8)
    }
}

#[cfg(test)]
mod tests
{
    #[test]
    fn it_works()
    {
        assert_eq!(2 + 2, 4);
    }
}
