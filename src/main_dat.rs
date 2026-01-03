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

use dat_section::DatSection;
use planar_bmp::PaletteRGB;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use {case_sensitivity, planar_bmp};

struct LemmingsAnim {
    name: &'static str,
    num_frames: usize,
    width: usize,
    height: usize,
    planes: usize,
}

static LEMMINGS_ANIMS: &[LemmingsAnim] = &[
    LemmingsAnim {
        name: "walk_r",
        num_frames: 8,
        width: 16,
        height: 10,
        planes: 2,
    },
    LemmingsAnim {
        name: "jump_r",
        num_frames: 1,
        width: 16,
        height: 10,
        planes: 2,
    },
    LemmingsAnim {
        name: "walk_l",
        num_frames: 8,
        width: 16,
        height: 10,
        planes: 2,
    },
    LemmingsAnim {
        name: "jump_l",
        num_frames: 1,
        width: 16,
        height: 10,
        planes: 2,
    },
    LemmingsAnim {
        name: "dig",
        num_frames: 16,
        width: 16,
        height: 14,
        planes: 3,
    },
    LemmingsAnim {
        name: "climb_r",
        num_frames: 8,
        width: 16,
        height: 12,
        planes: 2,
    },
    LemmingsAnim {
        name: "climb_l",
        num_frames: 8,
        width: 16,
        height: 12,
        planes: 2,
    },
    LemmingsAnim {
        name: "drown",
        num_frames: 16,
        width: 16,
        height: 10,
        planes: 2,
    },
    LemmingsAnim {
        name: "pullup_r",
        num_frames: 8,
        width: 16,
        height: 12,
        planes: 2,
    },
    LemmingsAnim {
        name: "pullup_l",
        num_frames: 8,
        width: 16,
        height: 12,
        planes: 2,
    },
    LemmingsAnim {
        name: "build_r",
        num_frames: 16,
        width: 16,
        height: 13,
        planes: 3,
    },
    LemmingsAnim {
        name: "build_l",
        num_frames: 16,
        width: 16,
        height: 13,
        planes: 3,
    },
    LemmingsAnim {
        name: "bash_r",
        num_frames: 32,
        width: 16,
        height: 10,
        planes: 3,
    },
    LemmingsAnim {
        name: "bash_l",
        num_frames: 32,
        width: 16,
        height: 10,
        planes: 3,
    },
    LemmingsAnim {
        name: "mine_r",
        num_frames: 24,
        width: 16,
        height: 13,
        planes: 3,
    },
    LemmingsAnim {
        name: "mine_l",
        num_frames: 24,
        width: 16,
        height: 13,
        planes: 3,
    },
    LemmingsAnim {
        name: "fall_r",
        num_frames: 4,
        width: 16,
        height: 10,
        planes: 2,
    },
    LemmingsAnim {
        name: "fall_l",
        num_frames: 4,
        width: 16,
        height: 10,
        planes: 2,
    },
    LemmingsAnim {
        name: "brolly_r",
        num_frames: 4,
        width: 16,
        height: 16,
        planes: 3,
    },
    LemmingsAnim {
        name: "float_r",
        num_frames: 4,
        width: 16,
        height: 16,
        planes: 3,
    },
    LemmingsAnim {
        name: "brolly_l",
        num_frames: 4,
        width: 16,
        height: 16,
        planes: 3,
    },
    LemmingsAnim {
        name: "float_l",
        num_frames: 4,
        width: 16,
        height: 16,
        planes: 3,
    },
    LemmingsAnim {
        name: "splat",
        num_frames: 16,
        width: 16,
        height: 10,
        planes: 2,
    },
    LemmingsAnim {
        name: "exit",
        num_frames: 8,
        width: 16,
        height: 13,
        planes: 2,
    },
    LemmingsAnim {
        name: "fry",
        num_frames: 14,
        width: 16,
        height: 14,
        planes: 4,
    },
    LemmingsAnim {
        name: "block",
        num_frames: 16,
        width: 16,
        height: 10,
        planes: 2,
    },
    LemmingsAnim {
        name: "shrug_r",
        num_frames: 8,
        width: 16,
        height: 10,
        planes: 2,
    },
    LemmingsAnim {
        name: "shrug_l",
        num_frames: 8,
        width: 16,
        height: 10,
        planes: 2,
    },
    LemmingsAnim {
        name: "ohno",
        num_frames: 16,
        width: 16,
        height: 10,
        planes: 2,
    },
    LemmingsAnim {
        name: "boom",
        num_frames: 1,
        width: 32,
        height: 32,
        planes: 3,
    },
];

