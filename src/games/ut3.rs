use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ut3Board {
    pub us: u64,
    pub them: u64,
    pub share: u64,
}

impl Ut3Board {
    const CHUNK: u64 = 0b111111111;
    const DBLCHUNK: u64 = (Self::CHUNK << 9) | Self::CHUNK;
    const ZONE_ANY: u64 = 9;
    #[inline]
    const fn lines(grid: u64) -> u64 {
        0b_000_100_000_000_100_000_000_100 * (grid & 1)
            + 0b_000_000_000_000_010_000_100_000 * ((grid >> 1) & 1)
            + 0b_100_000_000_000_001_100_000_000 * ((grid >> 2) & 1)
            + 0b_000_000_000_100_000_000_000_010 * ((grid >> 3) & 1)
            + 0b_010_010_000_010_000_000_010_000 * ((grid >> 4) & 1)
            + 0b_000_000_000_001_000_010_000_000 * ((grid >> 5) & 1)
            + 0b_001_000_100_000_000_000_000_001 * ((grid >> 6) & 1)
            + 0b_000_000_010_000_000_000_001_000 * ((grid >> 7) & 1)
            + 0b_000_001_001_000_000_001_000_000 * ((grid >> 8) & 1)
    }
    #[inline]
    const fn line_presence(grid: u64) -> bool {
        0 != ((0b10110110 | ((grid & 1) * 0xff))
            & (0b11101110 | (((grid >> 1) & 1) * 0xff))
            & (0b01011110 | (((grid >> 2) & 1) * 0xff))
            & (0b11110101 | (((grid >> 3) & 1) * 0xff))
            & (0b00101101 | (((grid >> 4) & 1) * 0xff))
            & (0b11011101 | (((grid >> 5) & 1) * 0xff))
            & (0b01110011 | (((grid >> 6) & 1) * 0xff))
            & (0b11101011 | (((grid >> 7) & 1) * 0xff))
            & (0b10011011 | (((grid >> 8) & 1) * 0xff)))
    }
}

impl GamePosition for Ut3Board {
    type Move = u64;

    fn startpos() -> Self {
        Self { us: 0u64, them: 0u64, share: Self::ZONE_ANY << 54 }
    }

    fn play_move(&self, mv: Self::Move) -> Self {
        let Self { mut us, them, mut share } = *self;
        let line_occupancy = if mv > 62 {
            share |= 1 << (mv - 63);
            Self::line_presence(share >> (9 * (mv / 9) - 63))
        } else {
            us |= 1 << mv;
            Self::line_presence(us >> (9 * (mv / 9)))
        };
        if line_occupancy {
            share |= 1 << (36 + mv / 9);
        }
        let next_chunk = if mv % 9 > 6 {
            ((share | (share >> 18)) >> (9 * ((mv % 9) - 7))) & Self::CHUNK
        } else {
            ((us | them) >> (9 * (mv % 9))) & Self::CHUNK
        };
        let zone = if next_chunk == Self::CHUNK ||(((share | (share >> 9)) >> (36 + mv % 9)) & 1) == 1 {
            Self::ZONE_ANY
        } else {
            mv % 9
        };
        // Now we flip the board.
        share = ((share & Self::DBLCHUNK) << 18)
            | ((share >> 18) & Self::DBLCHUNK)
            | ((share & (Self::CHUNK << 45)) >> 9)
            | ((share & (Self::CHUNK << 36)) << 9)
            | (zone << 54);
        Self { us: them, them: us, share }
    }
}

pub struct Ut3Handler {
    large_table: Vec<i32>,
    small_table: Vec<i32>,
}

impl Ut3Handler {
    pub const OUTCOME_WIN: i32 = 1000000;
    pub const OUTCOME_DRAW: i32 = 0;
    pub const OUTCOME_LOSS: i32 = -1000000;

