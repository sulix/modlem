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

use std::fmt::{Debug, Formatter};
use binary_io::*;

const BITMAP_SIGNATURE : u16 = 0x4D42; // 'MB', in little-endian
const BI_RGB : u32 = 0;

#[allow(non_snake_case)]
#[derive(Debug)]
/// A Windows .bmp file header.
/// See: <https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapfileheader>
struct BitmapFileHeader {
    bfType : u16,
    bfSize : u32,
    _bfReserved1 : u16,
    _bfReserved2 : u16,
    bfOffBits : u32
}

impl BitmapFileHeader {
    const STRUCT_SIZE : usize = 14;

    /// Create a new, empty bitmap file header, of a given size and data offset.
    /// @size should probably be BitmapFileHeader::STRUCT_SIZE
    fn new(size : usize, data_offset : usize) -> BitmapFileHeader {
        BitmapFileHeader {
            bfType: BITMAP_SIGNATURE,
            bfSize: size as u32,
            _bfReserved1: 0,
            _bfReserved2: 0,
            bfOffBits: data_offset as u32
        }
    }

    /// Parse a bitmap file header from an input stream.
    fn from_data(reader : &mut dyn std::io::Read) -> std::io::Result<BitmapFileHeader> {
        Ok(BitmapFileHeader {
        bfType : read_le16(reader)?,
        bfSize : read_le32(reader)?,
        _bfReserved1 : read_le16(reader)?,
        _bfReserved2 : read_le16(reader)?,
        bfOffBits : read_le32(reader)?
    })
    }

    /// Write a bitmap file header to an output stream.
    fn write(self: &BitmapFileHeader, writer : &mut dyn std::io::Write) -> std::io::Result<()> {
        write_le16(self.bfType, writer)?;
        write_le32(self.bfSize, writer)?;
        write_le16(self._bfReserved1, writer)?;
        write_le16(self._bfReserved2, writer)?;
        write_le32(self.bfOffBits, writer)?;
        Ok(())
    }
}

#[allow(non_snake_case)]
#[derive(Debug)]
struct BitmapInfoHeader {
    biSize : u32,
    biWidth : u32, /* signed? */
    biHeight : u32, /* signed? */
    biPlanes : u16,
    biBitCount : u16,
    biCompression : u32,
    biSizeImage : u32,
    _biXPelsPerMeter : u32, /* signed? */
    _biYPelsPerMeter : u32, /* signed? */
    biClrUsed : u32,
    _biClrImportant : u32
}

impl BitmapInfoHeader {
    const STRUCT_SIZE : usize = 40;

    fn new(width : usize, height : usize, bpp : usize, pal_size : usize, image_size : usize) -> BitmapInfoHeader {
        BitmapInfoHeader {
            biSize : BitmapInfoHeader::STRUCT_SIZE as u32,
            biWidth : width as u32,
            biHeight : height as u32,
            biPlanes : 1,
            biBitCount : bpp as u16,
            biCompression : BI_RGB,
            biSizeImage : image_size as u32,
            _biXPelsPerMeter : 0,
            _biYPelsPerMeter : 0,
            biClrUsed : pal_size as u32,
            _biClrImportant : 0
        }
    }
    fn from_data(reader : &mut dyn std::io::Read) -> std::io::Result<BitmapInfoHeader> {
        let res = BitmapInfoHeader {
            biSize : read_le32(reader)?,
            biWidth : read_le32(reader)?,
            biHeight : read_le32(reader)?,
            biPlanes : read_le16(reader)?,
            biBitCount : read_le16(reader)?,
            biCompression : read_le32(reader)?,
            biSizeImage : read_le32(reader)?,
            _biXPelsPerMeter : read_le32(reader)?,
            _biYPelsPerMeter : read_le32(reader)?,
            biClrUsed : read_le32(reader)?,
            _biClrImportant : read_le32(reader)?,
        };
        let mut unused = vec![0u8; res.biSize as usize - 40];
        let _ = reader.read_exact(&mut unused[..]);
        Ok(res)
    }
    fn write(self: &BitmapInfoHeader, writer : &mut dyn std::io::Write) -> std::io::Result<()> {
        write_le32(self.biSize, writer)?;
        write_le32(self.biWidth, writer)?;
        write_le32(self.biHeight, writer)?;
        write_le16(self.biPlanes, writer)?;
        write_le16(self.biBitCount, writer)?;
        write_le32(self.biCompression, writer)?;
        write_le32(self.biSizeImage, writer)?;
        write_le32(self._biXPelsPerMeter, writer)?;
        write_le32(self._biYPelsPerMeter, writer)?;
        write_le32(self.biClrUsed, writer)?;
        write_le32(self._biClrImportant, writer)?;
        Ok(())
    }
}

