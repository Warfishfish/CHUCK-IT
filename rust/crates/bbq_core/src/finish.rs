//! Shared art direction. Hex sRGB values; preserve palette indices in saved appearances.
/// 0 disables added surface detail, 1 subtle (default), 2 stronger. No gameplay effect.
pub const DETAIL_LEVEL:u8=1;
pub struct Palette {pub body:[u32;10],pub ink:u32,pub paper:u32,pub sun:u32,pub blue:u32,pub red:u32,pub lawn:u32,pub wood:u32}
pub const PALETTE:Palette=Palette {
 body:[0xf5cc45,0xe99532,0xd95c43,0xd88daf,0x956fcb,0x408cce,0x37a9b7,0xc4ce78,0xb88352,0x8795aa],
 ink:0x26352f,paper:0xf5f3e4,sun:0xf5cc45,blue:0x408cce,red:0xd95c43,lawn:0x709345,wood:0xa77a4e,
};
pub const GRADIENT_DEPTH:f32=0.10;
pub const BODY_ROUGHNESS:f32=0.66;
pub const BODY_COAT:f32=0.08;