    const BIG_TWO_COUNT: i32 = 90;
    const BIG_ONE_COUNT: i32 = 20;
    const SMALL_TWO_COUNT: i32 = 8;
    const SMALL_ONE_COUNT: i32 = 1;

    const CENTRE: i32 = 9;
    const CORNER: i32 = 7;
    const EDGE: i32 = 5;
    const SQ_BIG: i32 = 25;

    const CORNER_MASK: u64 = 0b_101_000_101;
    const EDGE_MASK: u64 = 0b_010_101_010;
    const CENTRE_MASK: u64 = 0b_000_010_000;

    const LINE: u64 = 0b111;
}

impl GameHandler<Ut3Board> for Ut3Handler {

    type Eval = i32;

    const EVAL_MINIMUM: i32 = Self::OUTCOME_LOSS;
    const EVAL_MAXIMUM: i32 = Self::OUTCOME_WIN;
    const EVAL_EPSILON: i32 = 1;

    fn new() -> Self {
        let mut large_table: Vec<i32> = vec![0; 262144];
        let mut small_table: Vec<i32> = vec![0; 262144];

        let pop_count: Vec<i32> = (0..512)
            .map(|i| (0..9).fold(0, |acc, j| acc + ((i >> j) & 1)))
            .collect();

        for us in (0..512).map(|us| us as u64) {
            for them in (0..512).map(|them| them as u64) {
                let mut eval_large: i32 = 0;
                let mut eval_small: i32 = 0;

                let us_lines = Ut3Board::lines(us);
                let them_lines = Ut3Board::lines(them);

                let mut us_won: bool = false;
                let mut them_won: bool = false;

                for i in (0..24).step_by(3) {
                    let us_count = pop_count[((us_lines >> i) & Self::LINE) as usize];
                    let them_count = pop_count[((them_lines >> i) & Self::LINE) as usize];

                    if us_count != 0 && them_count != 0 {
                        continue;
                    }
                    if us_count == 3 {
                        us_won = true;
                        break;
                    }
                    if them_count == 3 {
                        them_won = true;
                        break;
                    }

                    eval_large += match us_count {
                        2 => Self::BIG_TWO_COUNT,
                        1 => Self::BIG_ONE_COUNT,
                        _ => 0,
                    } - match them_count {
                        2 => Self::BIG_TWO_COUNT,
                        1 => Self::BIG_ONE_COUNT,
                        _ => 0,
                    };
                    eval_small += match us_count {
                        2 => Self::SMALL_TWO_COUNT,
                        1 => Self::SMALL_ONE_COUNT,
                        _ => 0,
                    } - match them_count {
                        2 => Self::SMALL_TWO_COUNT,
                        1 => Self::SMALL_ONE_COUNT,
                        _ => 0,
                    };
                }
                let eval_pos = Self::CORNER
                    * (pop_count[(us & Self::CORNER_MASK) as usize]
                        - pop_count[(them & Self::CORNER_MASK) as usize])
                    + Self::EDGE
                        * (pop_count[(us & Self::EDGE_MASK) as usize]
                            - pop_count[(them & Self::EDGE_MASK) as usize])
                    + Self::CENTRE
                        * (pop_count[(us & Self::CENTRE_MASK) as usize]
                            - pop_count[(them & Self::CENTRE_MASK) as usize]);
                if us_won {
                    large_table[((them << 9) | us) as usize] = Self::OUTCOME_WIN;
                } else if them_won {
                    large_table[((them << 9) | us) as usize] = Self::OUTCOME_LOSS;
                } else if pop_count[(us | them) as usize] == 9 {
                    large_table[((them << 9) | us) as usize] = Self::OUTCOME_DRAW;
                } else {
                    large_table[((them << 9) | us) as usize] = eval_large + eval_pos * Self::SQ_BIG;
                    small_table[((them << 9) | us) as usize] = eval_small + eval_pos;
                }
            }
        }
        Self { large_table, small_table }
    }

