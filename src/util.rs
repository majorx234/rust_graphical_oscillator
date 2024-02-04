/*
 * This file is part of the rust_midi_mapper distribution (https://github.com/majorx234/rust_graphical_oscillator ).
 * Copyright (c) 2022-2024 Majorx234 <majorx234@googlemail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful, but
 * WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

#[inline(always)]
pub fn mask7(input: u8) -> u8 {
    input & 0b01111111
}

/// 14 bit mask
#[inline(always)]
pub fn mask14(input: u16) -> u16 {
    input & 0b0011111111111111
}

/// Extract the MSB and LSB from a `U14`
#[inline]
pub fn u14_to_msb_lsb(input: u16) -> (u8, u8) {
    let msb = mask7((input >> 7) as u8);
    let lsb = mask7(input as u8);
    (msb, lsb)
}

/// Convert an MSB and LSB to a `U14`
#[inline]
pub fn msb_lsb_to_u14(msb: u8, lsb: u8) -> u16 {
    ((mask7(msb) as u16) << 7) + mask7(lsb) as u16
}

/// Calculate the status byte for a given channel no.
#[inline(always)]
pub fn status_byte(status: u8, channel: u8) -> u8 {
    (status & 0b00001111) * 16 + channel
}

/// Seperate the status from the channel no.
#[inline]
pub fn from_status_byte(sb: u8) -> (u8, u8) {
    let status = (sb & 0b11110000) >> 4;
    let channel = sb & 0b00001111;
    (status, channel)
}
