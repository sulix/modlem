pub use std::{fs::File, io::Write, path::Path, convert::TryInto};

mod planar_bmp;
mod binary_io;

use binary_io::*;

struct DatSection
{
    uncomp_size : u32,
    comp_size : u32,
    /* checksum : u8 is unused. We check for it when loading. */
    num_bits_in_first_byte: u8,
    byte_offset : u32,
    bit_offset: u32,
    comp_data : std::vec::Vec<u8>,
}

#[derive(Debug)]
struct ObjectHeader
{
    animation_flags : u16,
    frame_start : u8,
    frame_end : u8,
    width : u8,
    height : u8,
    animation_frame_data_size : u16,
    mask_offset : u16,
    _unknown0 : u16,
    _unknown1 : u16,
    trigger_x : u16,
    trigger_y : u16,
    trigger_w : u8,
    trigger_h : u8,
    trigger_effect_id : u8,
    animation_offset : u16,
    preview_frame : u16,
    _unknown2 : u16,
    trap_sound : u8,
}

#[derive(Debug)]
struct TerrainHeader
{
    width : u8,
    height : u8,
    gfx_offset : u16,
    mask_offset : u16,
    _unknown1 : u16,
}

#[derive(Debug)]
struct VgaPaletteEntry
{
    red : u8,
    green : u8,
    blue : u8,
}

#[derive(Default)]
struct Palettes
{
    ega_custom : [u8; 8],
    ega_standard : [u8; 8],
    ega_preview : [u8; 8],
//    vga_custom : [VgaPaletteEntry; 8],
//    vga_standard : [VgaPaletteEntry; 8],
//    vga_preview : [VgaPaletteEntry; 8],
    vga_custom : [u8; 24],
    vga_standard : [u8; 24],
    vga_preview : [u8; 24],
}


fn read_object_header(reader : &mut dyn std::io::Read) -> std::io::Result<ObjectHeader> {
    Ok(ObjectHeader {
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
        preview_frame : read_le16(reader)?,
        _unknown2 : read_le16(reader)?,
        trap_sound : read_byte(reader)?,
    })
}

fn read_terrain_header(reader : &mut dyn std::io::Read) -> std::io::Result<TerrainHeader> {
    Ok(TerrainHeader {
        width : read_byte(reader)?,
        height : read_byte(reader)?,
        gfx_offset : read_le16(reader)?,
        mask_offset : read_le16(reader)?,
        _unknown1 : read_le16(reader)?,
    })
}

fn read_palettes(reader : &mut dyn std::io::Read) -> Palettes {
    let mut raw_pal = [0 as u8; 32 * 3];
    reader.read_exact(&mut raw_pal).unwrap();
    Palettes {
        ega_custom : raw_pal[0..8].try_into().unwrap(),
        ega_standard : raw_pal[8..16].try_into().unwrap(),
        ega_preview : raw_pal[16..24].try_into().unwrap(),
        vga_custom : raw_pal[24..48].try_into().unwrap(),
        vga_standard : raw_pal[48..72].try_into().unwrap(),
        vga_preview : raw_pal[72..96].try_into().unwrap(),
    }
}


fn read_dat_section(reader : &mut dyn std::io::Read) -> std::io::Result<DatSection> {
    let num_bits_in_first_byte = read_byte(reader)?;
    let checksum = read_byte(reader)?;
    let uncomp_size = read_be32(reader)?;
    let comp_size = read_be32(reader)?;

    println!("Uncomp {}, Comp {}, Chksum {}, Bits in first byte {}\n", uncomp_size, comp_size, checksum, num_bits_in_first_byte);

    let mut comp_data = std::vec::Vec::<u8>::new();
    comp_data.resize((comp_size - 10) as usize, 0);
    reader.read_exact(&mut comp_data)?;
    let mut data_checksum = 0;
    for b in &comp_data {
        data_checksum ^= b;
    }
    if data_checksum != checksum {
        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "checksum invalid"))
    } else {
        Ok(DatSection {
            uncomp_size,
            comp_size : if num_bits_in_first_byte == 0 { comp_size-1 } else { comp_size },
            /* checksum : checksum, */
            num_bits_in_first_byte: if num_bits_in_first_byte == 0 { 8 } else { num_bits_in_first_byte },
            byte_offset: comp_size - if num_bits_in_first_byte == 0 { 12 } else { 11 } as u32,
            bit_offset : 0 as u32,
            comp_data
        })
    }
}