/// An RGB colour tuple, 8 bits per channel.
#[derive(Clone)]
#[derive(Copy)]
struct ColourRGB {
    r : u8,
    g : u8,
    b : u8,
}

impl Debug for ColourRGB {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(r:{}, g:{}, b:{})", self.r, self.g, self.b)
    }
}

impl ColourRGB {
    /// Create a ColourRGB from 3 8-bit values.
    fn rgb(r : u8, g : u8, b : u8) -> ColourRGB {
        ColourRGB { r, g, b }
    }

    /// Create a ColourRGB from 6-bit values, à la VGA palette registers.
    fn vga_rgb(r : u8, g : u8, b : u8) -> ColourRGB {
        ColourRGB { r: r * 4, g: g * 4, b: b * 4 }
    }

    /// Create a ColourRGB from a 2-bit RGB, à la EGA palette registers
    fn ega_rgb(val : u8) -> ColourRGB
    {
        // All this would be much easier if we had the 'pext' instruction.
        ColourRGB {
            r: (((val & 0b00010000) >> 3) | (val & 0b00000100) >> 2) * 85,
            g: (((val & 0b00010000) >> 3) | (val & 0b00000010) >> 1) * 85,
            b: (((val & 0b00010000) >> 3) | (val & 0b00000001)) * 85
        }
    }
}

#[derive(Default)]
#[derive(Clone)]
pub struct PaletteRGB {
    colours : std::vec::Vec::<ColourRGB>
}

impl PaletteRGB {

    /// Create a new pallete, with num_colours entries, all black.
    pub fn new(num_colours : usize) -> PaletteRGB {
        PaletteRGB { colours: vec![ColourRGB::rgb(0,0,0); num_colours] }
    }

    /// Update num_colours contiguous elements from a slice of 6-bit VGA values.
    pub fn set_vga_data(&mut self, start_offset : usize, num_colours : usize, data : &[u8]) {
        for i in 0..num_colours {
            #[allow(clippy::identity_op)]
            let vga_r = data[i*3+0];
            let vga_g = data[i*3+1];
            let vga_b = data[i*3+2];
            self.colours[i+start_offset] = ColourRGB::vga_rgb(vga_r, vga_g, vga_b);
        }
    }
    pub fn from_vga_data(num_colours : usize, data : &[u8]) -> PaletteRGB {
        let mut out = PaletteRGB::default();
        for i in 0..num_colours {
            #[allow(clippy::identity_op)]
            let vga_r = data[i*3+0];
            let vga_g = data[i*3+1];
            let vga_b = data[i*3+2];
            out.colours.push(ColourRGB::vga_rgb(vga_r, vga_g, vga_b));
        }
        out
    }

    /// Update num_colours contiguous elements from a slice of 2-bit EGA values.
    pub fn set_ega_data(&mut self, start_offset : usize, num_colours : usize, data : &[u8]) {
        for (i, &val) in data.iter().enumerate().take(num_colours) {
            self.colours[i+start_offset] = ColourRGB::ega_rgb(val);
        }
    }

    /// Create a new PaletteRGB from EGA data. Unused for now.
    #[allow(dead_code)]
    pub fn from_ega_data(num_colours : usize, data : &[u8]) -> PaletteRGB {
        let mut out = PaletteRGB::default();
        for &val in data.iter().take(num_colours) {
            out.colours.push(ColourRGB::ega_rgb(val));
        }
        out
    }
    /// Write the first num_colours entries to a file in Windows BMP 'RGBQUADS' format.
    fn write_prefix_as_rgbquads(&self, writer : &mut dyn std::io::Write, num_colours : usize) -> std::io::Result::<()> {
        assert!(num_colours <= self.colours.len());
        for &entry in self.colours.iter().take(num_colours) {
            write_byte(entry.b, writer)?;
            write_byte(entry.g, writer)?;
            write_byte(entry.r, writer)?;
            write_byte(0, writer)?;
        }
        Ok(())
    }