static LEMMINGS_MASKS: &[LemmingsAnim] = &[
    LemmingsAnim {
        name: "bash_r",
        num_frames: 4,
        width: 16,
        height: 10,
        planes: 1,
    },
    LemmingsAnim {
        name: "bash_l",
        num_frames: 4,
        width: 16,
        height: 10,
        planes: 1,
    },
    LemmingsAnim {
        name: "mine_r",
        num_frames: 2,
        width: 16,
        height: 13,
        planes: 1,
    },
    LemmingsAnim {
        name: "mine_l",
        num_frames: 2,
        width: 16,
        height: 13,
        planes: 1,
    },
    LemmingsAnim {
        name: "bomb",
        num_frames: 1,
        width: 16,
        height: 22,
        planes: 1,
    },
    LemmingsAnim {
        name: "bomb_font",
        num_frames: 10,
        width: 8,
        height: 8,
        planes: 1,
    },
];

static LEMMINGS_INTERFACE_HI: &[LemmingsAnim] = &[
    LemmingsAnim {
        name: "skills_hi",
        num_frames: 1,
        width: 320,
        height: 40,
        planes: 4,
    },
    LemmingsAnim {
        name: "skillcount",
        num_frames: 20,
        width: 8,
        height: 8,
        planes: 1,
    },
    LemmingsAnim {
        name: "font_hi",
        num_frames: 37,
        width: 8,
        height: 16,
        planes: 3,
    },
];

static LEMMINGS_MAINMENU: &[LemmingsAnim] = &[
    LemmingsAnim {
        name: "background",
        num_frames: 1,
        width: 320,
        height: 104,
        planes: 2,
    },
    LemmingsAnim {
        name: "logo",
        num_frames: 1,
        width: 632,
        height: 94,
        planes: 4,
    },
    LemmingsAnim {
        name: "oneplayer",
        num_frames: 1,
        width: 120,
        height: 61,
        planes: 4,
    },
    LemmingsAnim {
        name: "newgame",
        num_frames: 1,
        width: 120,
        height: 61,
        planes: 4,
    },
    LemmingsAnim {
        name: "sndbutton",
        num_frames: 1,
        width: 120,
        height: 61,
        planes: 4,
    },
    LemmingsAnim {
        name: "rating",
        num_frames: 1,
        width: 120,
        height: 61,
        planes: 4,
    },
    LemmingsAnim {
        name: "exittodos",
        num_frames: 1,
        width: 120,
        height: 61,
        planes: 4,
    },
    LemmingsAnim {
        name: "controls",
        num_frames: 1,
        width: 120,
        height: 61,
        planes: 4,
    },
    LemmingsAnim {
        name: "musicon",
        num_frames: 1,
        width: 64,
        height: 31,
        planes: 4,
    },
    LemmingsAnim {
        name: "sfxicon",
        num_frames: 1,
        width: 64,
        height: 31,
        planes: 4,
    },
];

static LEMMINGS_MENUANIM: &[LemmingsAnim] = &[
    LemmingsAnim {
        name: "blink1",
        num_frames: 8,
        width: 32,
        height: 12,
        planes: 4,
    },
    LemmingsAnim {
        name: "blink2",
        num_frames: 8,
        width: 32,
        height: 12,
        planes: 4,
    },
    LemmingsAnim {
        name: "blink3",
        num_frames: 8,
        width: 32,
        height: 12,
        planes: 4,
    },
    LemmingsAnim {
        name: "blink4",
        num_frames: 8,
        width: 32,
        height: 12,
        planes: 4,
    },
    LemmingsAnim {
        name: "blink5",
        num_frames: 8,
        width: 32,
        height: 12,
        planes: 4,
    },
    LemmingsAnim {
        name: "blink6",
        num_frames: 8,
        width: 32,
        height: 12,
        planes: 4,
    },
    LemmingsAnim {
        name: "blink7",
        num_frames: 8,
        width: 32,
        height: 12,
        planes: 4,
    },
    LemmingsAnim {
        name: "scroll_l",
        num_frames: 16,
        width: 48,
        height: 16,
        planes: 4,
    },
    LemmingsAnim {
        name: "scroll_r",
        num_frames: 16,
        width: 48,
        height: 16,
        planes: 4,
    },
    LemmingsAnim {
        name: "reel",
        num_frames: 1,
        width: 16,
        height: 16,
        planes: 4,
    },
    LemmingsAnim {
        name: "difficulty4",
        num_frames: 1,
        width: 72,
        height: 27,
        planes: 4,
    },
    LemmingsAnim {
        name: "difficulty3",
        num_frames: 1,
        width: 72,
        height: 27,
        planes: 4,
    },
    LemmingsAnim {
        name: "difficulty2",
        num_frames: 1,
        width: 72,
        height: 27,
        planes: 4,
    },
    LemmingsAnim {
        name: "difficulty1",
        num_frames: 1,
        width: 72,
        height: 27,
        planes: 4,
    },
    // TODO: Support Oh-no! More Lemmings! here.
    //    LemmingsAnim { name: "difficulty0", num_frames: 1, width: 72, height: 27, planes: 4},
    LemmingsAnim {
        name: "menufont",
        num_frames: 93,
        width: 16,
        height: 16,
        planes: 3,
    },
];

