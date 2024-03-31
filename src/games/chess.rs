#![allow(arithmetic_overflow)]

use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChessPos {
    pub pawn: u64,
    pub ortho: u64,
    pub diag: u64,
    pub own: u64,
    pub other: u64,
    pub squares: u64,
    pub half_move: u64,
    pub full_move: u64,
}

#[derive(Clone, Debug)]
struct SMagic {
    pub attack_table: Vec<u64>,
    pub mask: u64,
    pub magic: u64,
    pub shift: u64,
}

pub struct ChessHandler {
    bishop_magics: [SMagic; 64],
    rook_magics: [SMagic; 64],
}

const KINGSIDE_CASTLE_CLEARANCE_MASK: u64 = 0x60;
const KINGSIDE_CASTLE_CHECK_MASK: u64 = 0x70;
const QUEENSIDE_CASTLE_CLEARANCE_MASK: u64 = 0x0e;
const QUEENSIDE_CASTLE_CHECK_MASK: u64 = 0x1c;

const FLAG_NONE: u64 = 0;
const FLAG_PROMOTE: u64 = 1;
const FLAG_CASTLE: u64 = 2;
const FLAG_ENPASSANT: u64 = 3;

const PRMT_QUEEN: u64 = 0;
const PRMT_ROOK: u64 = 1;
const PRMT_BISHOP: u64 = 2;

const NO_EN_PASSANT: u64 = 64;

const FILE_A: u64 = 0x0101010101010101u64;
const RANK_1: u64 = 0xffu64;
const MAJOR_DIAG: u64 = 0x8040201008040201u64;
const MINOR_DIAG: u64 = 0x0102040810204080u64;
const EDGES: u64 = FILE_A | (FILE_A << 7) | RANK_1 | (RANK_1 << 56);
const LOG_2_DE_BRUIJN: u64 = 0x218a392cd3d5dbf;
const LOG_2_TABLE: [u64; 64] = [
    0, 1, 2, 7, 3, 13, 8, 19, 4, 25, 14, 28, 9, 34, 20, 40, 5, 17, 26, 38, 15, 46, 29, 48, 10, 31,
    35, 54, 21, 50, 41, 57, 63, 6, 12, 18, 24, 27, 33, 39, 16, 37, 45, 47, 30, 53, 49, 56, 62, 11,
    23, 32, 36, 44, 52, 55, 61, 22, 43, 51, 60, 42, 59, 58,
];

const PAWN_ATTACKS: [u64; 64] = [
    0x200,
    0x500,
    0xa00,
    0x1400,
    0x2800,
    0x5000,
    0xa000,
    0x4000,
    0x20000,
    0x50000,
    0xa0000,
    0x140000,
    0x280000,
    0x500000,
    0xa00000,
    0x400000,
    0x2000000,
    0x5000000,
    0xa000000,
    0x14000000,
    0x28000000,
    0x50000000,
    0xa0000000,
    0x40000000,
    0x200000000,
    0x500000000,
    0xa00000000,
    0x1400000000,
    0x2800000000,
    0x5000000000,
    0xa000000000,
    0x4000000000,
    0x20000000000,
    0x50000000000,
    0xa0000000000,
    0x140000000000,
    0x280000000000,
    0x500000000000,
    0xa00000000000,
    0x400000000000,
    0x2000000000000,
    0x5000000000000,
    0xa000000000000,
    0x14000000000000,
    0x28000000000000,
    0x50000000000000,
    0xa0000000000000,
    0x40000000000000,
    0x200000000000000,
    0x500000000000000,
    0xa00000000000000,
    0x1400000000000000,
    0x2800000000000000,
    0x5000000000000000,
    0xa000000000000000,
    0x4000000000000000,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
    0x0,
];

const KNIGHT_ATTACKS: [u64; 64] = [
    0x20400,
    0x50800,
    0xa1100,
    0x142200,
    0x284400,
    0x508800,
    0xa01000,
    0x402000,
    0x2040004,
    0x5080008,
    0xa110011,
    0x14220022,
    0x28440044,
    0x50880088,
    0xa0100010,
    0x40200020,
    0x204000402,
    0x508000805,
    0xa1100110a,
    0x1422002214,
    0x2844004428,
    0x5088008850,
    0xa0100010a0,
    0x4020002040,
    0x20400040200,
    0x50800080500,
    0xa1100110a00,
    0x142200221400,
    0x284400442800,
    0x508800885000,
    0xa0100010a000,
    0x402000204000,
    0x2040004020000,
    0x5080008050000,
    0xa1100110a0000,
    0x14220022140000,
    0x28440044280000,
    0x50880088500000,
    0xa0100010a00000,
    0x40200020400000,
    0x204000402000000,
    0x508000805000000,
    0xa1100110a000000,
    0x1422002214000000,
    0x2844004428000000,
    0x5088008850000000,
    0xa0100010a0000000,
    0x4020002040000000,
    0x400040200000000,
    0x800080500000000,
    0x1100110a00000000,
    0x2200221400000000,
    0x4400442800000000,
    0x8800885000000000,
    0x100010a000000000,
    0x2000204000000000,
    0x4020000000000,
    0x8050000000000,
    0x110a0000000000,
    0x22140000000000,
    0x44280000000000,
    0x88500000000000,
    0x10a00000000000,
    0x20400000000000,
];