    /// Write the entire palette to a file in Windows .BMP 'RGBQUADS' format.
    #[allow(dead_code)]
    fn write_as_rgbquads(&self, writer : &mut dyn std::io::Write) -> std::io::Result::<()> {
        self.write_prefix_as_rgbquads(writer, self.colours.len())
    }

    /// Read a palette with num_colours entries from a .BMP file in 'RGBQUADS' format.
    fn read_as_rgbquads(reader : &mut dyn std::io::Read, num_colours : usize) -> std::io::Result<PaletteRGB>
    {
        let mut pal = PaletteRGB::new(num_colours);
        for i in 0..num_colours {
            pal.colours[i].b = read_byte(reader)?;
            pal.colours[i].g = read_byte(reader)?;
            pal.colours[i].r = read_byte(reader)?;
            read_byte(reader)?;
        }
        Ok(pal)
    }

}

/// A Bitmap consisting of several 'planes': 1bpp images which are overlaid.
/// For example, EGA/16-colour VGA uses a 4-plane format.
/// 4- and 8- plane images can be saved as Windows .BMP files.
pub struct PlanarBMP {
    pub width: usize,
    pub height: usize,
    pitch: usize,
    pub planes: usize,
    data: std::vec::Vec<u8>,
    palette: PaletteRGB,
}

impl PlanarBMP {
    /// Create a new empty (all palette entry 0) bitmap, of size @width×@height, and @planes planes.
    pub fn new(width: usize, height: usize, planes: usize, palette : &PaletteRGB) -> PlanarBMP {
        assert!(planes <= 8);
        let pitch = width.div_ceil(8);
        let plane_size = pitch * height;
        PlanarBMP {
            width,
            height,
            pitch,
            planes,
            data: vec![0; plane_size * planes],
            palette : palette.clone()
        }
    }

    /// Create a new bitmap from 'contiguous' data, i.e., where all of plane 0 is stored, followed immediately by plane 1, etc.
    pub fn from_contiguous_data(data: &[u8], width: usize, height: usize, planes: usize, palette: &PaletteRGB) -> PlanarBMP {
        let pitch = width.div_ceil(8);
        PlanarBMP {
            width,
            height,
            pitch,
            planes,
            data: data.to_vec(),
            palette : palette.clone()
        }
    }

    /// Create a new bitmap from 'packed' (or chunky) data, i.e. all data for each pixel are packed tightly together.
    /// Think RGBIRGBIRGBI not RRRRGGGGBBBBIIII.
    /// Note: only supports 8 and 4 plane images.
    pub fn from_packed_data(data: &[u8], width: usize, height: usize, planes: usize, palette: &PaletteRGB) -> PlanarBMP {
        let mut planar_data = std::vec::Vec::<u8>::new();
        // Windows bitmaps have scanlines aligned on 32-bit boundaries.
        let scanline_size = (width * planes) / 8;
        let pitch = (scanline_size + 3) & !3;
        // But are not padded to those boundaries.
        match planes {
            8 => {
                for plane in 0..planes {
                    for y in 0..height {
                        let mut out_byte : u8 = 0;
                        for x in 0..width {
                            let px_byte = data[(height - y - 1) * pitch + x];
                            let px_mask: u8 = 1 << plane;
                            let px_val : u8 = if (px_byte & px_mask) != 0 { 1 << (7 - (x as u8 % 8)) } else { 0 };
                            out_byte |= px_val;
                            if x % 8 == 7 {
                                planar_data.push(out_byte);
                                out_byte = 0;
                            }
                        }
                        //planar_data.push(out_byte);
                    }
                }
            }
            4 => {
                for plane in 0..planes {
                    for y in 0..height {
                        let mut out_byte : u8 = 0;
                        for x in 0..width {
                            let px_byte = data[(height - y - 1) * pitch + (x / 2)];
                            let px_mask: u8 = (1 << plane) * if (x & 1) == 0 { 16 } else { 1 };
                            let px_val : u8 = if (px_byte & px_mask) != 0 { 1 << (7 - (x as u8 % 8)) } else { 0 };
                            out_byte |= px_val;
                            if x % 8 == 7 {
                                planar_data.push(out_byte);
                                out_byte = 0;
                            }
                        }
                        if width & 1 != 0 {
                            planar_data.push(out_byte);
                        }
                    }
                }
            }
            1 => {
                for y in 0..height {
                    let line_off = (height - y - 1) * pitch;
                    planar_data.extend_from_slice(&data[line_off..line_off+scanline_size]);
                }
            }
            _ => {
                panic!("Unsupported bit depth!");
            }
        }

        PlanarBMP {
            width,
            height,
            pitch: width / 8,
            planes,
            data: planar_data,
            palette: palette.clone()
        }
    }

