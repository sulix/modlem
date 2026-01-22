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

use std::convert::TryInto;
use std::fs::File;
use std::path::Path;
use binary_io::*;
use dat_section::DatSection;
use planar_bmp;
use parser;

#[derive(Debug)]
#[derive(Default)]
#[allow(dead_code)]
pub struct ObjectHeader
{
    pub animation_flags : u16,
    pub frame_start : u8,
    pub frame_end : u8,
    pub width : u8,
    pub height : u8,
    pub animation_frame_data_size : u16,
    pub mask_offset : u16,
    pub _unknown0 : u16,
    pub _unknown1 : u16,
    pub trigger_x : u16,
    pub trigger_y : u16,
    pub trigger_w : u8,
    pub trigger_h : u8,
    pub trigger_effect_id : u8,
    pub animation_offset : u16,
    pub preview_frame_offset : u16,
    pub preview_frame_number : u8,
    pub _unknown2 : u16,
    pub trap_sound : u8,
}

impl std::fmt::Display for ObjectHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{\n")?;
        write!(f, "\tanimation_flags = {}\n", self.animation_flags)?;
        write!(f, "\tframes = ({},{})\n", self.frame_start, self.frame_end)?;
        //write!(f, "\tsize = ({},{})\n", self.width, self.height)?;
        write!(f, "\ttrigger = ({},{},{},{})\n", self.trigger_x, self.trigger_y, self.trigger_w, self.trigger_h)?;
        write!(f, "\ttrigger_effect = {}\n", self.trigger_effect_id)?;
        write!(f, "\tpreview_frame = {}\n", self.preview_frame_number)?;
        write!(f, "\ttrap_sound = {}\n", self.trap_sound)?;
        write!(f, "}}")
    }
}

impl ObjectHeader {
    pub fn read(reader : &mut dyn std::io::Read) -> std::io::Result<ObjectHeader> {
        let mut oh = ObjectHeader {
            animation_flags : read_le16(reader)?,
            frame_start : read_byte(reader)?,
            frame_end : read_byte(reader)?,
            width : read_byte(reader)?,
            height : read_byte(reader)?,
            animation_frame_data_size : read_le16(reader)?,
            mask_offset : read_le16(reader)?,
            _unknown0 : read_le16(reader)?,
            _unknown1 : read_le16(reader)?,
            trigger_x : read_le16(reader)?,
            trigger_y : read_le16(reader)?,
            trigger_w : read_byte(reader)?,
            trigger_h : read_byte(reader)?,
            trigger_effect_id : read_byte(reader)?,
            animation_offset : read_le16(reader)?,
            preview_frame_offset : read_le16(reader)?,
            preview_frame_number : 0,
            _unknown2 : read_le16(reader)?,
            trap_sound : read_byte(reader)?,
        };
        if oh.animation_frame_data_size != 0 {
            oh.preview_frame_number = ((oh.preview_frame_offset - oh.animation_offset) / oh.animation_frame_data_size) as u8;
        }
        Ok(oh)
    }
    pub fn write(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        write_le16(self.animation_flags, writer)?;
        write_byte(self.frame_start, writer)?;
        write_byte(self.frame_end, writer)?;
        write_byte(self.width, writer)?;
        write_byte(self.height, writer)?;
        write_le16(self.animation_frame_data_size, writer)?;
        write_le16(self.mask_offset, writer)?;
        write_le16(self._unknown0, writer)?;
        write_le16(self._unknown1, writer)?;
        write_le16(self.trigger_x, writer)?;
        write_le16(self.trigger_y, writer)?;
        write_byte(self.trigger_w, writer)?;
        write_byte(self.trigger_h, writer)?;
        write_byte(self.trigger_effect_id, writer)?;
        write_le16(self.animation_offset, writer)?;
        write_le16(self.preview_frame_offset, writer)?;
        write_le16(self._unknown2, writer)?;
        write_byte(self.trap_sound, writer)?;
        Ok(())
    }
    pub fn parse(lex: &mut parser::Lexer) -> ObjectHeader {
        let mut res = ObjectHeader {
            animation_flags : 0,
            frame_start : 0,
            frame_end : 0,
            width : 0,
            height : 0,
            animation_frame_data_size : 0,
            mask_offset : 0,
            _unknown0 : 0,
            _unknown1 : 0,
            trigger_x : 0,
            trigger_y : 0,
            trigger_w : 0,
            trigger_h : 0,
            trigger_effect_id : 0,
            animation_offset : 0,
            preview_frame_offset : 0,
            preview_frame_number : 0,
            _unknown2 : 0,
            trap_sound : 0
        };

        lex.expect_symbol('{');
        loop {
            let tok = lex.next_token();
            match tok.unwrap() {
                parser::Token::Ident(var) => {
                    lex.expect_symbol('=');
                    match var {
                        "animation_flags" => {
                            res.animation_flags = lex.get_int_literal() as u16;
                        },
                        "frames" => {
                            lex.expect_symbol('(');
                            res.frame_start = lex.get_int_literal() as u8;
                            lex.expect_symbol(',');
                            res.frame_end = lex.get_int_literal() as u8;
                            lex.expect_symbol(')');
                        },
                        "trigger" => {
                            lex.expect_symbol('(');
                            res.trigger_x = lex.get_int_literal() as u16;
                            lex.expect_symbol(',');
                            res.trigger_y = lex.get_int_literal() as u16;
                            lex.expect_symbol(',');
                            res.trigger_w = lex.get_int_literal() as u8;
                            lex.expect_symbol(',');
                            res.trigger_h = lex.get_int_literal() as u8;
                            lex.expect_symbol(')');
                        },
                        "trigger_effect" => {
                            res.trigger_effect_id = lex.get_int_literal() as u8;
                        },
                        "preview_frame" => {
                            res.preview_frame_number = lex.get_int_literal() as u8;
                        },
                        "trap_sound" => {
                            res.trap_sound = lex.get_int_literal() as u8;
                        },
                        _ => {
                            panic!("Unknown object property {}", var);
                        }
                    }
                }
                parser::Token::Symbol('}') => {
                    // We're done
                    break;
                }
                _ => {
                    panic!("Invalid object spec");
                }
            }
        }
        res
    }
}
#[derive(Debug)]
#[derive(Default)]
#[allow(dead_code)]
pub struct TerrainHeader
{
    pub width : u8,
    pub height : u8,
    pub gfx_offset : u16,
    pub mask_offset : u16,
    pub _unknown1 : u16,
}