static LEMMINGS_INTERFACE_LO: &[LemmingsAnim] = &[
    LemmingsAnim {
        name: "skills_lo",
        num_frames: 1,
        width: 320,
        height: 40,
        planes: 4,
    },
    LemmingsAnim {
        name: "font_lo",
        num_frames: 37,
        width: 8,
        height: 16,
        planes: 3,
    },
];

fn extract_anims(data: &Vec<u8>, anims: &[LemmingsAnim], name: &str, pal: &PaletteRGB) {
    let mut running_offset: usize = 0;
    for anim in anims {
        let outfile_name = format!("{}_{}.bmp", name, anim.name);
        let mut filmstrip_image = planar_bmp::PlanarBMP::new(
            anim.width,
            anim.height * anim.num_frames,
            anim.planes,
            &pal,
        );
        for frame in 0..anim.num_frames {
            println!("Extracting {} {} frame #{}", name, anim.name, frame);
            let planar_size = anim.width * anim.height / 8;
            let converted_image = planar_bmp::PlanarBMP::from_contiguous_data(
                &data[running_offset..(running_offset + planar_size * anim.planes)],
                anim.width,
                anim.height,
                anim.planes,
                &pal,
            );
            running_offset += planar_size * anim.planes;
            filmstrip_image.blit(&converted_image, 0, frame * anim.height);
        }
        let out_path = Path::new(outfile_name.as_str());
        let mut output_file = File::create(out_path).unwrap();
        filmstrip_image.save_as_file(&mut output_file);
    }
}

