use binary_io::*;

const BITMAP_SIGNATURE : u16 = 0x4D42; // 'MB', in little-endian
const BI_RGB : u32 = 0;

#[allow(non_snake_case)]
struct BitmapFileHeader {
    bfType : u16,
    bfSize : u32,
    _bfReserved1 : u16,
    _bfReserved2 : u16,
    bfOffBits : u32
}

impl BitmapFileHeader {
    const STRUCT_SIZE : usize = 14;

    fn new(size : usize, data_offset : usize) -> BitmapFileHeader {
        BitmapFileHeader {
            bfType: BITMAP_SIGNATURE,
            bfSize: size as u32,
            _bfReserved1: 0,
            _bfReserved2: 0,
            bfOffBits: data_offset as u32
        }
    }
    fn from_data(reader : &mut dyn std::io::Read) -> std::io::Result<BitmapFileHeader> {
        Ok(BitmapFileHeader {
        bfType : read_le16(reader)?,
        bfSize : read_le32(reader)?,
        _bfReserved1 : read_le16(reader)?,
        _bfReserved2 : read_le16(reader)?,
        bfOffBits : read_le32(reader)?
    })
    }
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
        Ok(BitmapInfoHeader {
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

    })
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

#[derive(Clone)]
struct ColourRGB {
    r : u8,
    g : u8,
    b : u8,
}

impl ColourRGB {
    fn rgb(r : u8, g : u8, b : u8) -> ColourRGB {
        ColourRGB { r, g, b }
    }
    fn vga_rgb(r : u8, g : u8, b : u8) -> ColourRGB {
        ColourRGB { r: r * 4, g: g * 4, b: b * 4 }
    }
}

#[derive(Default)]
pub struct PaletteRGB {
    colours : std::vec::Vec::<ColourRGB>
}

impl PaletteRGB {
    fn from_vga_data(num_colours : usize, data : &[u8]) -> PaletteRGB {
        let mut out  = PaletteRGB::default();
        for i in 0..num_colours {
            let vga_r = data[i*3+0];
            let vga_g = data[i*3+1];
            let vga_b = data[i*3+2];
            out.colours.push(ColourRGB::vga_rgb(vga_r, vga_g, vga_b));
        }
        out
    }
}

pub struct PlanarBMP {
    width: usize,
    height: usize,
    pitch: usize,
    planes: usize,
    data: std::vec::Vec<u8>,
    palette: std::vec::Vec<u8>,
}

impl PlanarBMP {
    pub fn new(width: usize, height: usize, planes: usize, palette : std::vec::Vec<u8>) -> PlanarBMP {
        assert!(planes <= 8);
        let pitch = (width + 7) / 8;
        let plane_size = pitch * height;
        PlanarBMP {
            width,
            height,
            pitch,
            planes,
            data: vec![0; plane_size * planes],
            palette
        }
    }
    pub fn from_contiguous_data(data: &[u8], width: usize, height: usize, planes: usize, palette: std::vec::Vec<u8>) -> PlanarBMP {
        let pitch = (width + 7) / 8;
        PlanarBMP {
            width,
            height,
            pitch,
            planes,
            data: data.to_vec(),
            palette
        }
    }

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

    pub fn pset(&mut self, x : usize, y : usize, value : u8) {
        let plane_size = self.pitch * self.height;
        for plane in 0..self.planes {
            let mut byte: u8 = self.data[plane * plane_size + y * self.pitch + (x / 8)];
            let px_mask: u8 = 1 << (7 - (x % 8));
            let px_plane_value: u8 = if byte & px_mask != 0 { 1 << plane } else { 0 };
            if (value & (1 << plane)) != 0 {
                byte |= px_mask;
            } else {
                byte &= !px_mask;
            }
            self.data[plane * plane_size + y * self.pitch + (x / 8)] = byte
        }
    }

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
        return output;
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

    pub fn save_as_pal8(&self, writer : &mut dyn std::io::Write) {
        /* For PAL8, pitch == width. */
        let data_size = self.width * self.height;
        let num_colours = 1 << self.planes;
        let data_offset = BitmapFileHeader::STRUCT_SIZE + BitmapInfoHeader::STRUCT_SIZE + 4 * num_colours;
        let bmp_file_header = BitmapFileHeader::new(data_offset + data_size, data_offset);
        let bmp_info_header = BitmapInfoHeader::new(self.width, self.height, 8, num_colours, data_size);
        bmp_file_header.write(writer).unwrap();
        bmp_info_header.write(writer).unwrap();
        writer.write_all(self.palette.as_slice()).unwrap();
        let data = self.to_pal8_data();
        for line in data.rchunks(self.width) {
            writer.write_all(line).unwrap();
        }
    }
}