impl TerrainHeader
{
    pub fn read(reader : &mut dyn std::io::Read) -> std::io::Result<TerrainHeader> {
        Ok(TerrainHeader {
            width : read_byte(reader)?,
            height : read_byte(reader)?,
            gfx_offset : read_le16(reader)?,
            mask_offset : read_le16(reader)?,
            _unknown1 : read_le16(reader)?,
        })
    }

    pub fn write(&self, writer : &mut dyn std::io::Write) -> std::io::Result<()> {
        write_byte(self.width, writer)?;
        write_byte(self.height, writer)?; /* checksum */
        write_le16(self.gfx_offset, writer)?;
        write_le16(self.mask_offset, writer)?;
        write_le16(self._unknown1, writer)?;
        Ok(())
    }
}


#[derive(Default)]
pub struct Palettes
{
    pub ega_custom : [u8; 8],
    pub ega_standard : [u8; 8],
    pub ega_preview : [u8; 8],
    pub vga_custom : [u8; 24],
    pub vga_standard : [u8; 24],
    pub vga_preview : [u8; 24],
}

impl Palettes
{
    pub fn read(reader : &mut dyn std::io::Read) -> std::io::Result<Palettes> {
        let mut raw_pal = [0u8; 32 * 3];
        reader.read_exact(&mut raw_pal)?;
        Ok(Palettes {
            ega_custom : raw_pal[0..8].try_into().unwrap(),
            ega_standard : raw_pal[8..16].try_into().unwrap(),
            ega_preview : raw_pal[16..24].try_into().unwrap(),
            vga_custom : raw_pal[24..48].try_into().unwrap(),
            vga_standard : raw_pal[48..72].try_into().unwrap(),
            vga_preview : raw_pal[72..96].try_into().unwrap(),
        })
    }

    pub fn write(&self, writer : &mut dyn std::io::Write) {
        writer.write_all(&self.ega_custom).unwrap();
        writer.write_all(&self.ega_standard).unwrap();
        writer.write_all(&self.ega_preview).unwrap();
        writer.write_all(&self.vga_custom).unwrap();
        writer.write_all(&self.vga_standard).unwrap();
        writer.write_all(&self.vga_preview).unwrap();
    }

