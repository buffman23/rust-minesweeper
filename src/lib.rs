
use rand::prelude::*;
use std::collections::HashSet;
use std::hash::Hash;
use std::{fmt, cmp};
use std::mem;

pub const EMPTY: u8 = 0;
pub const BOMB: u8 = 9;
pub const TILE_UP: u8 = 10;
pub const FLAGGED: u8 = 20;

#[derive(Debug)]
pub struct TileSymbols {
    pub empty: char,
    pub bomb: char,
    pub tile_up: char,
    pub flagged: char,
    pub flagged_bomb: char,
    pub sweeped_bomb: char,
}

pub struct MineField {
    grid: Box<Vec<u8>>,
    width: usize,
    height: usize,
    bomb_count: i32,
    bombs_left: i32,
    rng: StdRng,
    first_sweep: bool,
    tile_symbols: TileSymbols,
}


#[derive(Debug, Clone, Copy)]
#[derive(Eq, Hash, PartialEq)]
pub struct Tile {
    pub x: usize,
    pub y: usize,
    pub val: u8,
}

pub struct MineFieldItr<'field> {
    field : &'field MineField,
    curr_x : usize,
    curr_y : usize,
}

pub struct MutTile<'field> {
    pub x: usize,
    pub y: usize,
    pub val: &'field mut u8,
}

#[derive(Debug)]
pub struct MutMineFieldItr<'field> {
    grid: &'field mut [u8],
    curr_x: usize,
    curr_y: usize,
    width: usize,
}

impl TileSymbols {
    pub const DEFAULT: i32 = 0;
    pub const ASCII: i32 = 1;
    pub const EMOJI: i32 = 2;

    pub fn new(style : i32) -> TileSymbols {
        match style {
            1 => TileSymbols {
                empty: ' ',
                bomb: '*',
                tile_up: 'o',
                flagged: '>',
                flagged_bomb: 'x',
                sweeped_bomb: '!',
            },
            2 => TileSymbols {
                empty: ' ',
                bomb: '💣',
                tile_up: '▢',
                flagged: '🚩',
                flagged_bomb: '❌',
                sweeped_bomb: '💥',
            },
            _ => TileSymbols {
                empty: ' ',
                bomb: '☀',
                tile_up: '▢',
                flagged: '>',
                flagged_bomb: 'x',
                sweeped_bomb: '!',
            },
        }
    }
}

impl<'field> MutTile<'field> {
    pub fn is_empty(&self) -> bool{
        *self.val == EMPTY
    }

    pub fn is_up(&self) -> bool{
        *self.val >= TILE_UP
    }

    pub fn is_flagged(&self) -> bool{
        *self.val >= FLAGGED
    }

    pub fn up(&mut self) {
        if !self.is_up() {
            *self.val = *self.val + TILE_UP;
        }
    }

    pub fn down(&mut self) {
        if self.is_up() && !self.is_flagged() {
            *self.val = *self.val - TILE_UP;
        }
    }

    pub fn toggle_flag(&mut self) {
        if *self.val < FLAGGED {
            *self.val = *self.val + TILE_UP;
        } else {
            *self.val = *self.val - TILE_UP;
        }
    }
}

impl Tile {
    pub fn new(x: usize, y: usize, val: u8) -> Tile {
        Tile {
            x: x,
            y: y,
            val: val,
        }
    }

    pub fn is_empty(&self) -> bool{
        self.val == EMPTY
    }

    pub fn is_up(&self) -> bool{
        self.val >= TILE_UP
    }

    pub fn is_flagged(&self) -> bool{
        self.val >= FLAGGED
    }

    pub fn is_bomb(&self) -> bool{
        self.val == BOMB
    }
}

impl<'field> MutMineFieldItr<'field> {
    fn new(field: &'field mut MineField) -> MutMineFieldItr<'field> {
        MutMineFieldItr {
            grid: field.grid.as_mut_slice(),
            curr_x: 0,
            curr_y: 0,
            width: field.width,
        }
    }
}

impl<'field> Iterator for MutMineFieldItr<'field>{
    type Item = MutTile<'field>;

    fn next(&mut self) -> Option<Self::Item> {

        let slice = mem::replace(&mut self.grid, &mut []);

        if slice.is_empty() { return None; }

        let (l, r) = slice.split_at_mut(1);
        self.grid = r;

        let tile = l.get_mut(0).map(|val| MutTile {
            x: self.curr_x,
            y: self.curr_y,
            val:  val,
        });
        
        self.curr_x = self.curr_x + 1;
        if self.curr_x >= self.width {
            self.curr_x = 0;
            self.curr_y = self.curr_y + 1;
        }

        tile
    }
}

impl Iterator for MineFieldItr<'_> {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_y == self.field.height {
            return None;
        }

        let op = Some(Tile {
            x: self.curr_x,
            y: self.curr_y,
            val: self.field.get_value(self.curr_x, self.curr_y),
        });

        self.curr_x = self.curr_x + 1;

        if self.curr_x == self.field.width {
            self.curr_x = 0;
            self.curr_y = self.curr_y + 1;
        }

        op
    }
}