    // Creates a new PlanarBMP by copying (and swizzling) a set of planes from another.
    pub fn from_swizzle(bmp : &PlanarBMP, plane_map : Vec<usize>) -> PlanarBMP {
        let plane_size = bmp.pitch * bmp.height;
        let mut planar_data = vec![0u8; plane_size * plane_map.len()];

        for (dst_plane, &src_plane)  in plane_map.iter().enumerate() {
            assert!(src_plane < bmp.planes);
            planar_data[(dst_plane * plane_size)..((dst_plane+1)*plane_size)].copy_from_slice(&bmp.get_plane_data(src_plane, 0, 0, bmp.width, bmp.height));
        }


        PlanarBMP {
            width: bmp.width,
            height: bmp.height,
            pitch: bmp.pitch,
            planes: plane_map.len(),
            data: planar_data,
            palette: bmp.palette.clone()
        }

    }

    /// Load a PlanarBMP from a Windows .BMP file. Both Windows 3.1 and Windows 98 formats are
    /// supported, in 4- or 8- bit depths.
    pub fn from_file(reader : &mut dyn std::io::Read) -> std::io::Result<PlanarBMP> {
        let bfh = BitmapFileHeader::from_data(reader)?;

        assert_eq!(bfh.bfType, BITMAP_SIGNATURE);

        let bih = BitmapInfoHeader::from_data(reader)?;

        let pal = PaletteRGB::read_as_rgbquads(reader, bih.biClrUsed as usize)?;

        let mut data = vec![0; (bih.biSizeImage) as usize];
        reader.read_exact(&mut data)?;

        Ok(PlanarBMP::from_packed_data(&data[..], bih.biWidth as usize, bih.biHeight as usize, bih.biBitCount as usize, &pal))
    }

    /// Read one pixel value, with all planes packed together.
    pub fn get_packed_pixel(&self, x : usize, y : usize) -> u8 {
        let mut pixel_value: u8 = 0;
        let plane_size = self.pitch * self.height;
        for plane in 0..self.planes {
            let byte: u8 = self.data[plane * plane_size + y * self.pitch + (x / 8)];
            let px_mask: u8 = 1 << (7 - (x % 8));
            let px_plane_value: u8 = if byte & px_mask != 0 { 1 << plane } else { 0 };
            pixel_value |= px_plane_value;
        }
        pixel_value
    }

    /// Set one pixel value, with all planes packed together.
    pub fn pset(&mut self, x : usize, y : usize, value : u8) {
        let plane_size = self.pitch * self.height;
        for plane in 0..self.planes {
            let mut byte: u8 = self.data[plane * plane_size + y * self.pitch + (x / 8)];
            let px_mask: u8 = 1 << (7 - (x % 8));
            if (value & (1 << plane)) != 0 {
                byte |= px_mask;
            } else {
                byte &= !px_mask;
            }
            self.data[plane * plane_size + y * self.pitch + (x / 8)] = byte
        }
    }

    /// Get a vector with the 8-bit packed representation of the image.
    pub fn to_pal8_data(&self) -> Vec<u8> {
        let mut output = std::vec::Vec::<u8>::new();
        let plane_size = self.pitch * self.height;
        for y in 0..self.height {
            for x in 0..self.width {
                let mut pixel_value: u8 = 0;
                for plane in 0..self.planes {
                    let byte: u8 = self.data[plane * plane_size + y * self.pitch + (x / 8)];
                    let px_mask: u8 = 1 << (7 - (x % 8));
                    let px_plane_value: u8 = if byte & px_mask != 0 { 1 << plane } else { 0 };
                    pixel_value |= px_plane_value;
                }
                output.push(pixel_value);
            }
        }
        output
    }