    /// Parse a palette config from a text file lexer.
    pub fn parse(lex: &mut parser::Lexer) -> Palettes {
        let mut pal = Palettes::default();

        lex.expect_symbol('{');
        loop {
            let tok = lex.next_token();
            match tok.unwrap() {
                parser::Token::Ident(var) => {
                    lex.expect_symbol('=');
                    match var {
                        "ega_custom" => {
                            lex.expect_symbol('{');
                            for i in 0..8 {
                                lex.expect_symbol('(');
                                let red = lex.get_int_literal() as u8;
                                lex.expect_symbol(',');
                                let green = lex.get_int_literal() as u8;
                                lex.expect_symbol(',');
                                let blue = lex.get_int_literal() as u8;
                                lex.expect_symbol(')');
                                let val = red << 4 | green << 2 | blue;
                                pal.ega_custom[i] = val;
                                if i < 7 {
                                    lex.expect_symbol(',');
                                }
                            }
                            lex.expect_symbol('}');
                        },
                        "ega_standard" => {
                            lex.expect_symbol('{');
                            for i in 0..8 {
                                lex.expect_symbol('(');
                                let red = lex.get_int_literal() as u8;
                                lex.expect_symbol(',');
                                let green = lex.get_int_literal() as u8;
                                lex.expect_symbol(',');
                                let blue = lex.get_int_literal() as u8;
                                lex.expect_symbol(')');
                                let val = red << 4 | green << 2 | blue;
                                pal.ega_standard[i] = val;
                                if i < 7 {
                                    lex.expect_symbol(',');
                                }
                            }
                            lex.expect_symbol('}');
                        },
                        "ega_preview" => {
                            lex.expect_symbol('{');
                            for i in 0..8 {
                                lex.expect_symbol('(');
                                let red = lex.get_int_literal() as u8;
                                lex.expect_symbol(',');
                                let green = lex.get_int_literal() as u8;
                                lex.expect_symbol(',');
                                let blue = lex.get_int_literal() as u8;
                                lex.expect_symbol(')');
                                let val = red << 4 | green << 2 | blue;
                                pal.ega_preview[i] = val;
                                if i < 7 {
                                    lex.expect_symbol(',');
                                }
                            }
                            lex.expect_symbol('}');
                        },
                        "vga_custom" => {
                            lex.expect_symbol('{');
                            for i in 0..8 {
                                lex.expect_symbol('(');
                                let red = lex.get_int_literal() as u8;
                                lex.expect_symbol(',');
                                let green = lex.get_int_literal() as u8;
                                lex.expect_symbol(',');
                                let blue = lex.get_int_literal() as u8;
                                lex.expect_symbol(')');
                                pal.vga_custom[i*3 + 0] = red;
                                pal.vga_custom[i*3 + 1] = green;
                                pal.vga_custom[i*3 + 2] = blue;
                                if i < 7 {
                                    lex.expect_symbol(',');
                                }
                            }
                            lex.expect_symbol('}');
                        },
                        "vga_standard" => {
                            lex.expect_symbol('{');
                            for i in 0..8 {
                                lex.expect_symbol('(');
                                let red = lex.get_int_literal() as u8;
                                lex.expect_symbol(',');
                                let green = lex.get_int_literal() as u8;
                                lex.expect_symbol(',');
                                let blue = lex.get_int_literal() as u8;
                                lex.expect_symbol(')');
                                pal.vga_standard[i*3 + 0] = red;
                                pal.vga_standard[i*3 + 1] = green;
                                pal.vga_standard[i*3 + 2] = blue;
                                if i < 7 {
                                    lex.expect_symbol(',');
                                }
                            }
                            lex.expect_symbol('}');
                        },
                        "vga_preview" => {
                            lex.expect_symbol('{');
                            for i in 0..8 {
                                lex.expect_symbol('(');
                                let red = lex.get_int_literal() as u8;
                                lex.expect_symbol(',');
                                let green = lex.get_int_literal() as u8;
                                lex.expect_symbol(',');
                                let blue = lex.get_int_literal() as u8;
                                lex.expect_symbol(')');
                                pal.vga_preview[i*3 + 0] = red;
                                pal.vga_preview[i*3 + 1] = green;
                                pal.vga_preview[i*3 + 2] = blue;
                                if i < 7 {
                                    lex.expect_symbol(',');
                                }
                            }
                            lex.expect_symbol('}');
                        },
                        _ => {
                            panic!("Unknown palette {}", var);
                        }
                    }
                }
                parser::Token::Symbol('}') => {
                    // We're done
                    break;
                }
                _ => {
                    panic!("Invalid palettes spec");
                }
            }
        }
        pal
    }

}