trait BombCounter {
    fn bomb_count(&self) -> u8;
}
impl BombCounter for Vec<Tile> {
    fn bomb_count(&self) -> u8 {
        self.iter().map(|tile| match tile.val {
            BOMB => 1,
            _ => 0,
        }).sum()
    }
}

impl MineField {
    pub fn new(width: usize, height: usize) -> Self {
        let mut field = MineField {
            grid : Box::new(vec![0u8; width*height]),
            width: width,
            height: height,
            bomb_count: 0,
            bombs_left: 0,
            first_sweep: true,
            rng : StdRng::from_entropy(),
            tile_symbols: TileSymbols::new(TileSymbols::DEFAULT),
        };

        field.bomb_count = (field.area() as f32 * 0.33 + 0.5) as i32; // default bomb count (33% field bomb density)
        //field.clear();
        field.reset();

        field
    }

    pub fn get_value_checked(&self, x: usize, y: usize) -> Option<u8> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.get_value(x, y))
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get_value(&self, x: usize, y: usize) -> u8 {
        return self.grid[x + y * self.height];
    }

    pub fn set_value(&mut self, x: usize, y: usize, value: u8) {
        self.grid[x + y * self.height] = value;
    }

    pub fn inc_value(&mut self, x: usize, y: usize, value: u8) {
        self.grid[x + y * self.height] = self.grid[x + y * self.height] + value;
    }

    pub fn set_value_ckecked(&mut self, x: usize, y: usize, value: u8) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }

        self.grid[x + y * self.height] = value;

        true
    }

    pub fn set_random_seed(&mut self) {
        self.rng = StdRng::from_entropy();
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.rng = StdRng::seed_from_u64(seed);
    }
    
    pub fn populate(&mut self, sweep_x: i32, sweep_y: i32, radius: i32) {
        self.clear();
        let mut locations: Vec<usize> = (0..(self.area())).collect();
        
        locations.retain(|location| {
            let x = location % self.width;
            let y = location / self.width;

            !(x as i32 >= sweep_x - radius && x as i32 <= sweep_x + radius &&
            y as i32 >= sweep_y - radius && y as i32 <= sweep_y + radius)
        });

        self.bombs_left = cmp::min(self.bomb_count, locations.len() as i32);
        
        
        for _i in 0..self.bombs_left {
            let (idx, location) = locations.iter().enumerate().choose(&mut self.rng).unwrap();
            
            let x = location % self.width;
            let y = location / self.width;
            
            locations.swap_remove(idx);

            self.inc_value(x, y, BOMB);
        }

        // generate number tiles
        for y in 0..(self.height) {
            for x in 0..(self.width) {
                if self.get_value(x, y) != BOMB {
                    self.inc_value(x, y, self.neighbors(x, y).bomb_count());
                }
            }
        }

        // cover field with up tiles
        self.iter_mut().for_each(|tile| *tile.val += TILE_UP);
    }

    pub fn clear(&mut self) {
        self.iter_mut().for_each(|tile| *tile.val = 0);
    }

    #[inline]
    pub fn area(&self) -> usize {
        return self.width * self.height;
    }

    pub fn flag(&mut self, x: usize, y: usize) -> bool {
        let val = self.get_value(x, y);

        if val < TILE_UP {
            return false
        }

        if val < FLAGGED {
            self.set_value(x, y, val + FLAGGED);
            self.bombs_left = self.bombs_left - 1;
        } else {
            self.set_value(x, y, val - FLAGGED);
            self.bombs_left = self.bombs_left + 1 ;
        }
        
        val < FLAGGED
    }

    pub fn set_bomb_count(&mut self, count: i32) {
        self.bomb_count = count;
    }

    pub fn set_bomb_density(&mut self, density: f32) {
        self.set_bomb_count((self.area() as f32 * density) as i32);
    }


    pub fn sweep(&mut self, x: usize, y: usize) -> Vec<Tile> {

        if self.first_sweep {
            self.first_sweep = false;
            self.populate(x as i32, y as i32, 1);
        }

        let mut stack = Vec::<Tile>::new();
        let mut sweeped = Vec::<Tile>::new();


        let val = self.get_value(x, y);

        if val < TILE_UP || val >= FLAGGED {
            return sweeped;
        }

        stack.push(Tile::new(x, y, val));

        while stack.len() != 0 {
            let mut tile = stack.pop().unwrap();
            
            tile.val = tile.val - TILE_UP;
            self.set_value(tile.x, tile.y, tile.val);
            sweeped.push(tile);

            if tile.val != EMPTY {

                continue;
            }

            self.neighbors(tile.x, tile.y).drain(..).for_each(|tile| {
                if tile.is_up() && !tile.is_flagged() {
                    stack.push(tile);
                }
            });
        }

        sweeped
    }

    pub fn reveal(&mut self) {
        self.iter_mut().for_each(|mut tile| tile.down());
    }

    pub fn get_perimiter(&self, tiles: &Vec<Tile>) -> Vec<Tile> {
        let mut set: HashSet::<Tile> = HashSet::with_capacity(4 * tiles.len());

        tiles.iter()
        .filter(|tile| !tile.is_empty())
        .for_each(|tile| self.neighbors(tile.x, tile.y).iter()
            .filter(|tile| tile.is_up())
            .for_each(|tile2| {
                set.insert(*tile2);
        }));

        set.into_iter().collect()
    }

    pub fn neighbors(&self, x: usize, y: usize) -> Vec<Tile> {
        let mut vec: Vec::<Tile> = Vec::with_capacity(8);

        for i in 0..3 {
            for j in 0..3 {
                let x2 = (i as i32).wrapping_sub(1).wrapping_add(x as i32) as usize;
                let y2 = (j as i32).wrapping_sub(1).wrapping_add(y as i32) as usize;

                if x2 == x && y2 == y {
                    continue;
                }

                if let Some(val) = self.get_value_checked(x2, y2) {
                    vec.push(Tile::new(x2, y2, val));
                }
            }
        }

        vec
    }

    pub fn iter(&self) -> MineFieldItr{
        MineFieldItr {
            field : self,
            curr_x : 0,
            curr_y : 0,
        }
    }

    pub fn iter_mut(&mut self) -> MutMineFieldItr{
        MutMineFieldItr::new(self)
    }

    pub fn reset(&mut self) {
        self.iter_mut().for_each(|tile| *tile.val = TILE_UP);
        self.bombs_left =  self.bomb_count;
        self.first_sweep = true;
    }

    pub fn to_string(&self) -> String {
        let mut s =  String::new();
        let x_padding = self.width.to_string().len();
        let y_padding = self.height.to_string().len();

        s.reserve(self.area() + "bombs left".len() + 5);

        s.push_str(&format!("Bombs left: {}\n", self.bombs_left));

        for y in 0..(self.height) {
            s.push_str(format!("{:>1$}|", self.width - y, y_padding).as_str());
            for x in 0..(self.width) {
                let num = self.get_value(x, y);
                
                let c: char = if num == 0 {
                    self.tile_symbols.empty
                }else if num < BOMB {
                    std::char::from_digit(num as u32 , 10).unwrap()
                } else if num == BOMB {
                    self.tile_symbols.bomb
                }else if num < FLAGGED {
                   self.tile_symbols.tile_up
                } else {
                    self.tile_symbols.flagged
                };
                
                s.push(c);
                if x == self.width - 1 {
                    s.push('\n');
                } else {
                    s.push(' ');
                }
            }
        }

        s.push_str(&" ".repeat(y_padding + 1));
        s.push_str(&"-".repeat(self.width * 2 - 1));
        s.push('\n');

        for i in 0..x_padding {
            let pow = 10_i32.pow(i as u32) as usize;
            s.push_str(format!("{:>1$} ", ' ', y_padding).as_str());
            for j in 1..=self.width {
                if j >= pow {
                    s.push(j.to_string().chars().nth(i).unwrap());
                } else {
                    s.push(' ');
                }
                s.push(' ');
            }
            s.push('\n');
        }

        s
    }

}

impl fmt::Display for MineField {
    
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())?;

        Ok(())
    }
}


#[cfg(test)]
mod tests
{
    use crate::MineField;

    #[test]
    fn iter_test() {
        let mut field = MineField::new(5, 5);
        field.populate(2, 2, 0);

        field.iter().for_each(|tile| assert_eq!(tile.val, field.get_value(tile.x, tile.y)))
    }

    #[test]
    fn mut_itr_test() {
        let mut field = MineField::new(5, 5);
    
        field.iter_mut().for_each(|tile| *tile.val = tile.x as u8);
    
        for y in 0..field.height() {
            for x in 0..field.width() {
                let val = field.get_value(x, y);
                assert_eq!(val, x as u8);
            }
        }

        field.iter_mut().for_each(|tile| *tile.val = tile.y as u8);

        for y in 0..field.height() {
            for x in 0..field.width() {
                let val = field.get_value(x, y);
                assert_eq!(val, y as u8);
            }
        }
    }
}