const KING_ATTACKS: [u64; 64] = [
    0x302,
    0x705,
    0xe0a,
    0x1c14,
    0x3828,
    0x7050,
    0xe0a0,
    0xc040,
    0x30203,
    0x70507,
    0xe0a0e,
    0x1c141c,
    0x382838,
    0x705070,
    0xe0a0e0,
    0xc040c0,
    0x3020300,
    0x7050700,
    0xe0a0e00,
    0x1c141c00,
    0x38283800,
    0x70507000,
    0xe0a0e000,
    0xc040c000,
    0x302030000,
    0x705070000,
    0xe0a0e0000,
    0x1c141c0000,
    0x3828380000,
    0x7050700000,
    0xe0a0e00000,
    0xc040c00000,
    0x30203000000,
    0x70507000000,
    0xe0a0e000000,
    0x1c141c000000,
    0x382838000000,
    0x705070000000,
    0xe0a0e0000000,
    0xc040c0000000,
    0x3020300000000,
    0x7050700000000,
    0xe0a0e00000000,
    0x1c141c00000000,
    0x38283800000000,
    0x70507000000000,
    0xe0a0e000000000,
    0xc040c000000000,
    0x302030000000000,
    0x705070000000000,
    0xe0a0e0000000000,
    0x1c141c0000000000,
    0x3828380000000000,
    0x7050700000000000,
    0xe0a0e00000000000,
    0xc040c00000000000,
    0x203000000000000,
    0x507000000000000,
    0xa0e000000000000,
    0x141c000000000000,
    0x2838000000000000,
    0x5070000000000000,
    0xa0e0000000000000,
    0x40c0000000000000,
];

const BISHOP_MAGICS: [u64; 64] = [
    0xa4406882040823a0,
    0x5010011501020810,
    0x90010041100000,
    0x8060890104020102,
    0x26021000082100,
    0x901008410404,
    0x11090110300421e4,
    0x10820082202618,
    0x820106058212140,
    0x40600400808100,
    0x9410101108410100,
    0x802b844400825000,
    0x2082111040002140,
    0x60e882080c041000,
    0x8624044402601088,
    0x21820088880830,
    0x805005c054050420,
    0x8128202008010243,
    0x5010011501020810,
    0x401201c04028000,
    0x1000820281700,
    0x40120010088400a,
    0x10041402008e0900,
    0x2a00109011000,
    0x250204010e210,
    0x9410101108410100,
    0x810820010240010,
    0x108000502c100,
    0x180840110802000,
    0x10004008a41000,
    0x800850046211000,
    0x22902444d006808,
    0x8888824054100400,
    0x4048041100840101,
    0x44020100020410,
    0x10404880080201,
    0x40028022420020,
    0xa084821880041000,
    0x88a0402084101,
    0x1004204849020100,
    0x804211200880a000,
    0x80108420a005020,
    0x1060820802000500,
    0x1001044010480202,
    0x14104202000190,
    0x6004008083000200,
    0x9410101108410100,
    0x61840100480200,
    0x11090110300421e4,
    0x9044202200000,
    0x90010041100000,
    0xc011850a42020400,
    0x148b15002020000,
    0x80902001444134,
    0x1191100228014202,
    0x5010011501020810,
    0x10820082202618,
    0x21820088880830,
    0x1000004404c5000,
    0x80800000020a0a01,
    0x10d0020200,
    0x21820180220,
    0x820106058212140,
    0xa4406882040823a0,
];

const ROOK_MAGICS: [u64; 64] = [
    0x880088040011021,
    0x6140081000402000,
    0x2300200240281100,
    0x4100280410006100,
    0x80022c00800800,
    0x480120003800400,
    0x3880210002004180,
    0x1200088540220104,
    0x93800084204009,
    0x1400020100040,
    0x1001100c02009,
    0x42800804100080,
    0x20e1002800910015,
    0x8a0808006000c00,
    0x1004002108020410,
    0x10c0800a41000880,
    0x8080014000a0,
    0x8a8040022000,
    0x90008012200180,
    0x1050020100028,
    0x48008004000880,
    0x8a0808006000c00,
    0x40008102126,
    0x100460004440481,
    0x8480400080008020,
    0x1c00200080804000,
    0x200280500080,
    0x2000100080800800,
    0x1000500080010,
    0x6008200040811,
    0x8022013400081022,
    0x10808a00104104,
    0x420400022800380,
    0x1c00200080804000,
    0x1000302001004101,
    0x1010300083800801,
    0x10a3802402800800,
    0x440800600800c00,
    0x14010644001008,
    0x4a822843020005a4,
    0x20a24002848000,
    0x408482010004000,
    0x4328100020008080,
    0x419081001010020,
    0x20e1002800910015,
    0x8a28020004008080,
    0x812100c0001,
    0x84008104420034,
    0x420400022800380,
    0x2400e24001029100,
    0x200280500080,
    0x100108058080,
    0x48008004000880,
    0x4800200040080,
    0x8041085003020c00,
    0x2004281040600,
    0x321044480013023,
    0x1000d04001210481,
    0x200401100082001,
    0x804050020081001,
    0x840100121008008d,
    0x405005208040021,
    0x1025100800820c,
    0x440020844902,
];