impl std::fmt::Display for Palettes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        write!(f, "\n\tega_custom = {{")?;
        for i in 0..8 {
            let red = (self.ega_custom[i] & 0b00110000) >> 4;
            let green = (self.ega_custom[i] & 0b00001100) >> 2;
            let blue = self.ega_custom[i] & 0b00000011;
            let terminator = if i < 7 { ',' } else { '}' };
            write!(f, "({}, {}, {}){}", red, green, blue, terminator)?;
        }
        write!(f, "\n\tega_standard = {{")?;
        for i in 0..8 {
            let red = (self.ega_standard[i] & 0b00110000) >> 4;
            let green = (self.ega_standard[i] & 0b00001100) >> 2;
            let blue = self.ega_standard[i] & 0b00000011;
            let terminator = if i < 7 { ',' } else { '}' };
            write!(f, "({}, {}, {}){}", red, green, blue, terminator)?;
        }
        write!(f, "\n\tega_preview = {{")?;
        for i in 0..8 {
            let red = (self.ega_preview[i] & 0b00110000) >> 4;
            let green = (self.ega_preview[i] & 0b00001100) >> 2;
            let blue = self.ega_preview[i] & 0b00000011;
            let terminator = if i < 7 { ',' } else { '}' };
            write!(f, "({}, {}, {}){}", red, green, blue, terminator)?;
        }
        write!(f, "\n\tvga_custom = {{")?;
        for i in 0..8 {
            let red = self.vga_custom[i * 3 + 0];
            let green = self.vga_custom[i * 3 + 1];
            let blue = self.vga_custom[i * 3 + 2];
            let terminator = if i < 7 { ',' } else { '}' };
            write!(f, "({}, {}, {}){}", red, green, blue, terminator)?;
        }
        write!(f, "\n\tvga_standard = {{")?;
        for i in 0..8 {
            let red = self.vga_standard[i * 3 + 0];
            let green = self.vga_standard[i * 3 + 1];
            let blue = self.vga_standard[i * 3 + 2];
            let terminator = if i < 7 { ',' } else { '}' };
            write!(f, "({}, {}, {}){}", red, green, blue, terminator)?;
        }
        write!(f, "\n\tvga_preview = {{")?;
        for i in 0..8 {
            let red = self.vga_preview[i * 3 + 0];
            let green = self.vga_preview[i * 3 + 1];
            let blue = self.vga_preview[i * 3 + 2];
            let terminator = if i < 7 { ',' } else { '}' };
            write!(f, "({}, {}, {}){}", red, green, blue, terminator)?;
        }
        write!(f, "\n}}\n")
    }
}

pub struct ExtractOptions<'a> {
    pub terrain_filename_pattern : &'a str,
    pub terrain_mask_filename_pattern: Option<&'a str>,
    pub object_filename_pattern : &'a str,
    pub object_mask_filename_pattern : Option<&'a str>,
    pub ega_mode : bool,
}

impl<'a> Default for ExtractOptions<'a> {
    fn default() -> Self {
        ExtractOptions {
            terrain_filename_pattern : "terrain#.bmp",
            terrain_mask_filename_pattern : None,
            object_filename_pattern : "obj#.bmp",
            object_mask_filename_pattern : None,
            ega_mode : false,
        }
    }
}