    fn get_legal_moves(&self, board: Ut3Board) -> impl Iterator<Item = u64> {
        enum LegalMoves<T1, T2, T3> {
            NoMoves,
            AllZones(T1),
            FirstSeven(T2),
            LastTwo(T3),
        }

        impl<T1, T2, T3> Iterator for LegalMoves<T1, T2, T3>
        where
            T1: Iterator,
            T2: Iterator<Item = <T1 as Iterator>::Item>,
            T3: Iterator<Item = <T1 as Iterator>::Item>,
        {
            type Item = <T1 as Iterator>::Item;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Self::NoMoves => None,
                    Self::AllZones(x) => x.next(),
                    Self::FirstSeven(x) => x.next(),
                    Self::LastTwo(x) => x.next(),
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                match self {
                    Self::NoMoves => (0, Some(0)),
                    Self::AllZones(x) => x.size_hint(),
                    Self::FirstSeven(x) => x.size_hint(),
                    Self::LastTwo(x) => x.size_hint(),
                }
            }
        }

        let Ut3Board { us, them, share } = board;

        if Ut3Board::line_presence(share >> 36) || Ut3Board::line_presence(share >> 45) {
            return LegalMoves::NoMoves;
        }

        let zone = (share >> 54) & 0b1111;

        match zone {
            Ut3Board::ZONE_ANY => {
                let nw_to_sw = us | them;
                let s_to_se = (share >> 18) | share;
                let large = (share >> 36) | (share >> 45);

                LegalMoves::AllZones(
                    (0..63)
                        .filter(move |i| ((nw_to_sw >> i) & 1) == 0 && ((large >> (i / 9)) & 1) == 0)
                        .chain((63..81).filter(move |i| {
                            ((s_to_se >> (i - 63)) & 1) == 0 && ((large >> (i / 9)) & 1) == 0
                        })),
                )
            }
            7 | 8 => {
                let s_to_se = (share >> 18) | share;
                LegalMoves::LastTwo(
                    (9 * zone..9 * zone + 9).filter(move |i| ((s_to_se >> (i - 63)) & 1) == 0),
                )
            }
            _ => {
                let nw_to_sw = us | them;
                LegalMoves::FirstSeven(
                    (9 * zone..9 * zone + 9).filter(move |i| ((nw_to_sw >> i) & 1) == 0),
                )
            }
        }
    }

    fn evaluate(&self, board: Ut3Board, depth: usize, max_depth: usize) -> Self::Eval {
        let Ut3Board { us, them, share } = board;
        let eval = self.large_table[((share >> 36) & Ut3Board::DBLCHUNK) as usize];
        if eval == Self::OUTCOME_WIN || eval == Self::OUTCOME_LOSS {
            return eval - (max_depth - depth) as i32;
        }
        let large = ((share >> 36) | (share >> 45)) & Ut3Board::CHUNK;
        if large == Ut3Board::CHUNK {
            return Self::OUTCOME_DRAW;
        }
        (0..7)
            .map(|i| {
                let us_data = (us >> (9 * i)) & Ut3Board::CHUNK;
                let them_data = (them >> (9 * i)) & Ut3Board::CHUNK;

                if ((large >> i) & 1) == 1 || (us_data | them_data) == Ut3Board::CHUNK {
                    0
                } else {
                    self.small_table[((them_data << 9) | us_data) as usize]
                }
            })
            .chain((7..9).map(|i| {
                let us_data = (share >> (9 * i - 63)) & Ut3Board::CHUNK;
                let them_data = (share >> (9 * i - 45)) & Ut3Board::CHUNK;

                if ((large >> i) & 1) == 1 || (us_data | them_data) == Ut3Board::CHUNK {
                    0
                } else {
                    self.small_table[((them_data << 9) | us_data) as usize]
                }
            }))
            .fold(eval, |acc, x| acc + x)
    }
}