pub fn extract_main_dat(image: &mut dyn std::io::Read, xmas_mode: bool) {
    let pal = if xmas_mode {
        planar_bmp::PaletteRGB::from_vga_data(
            16,
            [
                0, 0, 0, 52, 8, 8, 0, 44, 0, 60, 52, 52, 60, 60, 0, 16, 16, 60, 32, 32, 32, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .as_slice(),
        )
    } else {
        planar_bmp::PaletteRGB::from_vga_data(
            16,
            [
                0, 0, 0, 16, 16, 56, 0, 44, 0, 60, 58, 58, 44, 44, 0, 60, 8, 8, 32, 32, 32, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
            .as_slice(),
        )
    };

    //NOTE: Christmas lemmings doesn't seem to have a valid palette for these at all?
    // Or, at least, I couldn't get the High Performance mode to launch for them.
    let hiperf_pal = planar_bmp::PaletteRGB::from_vga_data(
        16,
        [
            0, 0, 0, 16, 16, 56, 0, 44, 0, 60, 58, 58, 44, 44, 0, 60, 8, 8, 32, 32, 32, 0, 0, 0, 0,
            42, 0, 21, 63, 21, 21, 21, 21, 42, 0, 0, 42, 21, 0, 0, 42, 42, 63, 21, 63, 42, 0, 42,
        ]
        .as_slice(),
    );
    let menupal = planar_bmp::PaletteRGB::from_vga_data(
        16,
        [
            0, 0, 0, 32, 16, 8, 24, 12, 8, 12, 0, 4, //8,  0,  4,
            8, 2, 31, 16, 11, 36, 26, 22, 41, 38, 35, 47, 0, 20, 0, 0, 24, 4, 0, 28, 8, 0, 32, 16,
            52, 52, 52, 44, 44, 0, 16, 20, 44, 56, 32, 36,
        ]
        .as_slice(),
    );

    let mut lemming_anim_section = DatSection::from_file(image).unwrap();
    let lemming_anim_data = lemming_anim_section.decompress();

    extract_anims(&lemming_anim_data, &LEMMINGS_ANIMS, "lemming", &pal);

    let mut lemming_mask_section = DatSection::from_file(image).unwrap();
    let lemming_mask_data = lemming_mask_section.decompress();
    extract_anims(&lemming_mask_data, &LEMMINGS_MASKS, "mask", &pal);

    let mut lemming_interface_hi_section = DatSection::from_file(image).unwrap();
    let lemming_interface_hi_data = lemming_interface_hi_section.decompress();
    extract_anims(
        &lemming_interface_hi_data,
        &LEMMINGS_INTERFACE_HI,
        "interface_hi",
        &hiperf_pal,
    );

    let mut lemming_mainmenu_section = DatSection::from_file(image).unwrap();
    let lemming_mainmenu_data = lemming_mainmenu_section.decompress();
    extract_anims(&lemming_mainmenu_data, &LEMMINGS_MAINMENU, "menu", &menupal);

    let mut lemming_menuanim_section = DatSection::from_file(image).unwrap();
    let lemming_menuanim_data = lemming_menuanim_section.decompress();
    extract_anims(
        &lemming_menuanim_data,
        &LEMMINGS_MENUANIM,
        "menuanim",
        &menupal,
    );

    let mut pcspk_sound_section = DatSection::from_file(image).unwrap();
    let pcspk_sound_data = pcspk_sound_section.decompress();

    let mut pcspk_output_file = File::create("pcspkr.snd").unwrap();
    pcspk_output_file
        .write_all(pcspk_sound_data.as_slice())
        .unwrap();

    let mut interface_lo_section = DatSection::from_file(image).unwrap();
    let interface_lo_data = interface_lo_section.decompress();
    extract_anims(
        &interface_lo_data,
        &LEMMINGS_INTERFACE_LO,
        "interface_lo",
        &pal,
    );
}

fn compress_anims(anims: &[LemmingsAnim], name: &str) -> DatSection {
    let mut data = std::vec::Vec::<u8>::new();
    for anim in anims {
        let infile_name = format!("{}_{}.bmp", name, anim.name);
        let infile_path = Path::new(infile_name.as_str());
        let mut infile = match File::open(&infile_path) {
            Err(err) => panic!("Error opening {}: {}", infile_name, err),
            Ok(file) => file,
        };
        let mut running_h = 0;
        let filmstrip_image = planar_bmp::PlanarBMP::from_file(&mut infile).unwrap();
        for _ in 0..anim.num_frames {
            for plane in 0..anim.planes {
                data.append(&mut filmstrip_image.get_plane_data(
                    plane,
                    0,
                    running_h,
                    anim.width,
                    anim.height,
                ));
            }
            running_h += anim.height;
        }
    }
    DatSection::from_data(data.as_slice(), data.len())
}

pub fn create_main_dat() {
    let dat_path = match case_sensitivity::find_file_in_current_dir("main.dat") {
        Ok(path) => path,
        _ => Path::new("main.dat").to_path_buf(),
    };

    let mut data = match File::create(&dat_path) {
        Err(err) => panic!("Error opening main.dat: {}", err),
        Ok(file) => file,
    };

    compress_anims(LEMMINGS_ANIMS, "lemming")
        .write(&mut data)
        .unwrap();
    compress_anims(LEMMINGS_MASKS, "mask")
        .write(&mut data)
        .unwrap();
    compress_anims(LEMMINGS_INTERFACE_HI, "interface_hi")
        .write(&mut data)
        .unwrap();
    compress_anims(LEMMINGS_MAINMENU, "menu")
        .write(&mut data)
        .unwrap();
    compress_anims(LEMMINGS_MENUANIM, "menuanim")
        .write(&mut data)
        .unwrap();

    let pcspk_input_file = Path::new("pcspkr.snd");
    let pcspk_sound_data = std::fs::read(pcspk_input_file).unwrap();

    let pcspk_sound_section =
        DatSection::from_data(pcspk_sound_data.as_slice(), pcspk_sound_data.len());
    pcspk_sound_section.write(&mut data).unwrap();

    compress_anims(LEMMINGS_INTERFACE_LO, "interface_lo")
        .write(&mut data)
        .unwrap();
}
