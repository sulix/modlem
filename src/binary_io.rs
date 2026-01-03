/*
 * modlem: A graphics importer/exporter for Lemmings
 * Copyright (C) 2022–2026 David Gow <david@davidgow.net>
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see <https://www.gnu.org/licenses/>.
 */

#![allow(dead_code)]

/// Reads a single byte (unsigned, 8-bit integer) from an input stream.
pub fn read_byte(reader: &mut dyn std::io::Read) -> std::io::Result<u8> {
    let mut out_byte : u8 = 0;
    reader.read_exact(std::slice::from_mut(&mut out_byte))?;
    return Ok(out_byte);
}

/// Reads a little-endian unsigned 16-bit integer from a stream.
pub fn read_le16(reader : &mut dyn std::io::Read) -> std::io::Result<u16> {
    let mut raw_bytes = [0 as u8; 2];
    reader.read_exact(&mut raw_bytes)?;
    return Ok((raw_bytes[1] as u16) << 8 | (raw_bytes[0] as u16));
}

/// Reads a little-endian unsigned 32-bit integer from a stream.
pub fn read_le32(reader : &mut dyn std::io::Read) -> std::io::Result<u32> {
    let mut raw_bytes = [0 as u8; 4];
    reader.read_exact(&mut raw_bytes)?;
    return Ok((raw_bytes[3] as u32) << 24 | (raw_bytes[2] as u32) << 16 | (raw_bytes[1] as u32) << 8 | (raw_bytes[0] as u32));
}

/// Reads a big-endian unsigned 16-bit integer from a stream.
pub fn read_be16(reader : &mut dyn std::io::Read) -> std::io::Result<u16> {
    let mut raw_bytes = [0 as u8; 2];
    reader.read_exact(&mut raw_bytes)?;
    return Ok((raw_bytes[0] as u16) << 8 | (raw_bytes[1] as u16));
}

/// Reads a big-endian unsigned 32-bit integer from a stream.
pub fn read_be32(reader : &mut dyn std::io::Read) -> std::io::Result<u32> {
    let mut raw_bytes = [0 as u8; 4];
    reader.read_exact(&mut raw_bytes)?;
    return Ok((raw_bytes[0] as u32) << 24 | (raw_bytes[1] as u32) << 16 | (raw_bytes[2] as u32) << 8 | (raw_bytes[3] as u32));
}

/// Writes a single byte (unsigned, 8-bit integer) to a stream.
pub fn write_byte(out_byte : u8, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    writer.write_all(std::slice::from_ref(&out_byte))
}

/// Writes a big-endian unsigned 16-bit integer to a stream.
pub fn write_be16(out_val : u16, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let raw_bytes = [(out_val >> 8) as u8, (out_val & 0xFF) as u8];
    writer.write_all(&raw_bytes)
}

/// Writes a big-endian unsigned 32-bit integer to a stream.
pub fn write_be32(out_val : u32, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let raw_bytes = [((out_val >> 24) )as u8, (out_val >> 16) as u8, (out_val >> 8) as u8, (out_val & 0xFF) as u8];
    writer.write_all(&raw_bytes)
}

/// Writes a little-endian unsigned 16-bit integer to a stream.
pub fn write_le16(out_val : u16, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let raw_bytes = [(out_val & 0xFF) as u8, (out_val >> 8) as u8];
    writer.write_all(&raw_bytes)
}

/// Writes a little-endian unsigned 32-bit integer to a stream.
pub fn write_le32(out_val : u32, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    let raw_bytes = [(out_val) as u8, (out_val >> 8) as u8, (out_val >> 16) as u8, (out_val >> 24) as u8];
    writer.write_all(&raw_bytes)
}