fn dat_read_bits(dat_section : &mut DatSection, bits: u32) -> u32 {
    let mut val : u32 = 0;
    for _n in 0..bits {
        let cur_byte = dat_section.comp_data[dat_section.byte_offset as usize];
        let bit = if ((1 << dat_section.bit_offset) & cur_byte) != 0 { 1 } else { 0 };
        let bits_in_byte = if (dat_section.byte_offset == dat_section.comp_size - 11) && (dat_section.num_bits_in_first_byte != 0) { dat_section.num_bits_in_first_byte as u32 } else { 8 };
        dat_section.bit_offset += 1;
        if dat_section.bit_offset >= bits_in_byte {
            if dat_section.byte_offset != 0 {
                dat_section.byte_offset -= 1;
            }
            dat_section.bit_offset = 0;
        }
        val = (val << 1) | bit;
    }
    val
}

fn decompress(dat_section : &mut DatSection) -> std::vec::Vec<u8> {
    let mut output : std::vec::Vec::<u8> = vec![0; dat_section.uncomp_size as usize];
    let mut i = (dat_section.uncomp_size - 1) as usize;

    while i > 0 {
        match dat_read_bits(dat_section, 1) {
            0 => {
                // Commands starting with '0' are two bits.
                match dat_read_bits(dat_section, 1) {
                    0 => {
                        // Raw bytes.
                        let len = dat_read_bits(dat_section, 3) + 1;
                        println!("Raw Byte Copy, length {}", len);
                        //println!("{}\t{:x} {}",i,output[i], output[i] as char);
                        i = i + 1;
                        for _b in 0..len {
                            i = i - 1;
                            output[i] = dat_read_bits(dat_section, 8) as u8;
                            //println!("{}\t{:x} {}",i,output[i], output[i] as char);
                        }
                    }
                    1 => {
                        // Two-byte reference
                        let raw_off = dat_read_bits(dat_section, 8);
                        let off = i + 1 + raw_off as usize;
                        println!("Two Byte Match, offset {} (raw: {})", off, raw_off);
                        output[i] = output[off];
                        i -= 1;
                        output[i] = output[off-1];
                    }
                    _ => {
                        panic!("Another bit which isn't 0 or 1");
                    }
                }
            }
            1 => {
                let cmd = dat_read_bits(dat_section, 2);
                //println!("cmd1 = {}", cmd);
                match cmd {
                    0 => {
                        let raw_off = dat_read_bits(dat_section, 9);
                        let off = i + 1 + raw_off as usize;
                        println!("Three Byte Match, offset {} (raw: {})", off, raw_off);
                        output[i] = output[off];
                        i -= 1;
                        output[i] = output[off-1];
                        i -= 1;
                        output[i] = output[off-2];
                    }
                    1 => {
                        let off = i + 1 + dat_read_bits(dat_section, 10) as usize;
                        println!("Four byte Match, offset {}", off);
                        output[i] = output[off];
                        i -= 1;
                        output[i] = output[off-1];
                        i -= 1;
                        output[i] = output[off-2];
                        i -= 1;
                        output[i] = output[off-3];
                    }
                    2 => {
                        let len = dat_read_bits(dat_section, 8) + 1;
                        let off = dat_read_bits(dat_section, 12) as usize;
                        println!("{} Byte Match, offset {}", len, off);
                        i += 1;
                        for _b in 0..len {
                            i -= 1;
                            output[i] = output[i + off + 1];
                        }
                    }
                    3 => {
                        let len = dat_read_bits(dat_section, 8) + 9;
                        println!("Raw Byte Copy, length {}", len);
                        i += 1;
                        for _b in 0..len {
                            i -= 1;
                            output[i] = dat_read_bits(dat_section, 8) as u8;
                            //println!("{}\t{:x} {}",i,output[i], output[i] as char);
                        }
                    }
                    _ => {
                        panic!("Bad 3-bit command {}", cmd);
                    }
                }
            }
            _ => {
                panic!("Looks like we've got a bit whose value is not 0 or 1!");
            }
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    output
}


fn pal8_to_rgb(pal8_data: Vec<u8>, pal: &Palettes) -> Vec<u8> {
    let mut rgb_data = Vec::<u8>::new();
    for i in pal8_data {
        let idx = i as usize;
        let r = if i < 8 {
            pal.vga_standard[idx * 3]
        } else {
            pal.vga_custom[(idx - 8) * 3]
        };
        let g = if i < 8 {
            pal.vga_standard[idx * 3 + 1]
        } else {
            pal.vga_custom[(idx - 8) * 3 + 1]
        };
        let b = if i < 8 {
            pal.vga_standard[idx * 3 + 2]
        } else {
            pal.vga_custom[(idx - 8) * 3 + 2]
        };

        rgb_data.push(b << 2);
        rgb_data.push(g << 2);
        rgb_data.push(r << 2);
        rgb_data.push(0);
    }
    rgb_data
}

fn extract_graphics_set(graphics_set : usize) {
    let ground_filename = format!("ground{}o.dat", graphics_set);
    let ground_path = Path::new(ground_filename.as_str());
    let mut ground_header_file = match File ::open(&ground_path) {
        Err(err) => panic!("Error opening ground header file: {}|", err),
        Ok(file) => file,
    };

    let data_filename = format!("vgagr{}.dat", graphics_set);
    let path = Path::new(data_filename.as_str());
    let path_name = path.display();

    let mut image = match File::open(&path) {
        Err(err) => panic!("Error opening lemmings file {}: {}", path_name, err),
        Ok(file) => file,
    };

    let mut terrain_section = read_dat_section(&mut image).unwrap();
    let terrain_data = decompress(&mut terrain_section);
    let mut object_section = read_dat_section(&mut image).unwrap();
    let object_data = decompress(&mut object_section);

    let mut obj_headers = Vec::<ObjectHeader>::new();
    for i in 0..16 {
        obj_headers.push(read_object_header(&mut ground_header_file).unwrap());
        println!("Object {}: {:?}", i, obj_headers[i]);
    }

    let mut terrain_headers = Vec::<TerrainHeader>::new();
    for i in 0..64 {
        terrain_headers.push(read_terrain_header(&mut ground_header_file).unwrap());
        println!("Terrain {}: {:?}", i, terrain_headers[i]);
    }

    let pal = read_palettes(&mut ground_header_file);

    for i in 0..64 {
        let pal16 = pal8_to_rgb(vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15], &pal);
        let terrain_header = &terrain_headers[i];
        if terrain_header.width == 0 { break; }
        let outfile_name = format!("terrain{}.bmp", i);
        let planar_size = terrain_header.width as usize * terrain_header.height as usize / 2;
        let terrain_image = planar_bmp::PlanarBMP::from_contiguous_data(&terrain_data[terrain_header.gfx_offset as usize..(terrain_header.gfx_offset as usize + planar_size)], terrain_header.width as usize, terrain_header.height as usize, 4, pal16);
        let out_path = Path::new(outfile_name.as_str());
        let mut output_file = File::create(out_path).unwrap();
        terrain_image.save_as_pal8(&mut output_file);
    }

    for i in 0..16 {
        let obj_header = &obj_headers[i];
        if obj_header.width == 0 { break; }
        for frame in obj_header.frame_start..obj_header.frame_end {
            let pal16 = pal8_to_rgb(vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15], &pal);
            let outfile_name = format!("obj{}_frame{}.bmp", i, frame);
            let planar_size = obj_header.animation_frame_data_size as usize;
            let frame_offset = obj_header.animation_offset as usize + planar_size * frame as usize;
            let object_image = planar_bmp::PlanarBMP::from_contiguous_data(&object_data[frame_offset..(frame_offset+ planar_size)], obj_header.width as usize, obj_header.height as usize, 4, pal16);
            let out_path = Path::new(outfile_name.as_str());
            let mut output_file = File::create(out_path).unwrap();
            object_image.save_as_pal8(&mut output_file);

        }
    }
}