#[inline]
const fn make_move(origin: u64, dest: u64, flag: u64, promote: u64) -> u64 {
    (promote << 14) | (flag << 12) | (dest << 6) | origin
}

#[inline]
const fn flip_bb(mut bb: u64) -> u64 {
    bb = (bb & 0x00000000ffffffff) << 32 | (bb >> 32) & 0x00000000ffffffff;
    bb = (bb & 0x0000ffff0000ffff) << 16 | (bb >> 16) & 0x0000ffff0000ffff;
    bb = (bb & 0x00ff00ff00ff00ff) << 8 | (bb >> 8) & 0x00ff00ff00ff00ff;
    bb
}

#[inline]
pub const fn flip_square(sq: u64) -> u64 {
    (!sq & 0x38) | (sq & 0x07)
}

#[inline]
const fn log2(x: u64) -> u64 {
    LOG_2_TABLE[((x * LOG_2_DE_BRUIJN) >> 58) as usize]
}

impl ChessPos {
    #[inline]
    const fn flip_position(&self) -> Self {
        Self {
            pawn: flip_bb(self.pawn),
            ortho: flip_bb(self.ortho),
            diag: flip_bb(self.diag),
            own: flip_bb(self.other),
            other: flip_bb(self.own),
            squares: (self.squares & (1 << 19))
                | ((self.squares & (0x5 << 21)) >> 1)
                | ((self.squares & (0x5 << 20)) << 1)
                | (if ((self.squares >> 12) & 0x7f) == NO_EN_PASSANT {
                    NO_EN_PASSANT
                } else {
                    flip_square((self.squares >> 12) & 0x7f)
                } << 12)
                | (flip_square(self.squares & 0x3f) << 6)
                | flip_square((self.squares >> 6) & 0x3f),
            half_move: self.half_move,
            full_move: self.full_move,
        }
    }

    pub fn square_to_string(sq: u64) -> String {
        let f = ["a", "b", "c", "d", "e", "f", "g", "h"];
        let r = ["1", "2", "3", "4", "5", "6", "7", "8"];
        format!("{}{}", f[(sq & 7) as usize], r[((sq >> 3) & 7) as usize])
    }

    pub fn string_to_square(s: &str) -> Option<u64> {
        let mut chars = s.chars();
        let f = chars.next().unwrap();
        let r = chars.next().unwrap();
        let file = match f {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => return None,
        };
        let rank = match r {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
            _ => return None,
        } << 3;
        Some(file + rank)
    }