/// Extract a graphics set
pub fn extract_graphics_set(script : &mut dyn std::io::Write, header_file : &mut dyn std::io::Read, data_file : &mut dyn std::io::Read, options : &ExtractOptions) {

    let mut terrain_section = DatSection::from_file(data_file).unwrap();
    let terrain_data = terrain_section.decompress();
    let mut object_section = DatSection::from_file(data_file).unwrap();
    let object_data = object_section.decompress();

    let mut obj_headers = Vec::<ObjectHeader>::new();
    for i in 0..16 {
        obj_headers.push(ObjectHeader::read(header_file).unwrap());
        println!("Object {}: {:?}", i, obj_headers[i]);
    }

    let mut terrain_headers = Vec::<TerrainHeader>::new();
    for i in 0..64 {
        terrain_headers.push(TerrainHeader::read(header_file).unwrap());
        println!("Terrain {}: {:?}", i, terrain_headers[i]);
    }

    let all_pals = Palettes::read(header_file).unwrap();
    let mut pal = planar_bmp::PaletteRGB::new(16);
    if options.ega_mode {
        pal.set_ega_data(0, 8, all_pals.ega_standard.as_slice());
        pal.set_ega_data(8, 8, all_pals.ega_custom.as_slice());
    } else {
        pal.set_vga_data(0, 8, all_pals.vga_standard.as_slice());
        pal.set_vga_data(8, 8, all_pals.vga_custom.as_slice());
    }

    for i in 0..64 {
        let terrain_header = &terrain_headers[i];
        if terrain_header.width == 0 { break; }
        let outfile_name = options.terrain_filename_pattern.replace("#", &i.to_string());
        let plane_size = terrain_header.width as usize * terrain_header.height as usize / 8;
        let image_size = plane_size * 4;
        let terrain_image = planar_bmp::PlanarBMP::from_contiguous_data(&terrain_data[terrain_header.gfx_offset as usize..(terrain_header.gfx_offset as usize + image_size)], terrain_header.width as usize, terrain_header.height as usize, 4, &pal);
        let mask_image_1bpp = planar_bmp::PlanarBMP::from_contiguous_data(&terrain_data[terrain_header.mask_offset as usize..(terrain_header.mask_offset as usize + plane_size)], terrain_header.width as usize, terrain_header.height as usize, 1, &pal);
        if options.terrain_mask_filename_pattern == None {
            // Combine the mask and image into one
            let mut output_image = planar_bmp::PlanarBMP::new(terrain_header.width as usize * 2, terrain_header.height as usize, 4, &pal);
            let mask_image_4bpp = planar_bmp::PlanarBMP::from_swizzle(&mask_image_1bpp, vec![0, 0, 0, 0]);
            output_image.blit(&terrain_image, 0, 0);
            output_image.blit(&mask_image_4bpp, terrain_header.width as usize, 0);
            let out_path = Path::new(outfile_name.as_str());
            let mut output_file = File::create(out_path).unwrap();
            output_image.save_as_file(&mut output_file);
            write!(script, "Terrain \"{}\"\n", outfile_name).unwrap();
        }
        else {
            let maskfile_name = options.terrain_mask_filename_pattern.unwrap().replace("#", &i.to_string());
            let out_path = Path::new(outfile_name.as_str());
            let mut output_file = File::create(out_path).unwrap();
            let mask_path = Path::new(maskfile_name.as_str());
            let mut mask_file = File::create(mask_path).unwrap();
            terrain_image.save_as_file(&mut output_file);
            mask_image_1bpp.save_as_file(&mut mask_file);
            write!(script, "Terrain \"{}\" Mask \"{}\"\n", outfile_name, maskfile_name).unwrap();
        }
    }

    for i in 0..16 {
        let obj_header = &obj_headers[i];
        if obj_header.width == 0 { break; }
        let filmstrip_width = obj_header.width as usize * if options.object_mask_filename_pattern.is_none() { 2 } else { 1 };
        let mut filmstrip_image = planar_bmp::PlanarBMP::new(filmstrip_width, obj_header.height as usize * obj_header.frame_end as usize, 4, &pal);
        let outfile_name = options.object_filename_pattern.replace("#", &i.to_string());
        let out_path = Path::new(outfile_name.as_str());

        let mask_fname = if options.object_mask_filename_pattern.is_some() { Some(options.object_mask_filename_pattern.unwrap().replace("#", &i.to_string())) } else { None };
        let mut mask_bmp = if mask_fname.is_some() {
            write!(script, "Object \"{}\" Mask \"{}\" = {}\n", outfile_name, mask_fname.as_ref().unwrap(), obj_headers[i]).unwrap();
            Some(planar_bmp::PlanarBMP::new(filmstrip_width, obj_header.height as usize * obj_header.frame_end as usize, 1, &pal))
        } else {
            write!(script, "Object \"{}\" = {}\n", outfile_name, obj_headers[i]).unwrap();
            None
        };

        for frame in 0..obj_header.frame_end {
            let frame_size = obj_header.animation_frame_data_size as usize;
            let plane_len = obj_header.width as usize * obj_header.height as usize / 8;
            //assert_eq!(planar_size, mask_len * 4);
            let frame_offset = obj_header.animation_offset as usize + frame_size * frame as usize;
            let mask_offset = frame_offset + obj_header.mask_offset as usize;
            let object_image = planar_bmp::PlanarBMP::from_contiguous_data(&object_data[frame_offset..(frame_offset + plane_len * 4)], obj_header.width as usize, obj_header.height as usize, 4, &pal);
            let object_mask_1bpp = planar_bmp::PlanarBMP::from_contiguous_data(&object_data[mask_offset..(mask_offset + plane_len)], obj_header.width as usize, obj_header.height as usize, 1, &pal);
            filmstrip_image.blit(&object_image, 0, frame as usize * obj_header.height as usize);
            if mask_bmp.is_some() {
                // Write the mask to a separate file.
                mask_bmp.as_mut().unwrap().blit(&object_mask_1bpp, 0, frame as usize * obj_header.height as usize);
            } else {
                // Put it in the filmstrip image.
                let object_mask_4bpp = planar_bmp::PlanarBMP::from_swizzle(&object_mask_1bpp, vec![0, 0, 0, 0]);
                filmstrip_image.blit(&object_mask_4bpp, obj_header.width as usize, frame as usize * obj_header.height as usize);
            }
        }
        let mut output_file = File::create(out_path).unwrap();
        filmstrip_image.save_as_file(&mut output_file);
        if mask_bmp.is_some() {
            let mask_path = Path::new(mask_fname.as_ref().unwrap().as_str());
            let mut mask_file = File::create(mask_path).unwrap();
            mask_bmp.unwrap().save_as_file(&mut mask_file);
        }
    }

    write!(script, "Palettes = {}\n", &all_pals).unwrap();
}