struct LemmingsAnim {
    name : &'static str,
    num_frames : usize,
    width : usize,
    height : usize,
    planes : usize,
}

static LEMMINGS_ANIMS : &[LemmingsAnim] = &[
    LemmingsAnim { name: "walk_r", num_frames: 8, width: 16, height: 10, planes: 2 },
    LemmingsAnim { name: "jump_r", num_frames: 1, width: 16, height: 10, planes: 2 },
    LemmingsAnim { name: "walk_l", num_frames: 8, width: 16, height: 10, planes: 2 },
    LemmingsAnim { name: "jump_l", num_frames: 1, width: 16, height: 10, planes: 2 },
    LemmingsAnim { name: "dig", num_frames: 16, width: 16, height: 14, planes: 3 },
    LemmingsAnim { name: "climb_r", num_frames: 8, width: 16, height: 12, planes: 2 },
    LemmingsAnim { name: "climb_l", num_frames: 8, width: 16, height: 12, planes: 2 },
    LemmingsAnim { name: "drown", num_frames: 16, width: 16, height: 10, planes: 2 },
    LemmingsAnim { name: "pullup_r", num_frames: 8, width: 16, height: 12, planes: 2 },
    LemmingsAnim { name: "pullup_l", num_frames: 8, width: 16, height: 12, planes: 2 },
    LemmingsAnim { name: "build_r", num_frames: 16, width: 16, height: 13, planes: 3 },
    LemmingsAnim { name: "build_l", num_frames: 16, width: 16, height: 13, planes: 3 },
    LemmingsAnim { name: "bash_r", num_frames: 32, width: 16, height: 10, planes: 3 },
    LemmingsAnim { name: "bash_l", num_frames: 32, width: 16, height: 10, planes: 3 },
    LemmingsAnim { name: "mine_r", num_frames: 24, width: 16, height: 13, planes: 3 },
    LemmingsAnim { name: "mine_l", num_frames: 24, width: 16, height: 13, planes: 3 },
    LemmingsAnim { name: "fall_r", num_frames: 4, width: 16, height: 10, planes: 2 },
    LemmingsAnim { name: "fall_l", num_frames: 4, width: 16, height: 10, planes: 2 },
    LemmingsAnim { name: "brolly_r", num_frames: 4, width: 16, height: 16, planes: 3 },
    LemmingsAnim { name: "float_r", num_frames: 4, width: 16, height: 16, planes: 3 },
    LemmingsAnim { name: "brolly_l", num_frames: 4, width: 16, height: 16, planes: 3 },
    LemmingsAnim { name: "float_l", num_frames: 4, width: 16, height: 16, planes: 3 },
    LemmingsAnim { name: "splat", num_frames: 16, width: 16, height: 10, planes: 2 },
    LemmingsAnim { name: "exit", num_frames: 8, width: 16, height: 13, planes: 2 },


];