    pub fn get_plane_data(&self, plane : usize, x : usize, y : usize, w : usize, h : usize) -> Vec<u8> {
        let mut output = std::vec::Vec::<u8>::new();
        let plane_size = self.pitch * self.height;

        for yi in 0..h {
            let mut out_byte : u8 = 0;
            for xi in 0..w {
                let px_byte = self.data[plane * plane_size + (yi + y) * (self.pitch) + ((xi + x) / 8)];
                let px_mask: u8 = 1 << (7 - ((xi + x) % 8));
                let px_val : u8 = if (px_byte & px_mask) != 0 { 1 << (7 - (xi as u8 % 8)) } else { 0 };
                out_byte |= px_val;
                if xi % 8 == 7 {
                    output.push(out_byte);
                    out_byte = 0;
                }
            }
        }
        output
    }

    pub fn blit(&mut self, src : &PlanarBMP, x : usize, y : usize) {
        assert!(src.width + x <= self.width);
        assert!(src.height + y <= self.height);
        // TODO: Support this?
        assert_eq!(src.planes, self.planes);
        for src_y in 0..src.height {
            for src_x in 0..src.width {
                let val = src.get_packed_pixel(src_x, src_y);
                self.pset(src_x + x, src_y + y, val);
            }
        }
    }

    #[allow(dead_code)] // We're not using this yet, but will for levels.
    pub fn blit_masked(&mut self, src : &PlanarBMP, x : usize, y : usize) {
        assert!(src.width + x <= self.width);
        assert!(src.height + y <= self.height);
        assert_eq!(src.planes, self.planes + 1);
        for src_y in 0..src.height {
            for src_x in 0..src.width {
                let val = src.get_packed_pixel(src_x, src_y);
                if val & (1 << self.planes) != 0 {
                    self.pset(src_x + x, src_y + y, val & ((1 << self.planes) - 1));
                }
            }
        }
    }

    pub fn save_as_pal8(&self, writer : &mut dyn std::io::Write) {
        /* For PAL8, pitch == width. */
        let data_size = self.width * self.height;
        let num_colours = self.palette.colours.len();
        let data_offset = BitmapFileHeader::STRUCT_SIZE + BitmapInfoHeader::STRUCT_SIZE + 4 * num_colours;
        let bmp_file_header = BitmapFileHeader::new(data_offset + data_size, data_offset);
        let bmp_info_header = BitmapInfoHeader::new(self.width, self.height, 8, num_colours, data_size);
        bmp_file_header.write(writer).unwrap();
        bmp_info_header.write(writer).unwrap();
        self.palette.write_prefix_as_rgbquads(writer, num_colours).unwrap();
        let data = self.to_pal8_data();
        for line in data.rchunks(self.width) {
            writer.write_all(line).unwrap();
        }
    }

    pub fn save_as_bpp(&self, bpp : usize, writer : &mut dyn std::io::Write) {
        let pitch = ((self.width * bpp).div_ceil(8) + 3) & !3;
        let data_size = pitch * self.height;
        let num_colours = std::cmp::min(self.palette.colours.len(), 1 << bpp);
        let data_offset = BitmapFileHeader::STRUCT_SIZE + BitmapInfoHeader::STRUCT_SIZE + 4 * num_colours;
        let bmp_file_header = BitmapFileHeader::new(data_offset + data_size, data_offset);
        let bmp_info_header = BitmapInfoHeader::new(self.width, self.height, bpp, num_colours, data_size);
        bmp_file_header.write(writer).unwrap();
        bmp_info_header.write(writer).unwrap();
        self.palette.write_prefix_as_rgbquads(writer, num_colours).unwrap();
        let data = self.to_pal8_data();
        for line in data.rchunks(self.width) {
            let mut byte_count = 0;
            let mut byte_buffer : u32 = 0;
            let mut current_bit = 0;
            for px in line {
                byte_buffer <<= bpp;
                byte_buffer |= *px as u32;
                current_bit += bpp;
                if current_bit >= 8 {
                    let cur_byte = byte_buffer as u8;
                    write_byte(cur_byte, writer).unwrap();
                    byte_buffer >>= 8;
                    current_bit -= 8;
                    byte_count += 1;
                }
            }
            while byte_count < pitch {
                write_byte(byte_buffer as u8, writer).unwrap();
                byte_buffer >>= 8;
                byte_count += 1;
            }
        }
    }

    pub fn save_as_file(&self, writer : &mut dyn std::io::Write) {
        match self.planes {
            1 => self.save_as_bpp(1, writer),
            #[allow(clippy::manual_range_patterns)]
            2 | 3 | 4 => self.save_as_bpp(4, writer),
            _ => self.save_as_pal8(writer),
        }
    }

}