pub fn create_graphics_set(lexer : &mut parser::Lexer) {
    lexer.expect_ident("HeaderFile");
    let header_filename = lexer.get_string_literal();

    lexer.expect_ident("DataFile");
    let data_filename = lexer.get_string_literal();

    let mut object_headers = Vec::<ObjectHeader>::new();
    let mut terrain_headers = Vec::<TerrainHeader>::new();

    let mut terrain_data = Vec::<u8>::new();
    let mut object_data = Vec::<u8>::new();

    let mut pal = Palettes::default();

    loop {
        let entry_type = lexer.next_token();
        match entry_type {
            None => { break; }
            Some(parser::Token::Ident("Terrain")) => {
                let terrain_fname = lexer.get_string_literal();
                let mut terrain_file = std::fs::File::open(&terrain_fname).unwrap();
                let mask_fname = if lexer.is_next_ident("Mask") {
                    lexer.next_token(); // Discard the keyword.
                    Some(lexer.get_string_literal())
                } else { None };
                let terrain_bmp = planar_bmp::PlanarBMP::from_file(&mut terrain_file).unwrap();
                let terrain_offset = terrain_data.len();
                let terrain_width = if mask_fname.is_some() { terrain_bmp.width } else { terrain_bmp.width / 2 }; // Make room for the mask.

                terrain_data.append(&mut terrain_bmp.get_plane_data(0, 0, 0, terrain_width, terrain_bmp.height));
                terrain_data.append(&mut terrain_bmp.get_plane_data(1, 0, 0, terrain_width, terrain_bmp.height));
                terrain_data.append(&mut terrain_bmp.get_plane_data(2, 0, 0, terrain_width, terrain_bmp.height));
                terrain_data.append(&mut terrain_bmp.get_plane_data(3, 0, 0, terrain_width, terrain_bmp.height));

                let mask_offset = terrain_data.len();
                if mask_fname.is_none() {
                    // Extract the mask from the same bitmap.
                    terrain_data.append(&mut terrain_bmp.get_plane_data(0, terrain_width, 0, terrain_width, terrain_bmp.height));
                } else {
                    let mut mask_file = std::fs::File::open(mask_fname.unwrap()).unwrap();
                    let mask_bmp = planar_bmp::PlanarBMP::from_file(&mut mask_file).unwrap();
                    assert_eq!(terrain_width, mask_bmp.width);
                    assert_eq!(terrain_bmp.height, mask_bmp.height);
                    terrain_data.append(&mut mask_bmp.get_plane_data(0, 0, 0, mask_bmp.width, mask_bmp.height));
                }

                terrain_headers.push(TerrainHeader {
                    width: terrain_bmp.width as u8,
                    height: terrain_bmp.height as u8,
                    gfx_offset: terrain_offset as u16,
                    mask_offset: mask_offset as u16,
                    _unknown1: 0 }
                );

            }
            Some(parser::Token::Ident("Object")) => {
                let object_fname = lexer.get_string_literal();
                let mask_fname = if lexer.is_next_ident("Mask") {
                    lexer.next_token(); // Discard the keyword.
                    Some(lexer.get_string_literal())
                } else { None };
                let mut object_file = std::fs::File::open(object_fname).unwrap();
                let object_bmp = planar_bmp::PlanarBMP::from_file(&mut object_file).unwrap();
                let object_width = if mask_fname.is_none() { object_bmp.width / 2 } else {object_bmp.width };

                // Open a separate mask .bmp if one exists
                let mask_bmp = if mask_fname.is_some() {
                    let mut mask_file = std::fs::File::open(mask_fname.unwrap()).unwrap();
                    let mask_bmp = planar_bmp::PlanarBMP::from_file(&mut mask_file).unwrap();
                    assert_eq!(mask_bmp.width, object_bmp.width);
                    assert_eq!(mask_bmp.height, object_bmp.height);
                    Some(mask_bmp)
                } else { None };

                // Get the info.
                lexer.expect_symbol('=');
                let mut object_header = ObjectHeader::parse(lexer);

                let frame_height = object_bmp.height / object_header.frame_end as usize;
                object_header.animation_offset = object_data.len() as u16;
                object_header.width = object_width as u8;
                object_header.height = frame_height as u8;
                object_header.animation_frame_data_size = ((object_bmp.width * frame_height / 8) * 5) as u16;
                object_header.preview_frame_offset = object_header.animation_offset + (object_header.animation_frame_data_size * object_header.preview_frame_number as u16);

                for frame in 0..object_header.frame_end as usize {
                    object_data.append(&mut object_bmp.get_plane_data(0, 0, frame * frame_height, object_width, frame_height));
                    object_data.append(&mut object_bmp.get_plane_data(1, 0, frame * frame_height, object_width, frame_height));
                    object_data.append(&mut object_bmp.get_plane_data(2, 0, frame * frame_height, object_width, frame_height));
                    object_data.append(&mut object_bmp.get_plane_data(3, 0, frame * frame_height, object_width, frame_height));
                    if mask_bmp.is_none() {
                        // Grab the mask from the main .bmp
                        object_data.append(&mut object_bmp.get_plane_data(0, object_width, frame * frame_height, object_width, frame_height));
                    } else {
                        // Grab it from the mask .bmp
                        object_data.append(&mut mask_bmp.as_ref().unwrap().get_plane_data(0, 0, frame * frame_height, object_width, frame_height));
                    }
                }

                object_header.mask_offset = (0 * object_bmp.width * frame_height / 2) as u16;


                object_headers.push(object_header);
            }
            Some(parser::Token::Ident("Palettes")) => {
                lexer.expect_symbol('=');
                pal = Palettes::parse(lexer);
            }
            _ => {
                panic!("Unknown token {:?}", entry_type);
            }

        }
    }

    // Now we've finished parsing the script, compress the data.
    let terrain_section = DatSection::from_data(&terrain_data[..], terrain_data.len());
    let object_section = DatSection::from_data(&object_data[..], object_data.len());

    // Open the output file
    let data_path = Path::new(&data_filename);

    let mut data = match File::create(data_path) {
        Err(err) => panic!("Error opening {}: {}", data_filename, err),
        Ok(file) => file,
    };

    terrain_section.write(&mut data).unwrap();
    object_section.write(&mut data).unwrap();
    // TODO: Palette section

    // Now write out the headers
    let header_path = Path::new(&header_filename);
    let mut header = match File::create(header_path) {
        Err(err) => panic!("Error opening {}: {}", header_filename, err),
        Ok(file) => file,
    };

    let null_object_header = ObjectHeader::default();
    for i in 0..16 {
        let object_header = if i < object_headers.len() { &object_headers[i] } else { &null_object_header };
        object_header.write(&mut header).unwrap();
    }

    let null_terrain_header = TerrainHeader::default();
    for i in 0..64 {
        let terrain_header = if i < terrain_headers.len() { &terrain_headers[i] } else { &null_terrain_header };
        terrain_header.write(&mut header).unwrap();
    }

    // Now the palette
    pal.write(&mut header);
}