static LEMMINGS_MASKS : &[LemmingsAnim] = &[
    LemmingsAnim { name: "bash_r", num_frames: 4, width: 16, height: 10, planes: 1 },
    LemmingsAnim { name: "bash_l", num_frames: 4, width: 16, height: 10, planes: 1 },
    LemmingsAnim { name: "mine_r", num_frames: 2, width: 16, height: 13, planes: 1 },
    LemmingsAnim { name: "mine_l", num_frames: 2, width: 16, height: 13, planes: 1 },
    LemmingsAnim { name: "bomb", num_frames: 1, width: 16, height: 22, planes: 1 },
    LemmingsAnim { name: "bomb_font", num_frames: 10, width: 8, height: 8, planes: 1 },
];

fn extract_main_dat() {
    let mut pal = Palettes::default();
    /*pal.vga_standard[1] = VgaPaletteEntry{red: 32, green: 16, blue: 8};
    pal.vga_standard[2] = VgaPaletteEntry{red: 24, green: 12, blue: 8};
    pal.vga_standard[2] = VgaPaletteEntry{red: 12, green: 0, blue: 4};*/
    pal.vga_standard = [ 0,  0, 0,
                        16, 16, 56,
                         0, 44, 0,
                        60, 58, 58,
                        44, 44, 0,
                        60,  8, 8,
                        32, 32, 32,
                        0, 0, 0];


    let path = Path::new("main.dat");

    let mut image = match File::open(&path) {
        Err(err) => panic!("Error opening main.dat: {}", err),
        Ok(file) => file,
    };

    let mut lemming_anim_section = read_dat_section(&mut image).unwrap();
    let lemming_anim_data = decompress(&mut lemming_anim_section);


    let mut running_offset : usize = 0;
    for anim in LEMMINGS_ANIMS {
        let pal2 = pal8_to_rgb(vec![0,1,2,3], &pal);
        let pal3 = pal8_to_rgb(vec![0,1,2,3,4,5,6,7], &pal);
        let outfile_name = format!("lemming_{}.bmp", anim.name);
        let mut filmstrip_image = planar_bmp::PlanarBMP::new(anim.width, anim.height * anim.num_frames, anim.planes, if anim.planes == 3 { pal3 } else { pal2 });
        for frame in 0..anim.num_frames {
            println!("Extracting lemming {} frame #{}", anim.name, frame);
            let pal2 = pal8_to_rgb(vec![0,1,2,3], &pal);
            let pal3 = pal8_to_rgb(vec![0,1,2,3,4,5,6,7], &pal);
            let planar_size = anim.width * anim.height / 8;
            let converted_image = planar_bmp::PlanarBMP::from_contiguous_data(&lemming_anim_data[running_offset..(running_offset + planar_size*anim.planes)], anim.width, anim.height, anim.planes, if anim.planes == 3 { pal3 } else { pal2 });
            running_offset += planar_size * anim.planes;
            filmstrip_image.blit(&converted_image, 0, frame * anim.height);

        }
        let out_path = Path::new(outfile_name.as_str());
        let mut output_file = File::create(out_path).unwrap();
        filmstrip_image.save_as_pal8(&mut output_file);
    }

    let mut lemming_mask_section = read_dat_section(&mut image).unwrap();
    let lemming_mask_data = decompress(&mut lemming_mask_section);
    running_offset = 0;


    for anim in LEMMINGS_MASKS {
        let pal_1bpp = pal8_to_rgb(vec![0,3], &pal);
        let outfile_name = format!("mask_{}.bmp", anim.name);
        let mut filmstrip_image = planar_bmp::PlanarBMP::new(anim.width, anim.height * anim.num_frames, anim.planes, pal_1bpp);
        for frame in 0..anim.num_frames {
            println!("Extracting mask {} frame #{}", anim.name, frame);
            let pal_1bpp = pal8_to_rgb(vec![0,1], &pal);
            let planar_size = anim.width * anim.height / 8;
            let converted_image = planar_bmp::PlanarBMP::from_contiguous_data(&lemming_mask_data[running_offset..(running_offset + planar_size*anim.planes)], anim.width, anim.height, anim.planes, pal_1bpp);
            running_offset += planar_size * anim.planes;
            filmstrip_image.blit(&converted_image, 0, frame * anim.height);

        }
        let out_path = Path::new(outfile_name.as_str());
        let mut output_file = File::create(out_path).unwrap();
        filmstrip_image.save_as_pal8(&mut output_file);
    }

}



fn main() {
    let args : Vec<std::string::String> = std::env::args().collect(); /* Skip the application name. */

    let command_name = &args[1];

    match command_name.as_str() {
        "extract-set" => {
            let set_num = args[2].parse::<usize>().unwrap();
            println!("Extracting graphics set {}…", set_num);
            extract_graphics_set(set_num);
        }
        "extract-main" => {
            extract_main_dat();
        }
        invalid_cmd => {
            panic!("Unknown command \"{}\"", invalid_cmd);
        }
    }
}