    pub fn to_fen(&self) -> String {
        let pos = if ((self.squares >> 19) & 1) == 1 {
            self.flip_position()
        } else {
            *self
        };
        let mut board: [char; 64] = ['.'; 64];
        for sq in 0..64 {
            if ((pos.own >> sq) & 1) == 1 {
                if (pos.squares & 0x3f) == sq {
                    board[sq as usize] = 'K';
                } else if ((pos.pawn >> sq) & 1) == 1 {
                    board[sq as usize] = 'P';
                } else if (((pos.ortho & pos.diag) >> sq) & 1) == 1 {
                    board[sq as usize] = 'Q';
                } else if ((pos.ortho >> sq) & 1) == 1 {
                    board[sq as usize] = 'R';
                } else if ((pos.diag >> sq) & 1) == 1 {
                    board[sq as usize] = 'B';
                } else {
                    board[sq as usize] = 'N';
                }
            } else if ((pos.other >> sq) & 1) == 1 {
                if ((pos.squares >> 6) & 0x3f) == sq {
                    board[sq as usize] = 'k';
                } else if ((pos.pawn >> sq) & 1) == 1 {
                    board[sq as usize] = 'p';
                } else if (((pos.ortho & pos.diag) >> sq) & 1) == 1 {
                    board[sq as usize] = 'q';
                } else if ((pos.ortho >> sq) & 1) == 1 {
                    board[sq as usize] = 'r';
                } else if ((pos.diag >> sq) & 1) == 1 {
                    board[sq as usize] = 'b';
                } else {
                    board[sq as usize] = 'n';
                }
            }
        }
        let flags = pos.squares;
        let en_passant = (flags >> 12) & 0x7f;
        let side = (flags >> 19) & 1;
        let w_oo = (flags >> 20) & 1;
        let b_oo = (flags >> 21) & 1;
        let w_ooo = (flags >> 22) & 1;
        let b_ooo = (flags >> 23) & 1;

        let castling_string = if (w_oo | b_oo | w_ooo | b_ooo) == 0 {
            "-".to_string()
        } else {
            format!(
                "{}{}{}{}",
                if w_oo == 1 { "K" } else { "" },
                if w_ooo == 1 { "Q" } else { "" },
                if b_oo == 1 { "k" } else { "" },
                if b_ooo == 1 { "q" } else { "" },
            )
        };

        format!(
            "{} {} {} {} {} {}",
            (0..64)
                .step_by(8)
                .rev()
                .map(|rank| {
                    (rank..rank + 8)
                        .map(|file| board[file as usize].to_string())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("/"),
            if side == 1 { "b" } else { "w" },
            castling_string,
            if en_passant == NO_EN_PASSANT {
                "-".to_string()
            } else {
                Self::square_to_string(en_passant)
            },
            pos.half_move,
            pos.full_move
        )
        .replace("........", "8")
        .replace(".......", "7")
        .replace("......", "6")
        .replace(".....", "5")
        .replace("....", "4")
        .replace("...", "3")
        .replace("..", "2")
        .replace('.', "1")
    }

    pub fn from_fen(pos_string: &str) -> Option<Self> {
        let items = pos_string.split_whitespace().collect::<Vec<_>>();

        if items.len() < 4 || items.len() > 6 {
            return None;
        }

        let mut pos = Self {
            pawn: 0,
            ortho: 0,
            diag: 0,
            own: 0,
            other: 0,
            squares: 0,
            half_move: 0,
            full_move: 0,
        };

        let (rows, side, castle, ep) = (items[0], items[1], items[2], items[3]);
        let half = if items.len() > 4 { items[4] } else { "0" };
        let full = if items.len() > 5 { items[5] } else { "1" };

        if side != "w" && side != "b" {
            return None;
        }

        for c in castle.chars() {
            match c {
                'K' => pos.squares |= 1 << 20,
                'Q' => pos.squares |= 1 << 22,
                'k' => pos.squares |= 1 << 21,
                'q' => pos.squares |= 1 << 23,
                '-' => {}
                _ => return None,
            }
        }

        if ep == "-" {
            pos.squares |= NO_EN_PASSANT << 12;
        } else if let Some(s) = Self::string_to_square(ep) {
            pos.squares |= s << 12;
        } else {
            return None;
        }

        if let (Ok(h), Ok(f)) = (half.parse::<u64>(), full.parse::<u64>()) {
            pos.half_move = h;
            pos.full_move = f;
        } else {
            return None;
        }

        let cells = rows
            .replace('1', ".")
            .replace('2', "..")
            .replace('3', "...")
            .replace('4', "....")
            .replace('5', ".....")
            .replace('6', "......")
            .replace('7', ".......")
            .replace('8', "........")
            .replace('/', "");

        for (sq, ch) in (0..64)
            .step_by(8)
            .rev()
            .flat_map(|i| i..i + 8)
            .zip(cells.chars())
        {
            let bb = 1 << (sq as u64);
            match ch {
                'P' => {
                    pos.own |= bb;
                    pos.pawn |= bb;
                }
                'N' => {
                    pos.own |= bb;
                }
                'B' => {
                    pos.own |= bb;
                    pos.diag |= bb;
                }
                'R' => {
                    pos.own |= bb;
                    pos.ortho |= bb;
                }
                'Q' => {
                    pos.own |= bb;
                    pos.diag |= bb;
                    pos.ortho |= bb;
                }
                'K' => {
                    pos.own |= bb;
                    pos.squares |= sq as u64;
                }
                'p' => {
                    pos.other |= bb;
                    pos.pawn |= bb;
                }
                'n' => {
                    pos.other |= bb;
                }
                'b' => {
                    pos.other |= bb;
                    pos.diag |= bb;
                }
                'r' => {
                    pos.other |= bb;
                    pos.ortho |= bb;
                }
                'q' => {
                    pos.other |= bb;
                    pos.diag |= bb;
                    pos.ortho |= bb;
                }
                'k' => {
                    pos.other |= bb;
                    pos.squares |= (sq as u64) << 6;
                }
                _ => {}
            }
        }

        if side == "b" {
            pos.squares |= 1 << 19;
            Some(pos.flip_position())
        } else {
            Some(pos)
        }
    }
}

impl GamePosition for ChessPos {
    type Move = u64;
    type Params = ();

    fn startpos(_: ()) -> Self {
        Self {
            pawn: 0x00ff00000000ff00,
            ortho: 0x8900000000000089,
            diag: 0x2c0000000000002c,
            own: 0x000000000000ffff,
            other: 0xffff000000000000,
            squares: (0xf << 20) | (NO_EN_PASSANT << 12) | (60 << 6) | 4,
            half_move: 0,
            full_move: 1,
        }
    }

    fn play_move(&self, mv: Self::Move) -> Self {
        let mut pos = *self;

        let (origin, destination) = (mv & 0x3f, (mv >> 6) & 0x3f);

        pos.half_move += 1;
        pos.full_move += (pos.squares >> 19) & 1;
        pos.squares ^= 1 << 19;

        let mut ep_sq = NO_EN_PASSANT;

        if (((pos.own | pos.other) >> destination) & 1) == 1 {
            pos.half_move = 0;
        }

        match origin {
            0 => pos.squares &= !(1 << 22),
            4 => pos.squares &= !((1 << 22) | (1 << 20)),
            7 => pos.squares &= !(1 << 20),
            _ => {}
        }
        match destination {
            0 => pos.squares &= !(1 << 22),
            4 => pos.squares &= !((1 << 22) | (1 << 20)),
            7 => pos.squares &= !(1 << 20),
            56 => pos.squares &= !(1 << 23),
            60 => pos.squares &= !((1 << 23) | (1 << 21)),
            63 => pos.squares &= !(1 << 21),
            _ => {}
        }

        let origin_bb = 1 << origin;
        let destination_bb = 1 << destination;

        match (mv >> 12) & 3 {
            FLAG_NONE => {
                if origin == (pos.squares & 0x3f) {
                    pos.squares &= !0x3f;
                    pos.squares |= destination;
                    pos.own &= !origin_bb;
                    pos.own |= destination_bb;
                    pos.other &= !destination_bb;
                    pos.pawn &= !destination_bb;
                    pos.ortho &= !destination_bb;
                    pos.diag &= !destination_bb;
                } else if (pos.pawn & origin_bb) != 0 {
                    pos.half_move = 0;
                    pos.own &= !origin_bb;
                    pos.own |= destination_bb;
                    pos.other &= !destination_bb;
                    pos.pawn = (pos.pawn & !origin_bb) | destination_bb;
                    pos.ortho &= !destination_bb;
                    pos.diag &= !destination_bb;
                    if destination - origin == 16 {
                        ep_sq = origin + 8;
                    }
                } else if (pos.ortho & pos.diag & origin_bb) != 0 {
                    pos.own &= !origin_bb;
                    pos.own |= destination_bb;
                    pos.other &= !destination_bb;
                    pos.pawn &= !destination_bb;
                    pos.ortho &= !origin_bb;
                    pos.ortho |= destination_bb;
                    pos.diag &= !origin_bb;
                    pos.diag |= destination_bb;
                } else if (pos.ortho & origin_bb) != 0 {
                    pos.own &= !origin_bb;
                    pos.own |= destination_bb;
                    pos.other &= !destination_bb;
                    pos.pawn &= !destination_bb;
                    pos.ortho &= !origin_bb;
                    pos.ortho |= destination_bb;
                    pos.diag &= !destination_bb;
                } else if (pos.diag & origin_bb) != 0 {
                    pos.own &= !origin_bb;
                    pos.own |= destination_bb;
                    pos.other &= !destination_bb;
                    pos.pawn &= !destination_bb;
                    pos.ortho &= !destination_bb;
                    pos.diag &= !origin_bb;
                    pos.diag |= destination_bb;
                } else if (pos.own & origin_bb) != 0 {
                    pos.own &= !origin_bb;
                    pos.own |= destination_bb;
                    pos.other &= !destination_bb;
                    pos.pawn &= !destination_bb;
                    pos.ortho &= !destination_bb;
                    pos.diag &= !destination_bb;
                }
            }
            FLAG_PROMOTE => {
                pos.own &= !origin_bb;
                pos.own |= destination_bb;
                pos.other &= !destination_bb;
                pos.pawn &= !(destination_bb | origin_bb);
                pos.ortho &= !destination_bb;
                pos.diag &= !destination_bb;
                match (mv >> 14) & 3 {
                    PRMT_QUEEN => {
                        pos.ortho |= destination_bb;
                        pos.diag |= destination_bb;
                    }
                    PRMT_ROOK => pos.ortho |= destination_bb,
                    PRMT_BISHOP => pos.diag |= destination_bb,
                    _ => {}
                }
            }
            FLAG_CASTLE => match destination {
                2 => {
                    pos.squares &= !0x3f;
                    pos.squares |= 2;
                    pos.squares &= !((1 << 22) | (1 << 20));
                    pos.ortho ^= 0x09;
                    pos.own ^= 0x1d;
                }
                6 => {
                    pos.squares &= !0x3f;
                    pos.squares |= 6;
                    pos.squares &= !((1 << 22) | (1 << 20));
                    pos.ortho ^= 0xa0;
                    pos.own ^= 0xf0;
                }
                _ => {}
            },
            FLAG_ENPASSANT => {
                pos.half_move = 0;
                pos.pawn ^= destination_bb | origin_bb | (destination_bb >> 8);
                pos.own &= !origin_bb;
                pos.own |= destination_bb;
                pos.other &= !(destination_bb >> 8);
            }
            _ => {}
        }
        pos.squares &= !0x7f000;
        pos.squares |= ep_sq << 12;

        pos.flip_position()
    }
}

impl SMagic {
    fn empty() -> Self {
        Self {
            attack_table: Vec::new(),
            mask: 0,
            magic: 0,
            shift: 0,
        }
    }
}

impl ChessHandler {
    pub fn square_to_string(&self, sq: u64) -> String {
        let f = ["a", "b", "c", "d", "e", "f", "g", "h"];
        let r = ["1", "2", "3", "4", "5", "6", "7", "8"];
        format!("{}{}", f[(sq & 7) as usize], r[((sq >> 3) & 7) as usize])
    }

    pub fn move_string(&self, mv: u64, side: u64) -> String {
        let o = if side == 1 {
            flip_square(mv & 0x3f)
        } else {
            mv & 0x3f
        };
        let d = if side == 1 {
            flip_square((mv >> 6) & 0x3f)
        } else {
            (mv >> 6) & 0x3f
        };
        let f = (mv >> 12) & 0x3;
        let p = (mv >> 14) & 0x3;

        match f {
            0 => format!("{}{}", self.square_to_string(o), self.square_to_string(d)),
            1 => {
                format!(
                    "{}{}{}",
                    self.square_to_string(o),
                    self.square_to_string(d),
                    if p == 0 {
                        "q"
                    } else if p == 1 {
                        "r"
                    } else if p == 2 {
                        "b"
                    } else if p == 3 {
                        "n"
                    } else {
                        panic!("promote flag invalid")
                    }
                )
            }
            2 => (if d == 2 { "e1c1" } else { "e1g1" }).to_string(),
            3 => format!("{}{}ep", self.square_to_string(o), self.square_to_string(d)),
            _ => panic!("move flag invalid: {}", f),
        }
    }

    fn bishop_unblocked_attack_rays(square: u64) -> u64 {
        let rank = square >> 3;
        let file = square & 7;
        let mut major: u64;
        let mut minor: u64;
        if file >= rank {
            let shift = file - rank;
            major = MAJOR_DIAG << shift;
            for excl_file in 0..shift {
                major &= !(FILE_A << excl_file);
            }
        } else {
            let shift = rank - file;
            major = MAJOR_DIAG >> shift;
            for excl_file in 0..shift {
                major &= !((FILE_A << 7) >> excl_file);
            }
        }
        if file + rank >= 7 {
            let shift = file + rank - 7;
            minor = MINOR_DIAG << shift;
            for excl_file in 0..shift {
                minor &= !(FILE_A << excl_file);
            }
        } else {
            let shift = 7 - file - rank;
            minor = MINOR_DIAG >> shift;
            for excl_file in 0..shift {
                minor &= !((FILE_A << 7) >> excl_file);
            }
        }
        (major | minor) & !(EDGES | (1 << square))
    }

    fn rook_unblocked_attack_rays(square: u64) -> u64 {
        let rank = square >> 3;
        let file = square & 7;
        let mut rays = (FILE_A << file) | (RANK_1 << (rank << 3));
        if file != 0 {
            rays &= !FILE_A;
        }
        if file != 7 {
            rays &= !(FILE_A << 7);
        }
        if rank != 0 {
            rays &= !RANK_1;
        }
        if rank != 7 {
            rays &= !(RANK_1 << 56);
        }
        rays & !(1 << square)
    }

    fn bishop_blocked_attack_rays(square: u64, blockers: u64) -> u64 {
        let rank = square >> 3;
        let file = square & 7;
        let mut attack_bb = 0u64;
        let mut res: u64 = 0;

        attack_bb |= (0..rank)
            .rev()
            .zip((0..file).rev())
            .take_while(|(r, f)| {
                res = 1 << ((r << 3) | f);
                (blockers & res) == 0
            })
            .map(|(r, f)| 1 << ((r << 3) | f))
            .fold(0u64, |acc, x| acc | x);
        attack_bb |= res;

        res = 0;
        attack_bb |= (0..rank)
            .rev()
            .zip(file + 1..=7)
            .take_while(|(r, f)| {
                res = 1 << ((r << 3) | f);
                (blockers & res) == 0
            })
            .map(|(r, f)| 1 << ((r << 3) | f))
            .fold(0u64, |acc, x| acc | x);
        attack_bb |= res;

        res = 0;
        attack_bb |= (rank + 1..=7)
            .zip((0..file).rev())
            .take_while(|(r, f)| {
                res = 1 << ((r << 3) | f);
                (blockers & res) == 0
            })
            .map(|(r, f)| 1 << ((r << 3) | f))
            .fold(0u64, |acc, x| acc | x);
        attack_bb |= res;

        res = 0;
        attack_bb |= (rank + 1..=7)
            .zip(file + 1..=7)
            .take_while(|(r, f)| {
                res = 1 << ((r << 3) | f);
                (blockers & res) == 0
            })
            .map(|(r, f)| 1 << ((r << 3) | f))
            .fold(0u64, |acc, x| acc | x);
        attack_bb |= res;

        attack_bb
    }

    fn rook_blocked_attack_rays(square: u64, blockers: u64) -> u64 {
        let rank = square >> 3;
        let file = square & 7;
        let mut attack_bb = 0u64;
        let mut res: u64 = 0;

        attack_bb |= (0..file)
            .rev()
            .take_while(|f| {
                res = 1 << ((rank << 3) | f);
                (blockers & res) == 0
            })
            .map(|f| 1 << ((rank << 3) | f))
            .fold(0u64, |acc, x| acc | x);
        attack_bb |= res;

        res = 0;
        attack_bb |= (file + 1..=7)
            .take_while(|f| {
                res = 1 << ((rank << 3) | f);
                (blockers & res) == 0
            })
            .map(|f| 1 << ((rank << 3) | f))
            .fold(0u64, |acc, x| acc | x);
        attack_bb |= res;

        res = 0;
        attack_bb |= (0..rank)
            .rev()
            .take_while(|r| {
                res = 1 << ((r << 3) | file);
                (blockers & res) == 0
            })
            .map(|r| 1 << ((r << 3) | file))
            .fold(0u64, |acc, x| acc | x);
        attack_bb |= res;

        res = 0;
        attack_bb |= (rank + 1..=7)
            .take_while(|r| {
                res = 1 << ((r << 3) | file);
                (blockers & res) == 0
            })
            .map(|r| 1 << ((r << 3) | file))
            .fold(0u64, |acc, x| acc | x);
        attack_bb |= res;

        attack_bb
    }

    const fn popcount(mut bb: u64) -> u64 {
        let mut count: u64 = 0;
        while bb != 0 {
            count += 1;
            bb &= bb - 1;
        }
        count
    }

    fn bit_permutations(mut bb: u64) -> Vec<u64> {
        let mut permutations: Vec<u64> = Vec::new();
        let mut digits: Vec<u64> = Vec::new();
        while bb != 0 {
            digits.push(LOG_2_TABLE[(((bb & (!bb + 1)) * LOG_2_DE_BRUIJN) >> 58) as usize]);
            bb &= bb - 1;
        }
        for perm_number in 0..1 << digits.len() {
            let mut value: u64 = 0;
            for (i, d) in digits.iter().enumerate() {
                if ((perm_number >> i) & 1) == 1 {
                    value |= 1 << d;
                }
            }
            permutations.push(value);
        }
        permutations
    }

    fn test_bishop_magic(square: u64, magic: u64) -> Option<([u64; 4096], u64)> {
        let mask = Self::bishop_unblocked_attack_rays(square);
        let shift = 64 - Self::popcount(mask);
        let mut vision_table = [0u64; 4096];
        for blocker_pattern in Self::bit_permutations(mask) {
            let index = (blocker_pattern * magic) >> shift;
            if vision_table[index as usize] == 0 {
                vision_table[index as usize] =
                    Self::bishop_blocked_attack_rays(square, blocker_pattern);
            } else {
                return None;
            }
        }
        Some((vision_table, shift))
    }

    fn test_rook_magic(square: u64, magic: u64) -> Option<([u64; 4096], u64)> {
        let mask = Self::rook_unblocked_attack_rays(square);
        let shift = 64 - Self::popcount(mask);
        let mut vision_table = [0u64; 4096];
        for blocker_pattern in Self::bit_permutations(mask) {
            let index = (blocker_pattern * magic) >> shift;
            if vision_table[index as usize] == 0 {
                vision_table[index as usize] =
                    Self::rook_blocked_attack_rays(square, blocker_pattern);
            } else {
                return None;
            }
        }
        Some((vision_table, shift))
    }

    fn square_is_attacked(&self, square: u64, pos: ChessPos) -> bool {
        let blockers = pos.own | pos.other;

        if (KING_ATTACKS[(pos.squares & 0x3f) as usize] & (1 << ((pos.squares >> 6) & 0x3f))) != 0 {
            return true;
        }

        let m_rook = &self.rook_magics[square as usize];
        if (pos.other
            & pos.ortho
            & (m_rook.attack_table
                [(((blockers & m_rook.mask) * m_rook.magic) >> m_rook.shift) as usize]))
            != 0
        {
            return true;
        }

        let m_bishop = &self.bishop_magics[square as usize];
        if (pos.other
            & pos.diag
            & (m_bishop.attack_table
                [(((blockers & m_bishop.mask) * m_bishop.magic) >> m_bishop.shift) as usize]))
            != 0
        {
            return true;
        }

        let opp_knights =
            pos.other & !(pos.ortho | pos.diag | pos.pawn | (1 << ((pos.squares >> 6) & 0x3f)));
        if (KNIGHT_ATTACKS[square as usize] & opp_knights) != 0 {
            return true;
        }

        if (PAWN_ATTACKS[square as usize] & pos.other & pos.pawn) != 0 {
            return true;
        }

        false
    }
}

impl GameHandler<ChessPos> for ChessHandler {
    type Eval = i32;
    type Params = ();

    const EVAL_MINIMUM: i32 = -100000000;
    const EVAL_MAXIMUM: i32 = 100000000;
    const EVAL_EPSILON: i32 = 1;

    fn new(_: ()) -> Self {
        let mut bishop_table: [SMagic; 64] = std::array::from_fn(|_| SMagic::empty());
        let mut rook_table: [SMagic; 64] = std::array::from_fn(|_| SMagic::empty());
        for square in 0u64..64u64 {
            if let Some((vision_table, shift)) =
                Self::test_bishop_magic(square, BISHOP_MAGICS[square as usize])
            {
                bishop_table[square as usize] = SMagic {
                    attack_table: vision_table
                        .into_iter()
                        .take((1 << (64 - shift)) as usize)
                        .collect(),
                    mask: Self::bishop_unblocked_attack_rays(square),
                    magic: BISHOP_MAGICS[square as usize],
                    shift,
                };
            } else {
                panic!(
                    "BISHOP PANIC: Square {}: Magic {} is invalid.",
                    square, BISHOP_MAGICS[square as usize]
                );
            }
            if let Some((vision_table, shift)) =
                Self::test_rook_magic(square, ROOK_MAGICS[square as usize])
            {
                rook_table[square as usize] = SMagic {
                    attack_table: vision_table
                        .into_iter()
                        .take((1 << (64 - shift)) as usize)
                        .collect(),
                    mask: Self::rook_unblocked_attack_rays(square),
                    magic: ROOK_MAGICS[square as usize],
                    shift,
                };
            } else {
                panic!(
                    "ROOK PANIC: Square {}: Magic {} is invalid.",
                    square, ROOK_MAGICS[square as usize]
                );
            }
        }
        Self {
            bishop_magics: bishop_table,
            rook_magics: rook_table,
        }
    }

    fn get_legal_moves(&self, pos: ChessPos) -> impl Iterator<Item = u64> {
        let pawn = pos.own & pos.pawn;
        let knight = pos.own & !(pos.ortho | pos.diag | pos.pawn | (1 << (pos.squares & 0x3f)));
        let bishop = pos.own & pos.diag & !pos.ortho;
        let rook = pos.own & pos.ortho & !pos.diag;
        let queen = pos.own & pos.diag & pos.ortho;

        let blockers = pos.own | pos.other;
        let mut moves: Vec<u64> = Vec::new();
        let mut bb: u64;

        if ((pos.squares >> 20) & 1) == 1 && (blockers & KINGSIDE_CASTLE_CLEARANCE_MASK) == 0 {
            bb = KINGSIDE_CASTLE_CHECK_MASK;
            let mut can_cross = true;
            while bb != 0 && can_cross {
                can_cross = !self.square_is_attacked(log2(bb & (!bb + 1)), pos);
                bb &= bb - 1;
            }
            if can_cross {
                moves.push(make_move(4, 6, FLAG_CASTLE, 0));
            }
        }
        if ((pos.squares >> 22) & 1) == 1 && (blockers & QUEENSIDE_CASTLE_CLEARANCE_MASK) == 0 {
            bb = QUEENSIDE_CASTLE_CHECK_MASK;
            let mut can_cross = true;
            while bb != 0 && can_cross {
                can_cross = !self.square_is_attacked(log2(bb & (!bb + 1)), pos);
                bb &= bb - 1;
            }
            if can_cross {
                moves.push(make_move(4, 2, FLAG_CASTLE, 0));
            }
        }

        bb = KING_ATTACKS[(pos.squares & 0x3f) as usize]
            & !KING_ATTACKS[((pos.squares >> 6) & 0x3f) as usize]
            & !pos.own;

        while bb != 0 {
            moves.push(make_move(
                pos.squares & 0x3f,
                log2(bb & (!bb + 1)),
                FLAG_NONE,
                0,
            ));
            bb &= bb - 1;
        }

        for square in 0..64 {
            if ((pawn >> square) & 1) == 1 {
                match square >> 3 {
                    6 => {
                        bb = PAWN_ATTACKS[square as usize] & pos.other;
                        let mut dest: u64;
                        while bb != 0 {
                            dest = log2(bb & (!bb + 1));
                            moves.extend(
                                (0..4).map(|prmt| make_move(square, dest, FLAG_PROMOTE, prmt)),
                            );
                            bb &= bb - 1;
                        }

                        if ((blockers >> (square + 8)) & 1) == 0 {
                            moves.extend(
                                (0..4)
                                    .map(|prmt| make_move(square, square + 8, FLAG_PROMOTE, prmt)),
                            );
                        }
                    }

                    s => {
                        if ((blockers >> (square + 8)) & 1) == 0 {
                            moves.push(make_move(square, square + 8, FLAG_NONE, 0));
                            if s == 1 && ((blockers >> (square + 16)) & 1) == 0 {
                                moves.push(make_move(square, square + 16, FLAG_NONE, 0));
                            }
                        }

                        bb = PAWN_ATTACKS[square as usize] & pos.other;
                        let mut dest: u64;
                        while bb != 0 {
                            dest = log2(bb & (!bb + 1));
                            moves.push(make_move(square, dest, FLAG_NONE, 0));
                            bb &= bb - 1;
                        }
                        let ep_square = (pos.squares >> 12) & 0x7f;

                        if (PAWN_ATTACKS[square as usize] & (1 << ep_square)) != 0 {
                            moves.push(make_move(square, ep_square, FLAG_ENPASSANT, 0));
                        }
                    }
                }
            } else if ((knight >> square) & 1) == 1 {
                bb = KNIGHT_ATTACKS[square as usize] & !pos.own;
                while bb != 0 {
                    moves.push(make_move(square, log2(bb & (!bb + 1)), FLAG_NONE, 0));
                    bb &= bb - 1;
                }
            } else if ((bishop >> square) & 1) == 1 {
                let m_bishop = &self.bishop_magics[square as usize];
                bb = !pos.own
                    & m_bishop.attack_table[(((blockers & m_bishop.mask) * m_bishop.magic)
                        >> m_bishop.shift) as usize];

                while bb != 0 {
                    moves.push(make_move(square, log2(bb & (!bb + 1)), FLAG_NONE, 0));
                    bb &= bb - 1;
                }
            } else if ((rook >> square) & 1) == 1 {
                let m_rook = &self.rook_magics[square as usize];
                bb = !pos.own
                    & m_rook.attack_table
                        [(((blockers & m_rook.mask) * m_rook.magic) >> m_rook.shift) as usize];

                while bb != 0 {
                    moves.push(make_move(square, log2(bb & (!bb + 1)), FLAG_NONE, 0));
                    bb &= bb - 1;
                }
            } else if ((queen >> square) & 1) == 1 {
                let m_bishop = &self.bishop_magics[square as usize];
                let m_rook = &self.rook_magics[square as usize];
                bb = !pos.own
                    & (m_bishop.attack_table[(((blockers & m_bishop.mask) * m_bishop.magic)
                        >> m_bishop.shift) as usize]
                        | m_rook.attack_table
                            [(((blockers & m_rook.mask) * m_rook.magic) >> m_rook.shift) as usize]);

                while bb != 0 {
                    moves.push(make_move(square, log2(bb & (!bb + 1)), FLAG_NONE, 0));
                    bb &= bb - 1;
                }
            }
        }
        moves.into_iter().filter(move |&mv| {
            let new_pos = pos.play_move(mv).flip_position();
            let king_sq = new_pos.squares & 0x3f;
            !self.square_is_attacked(king_sq, new_pos)
        })
    }

    fn evaluate(&self, _pos: ChessPos, _depth: usize, _max_depth: usize) -> Self::Eval {
        todo!();
    }
}
