use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;
use std::collections::VecDeque;
use std::ops::{Add, RangeBounds};
use rand::thread_rng;
use rand::seq::SliceRandom;
mod test;

#[wasm_bindgen]
#[derive(Copy, Clone)]
struct Board {
    block_pixel_size: u32,
    block_width: u32,
    block_height: u32,
}

//#[wasm_bindgen]
impl Board {
    fn draw_square (&self, ctx: &CanvasRenderingContext2d, row: u32, col: u32, color: &JsValue) {
        ctx.set_fill_style(color);
        ctx.fill_rect(
            (col * self.block_pixel_size) as f64, 
            (row * self.block_pixel_size) as f64,
            self.block_pixel_size as f64,
            self.block_pixel_size as f64);
    } 

    pub fn clear_board(&self, ctx: &CanvasRenderingContext2d){
        ctx.clear_rect(0.0, 0.0, 
            (self.block_width * self.block_pixel_size) as f64,
            (self.block_height * self.block_pixel_size) as f64)
    }

    pub fn test_board_draw(&self, ctx: &CanvasRenderingContext2d) {
        self.draw_square(ctx, 1,1, &JsValue::from_str("black"));
    }
}

// In a separate block because we want a pub fn without exposing it to WASM
impl Board {
    pub fn draw_piece(&self, ctx: &CanvasRenderingContext2d, piece: &Piece) {
        self.draw_square(ctx, 
            piece.row,
            piece.col,
            &piece.piece_type.color())
    }
}

enum DirectionEnum {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq)]
struct DirectionVec {
    row: i8, 
    col: i8 
}

impl DirectionEnum {
    pub fn direction_vector(&self) -> DirectionVec {
        match *self {
            DirectionEnum::Up => DirectionVec{row: -1, col: 0},
            DirectionEnum::Down => DirectionVec{row: 1, col: 0},
            DirectionEnum::Left => DirectionVec{row: 0, col: -1},
            DirectionEnum::Right => DirectionVec{row: 0, col: 1}
        }
    }

    pub fn from_string(direction: &str) -> Option<Self> {
        match direction {
            "ArrowUp" => Some(Self::Up),
            "ArrowDown" => Some(Self::Down),
            "ArrowLeft" => Some(Self::Left),
            "ArrowRight" => Some(Self::Right),
            _ => None,
        }
    }
}

impl Add for DirectionVec {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            row: self.row + other.row,
            col: self.col + other.col,
        }
    }
}

#[derive(Clone)]
enum PieceType {
    Food,
    Tail,
    Head,
}

impl PieceType {
    pub fn color(&self) -> JsValue {
        match *self {
            PieceType::Food => JsValue::from_str("blue"),
            PieceType::Tail => JsValue::from_str("green"),
            PieceType::Head => JsValue::from_str("red")
        }
    }
}

#[derive(Clone)]
struct Piece {
    piece_type: PieceType,
    row: u32,
    col: u32
}

impl Piece {
    pub fn to_tail(&self) -> Piece {
        Piece {
            piece_type: PieceType::Tail,
            row: self.row,
            col: self.col
        }
    }

    pub fn same_pos(&self, rhs: &Piece) -> bool {
        self.col == rhs.col && self.row == rhs.row
    }
}

struct Snake {
    dir: DirectionEnum,
    head: Piece,
    tail: VecDeque<Piece>
}

#[wasm_bindgen]
struct Game {
    pub board: Board,
    snake: Snake,
    food: Piece,
    gameover: bool
}

#[wasm_bindgen]
impl Game {
    pub fn new(ctx: &CanvasRenderingContext2d, block_pixel_size: u32, block_width: u32, block_height: u32) -> Game {
        let head_row = block_height/2 + 3;
        let head_col = block_width/2;
        
        let mut tail = VecDeque::<Piece>::new();
        for i in (1..3).rev() {
            tail.push_back(
                Piece{piece_type: PieceType::Tail, row: head_row + (i as u32), col: head_col}
            );
        }
        let game = Game {
            board: Board {block_pixel_size, block_width, block_height},
            snake: Snake { 
                dir: (DirectionEnum::Up), 
                head: Piece {piece_type: PieceType::Head, row: head_row, col: head_col}, 
                tail },
            food: Piece {piece_type: PieceType::Food, row: head_row - 4, col: head_col},
            gameover: false,
        };

        game.draw_pieces(ctx);
        game  
    } 

    pub fn step(&mut self, ctx: &CanvasRenderingContext2d, key: String) {
        self.logic(&key);
        self.draw_pieces(ctx);
    }

    pub fn get_gameover(&self) -> bool {
        self.gameover
    }
}

// New impl block so that we don't expose `pub fn logic` to WASM
impl Game {
    pub fn logic(&mut self, key: &String) {
        // update direction
        // first check if new direction is opposite
        let new_dir = match DirectionEnum::from_string(key) {
            Some(value) => value,
            None => {
                test::log("Error setting key");
                DirectionEnum::Up
            }
        };

        let dir_vec_sum = self.snake.dir.direction_vector() + new_dir.direction_vector();
        if !(dir_vec_sum == DirectionVec{row: 0, col: 0}) {
            self.snake.dir = new_dir;
        }

        // update head position
        let old_head_as_tail = self.snake.head.to_tail();
        let (drow, dcol) = match self.snake.dir {
            DirectionEnum::Down => (1, 0),
            DirectionEnum::Up => (-1, 0),
            DirectionEnum::Right => (0, 1),
            DirectionEnum::Left => (0, -1)
        };

        self.snake.head.col = ((self.snake.head.col as i32) + dcol) as u32;
        self.snake.head.row = ((self.snake.head.row as i32) + drow) as u32;

        //check food collision
        let mut food_collision: bool = false;
        if self.food.same_pos(&self.snake.head) {
            food_collision = true;

            let mut occupied_points = Vec::new();
            occupied_points.push((
                self.snake.head.row,
                self.snake.head.col
            ));
            occupied_points.push((
                old_head_as_tail.row,
                old_head_as_tail.col
            ));
            for item in &self.snake.tail{
                occupied_points.push(
                    (item.row, item.col)
                )
            }

            let (new_food_row, new_food_col) = self.random_unoccupied_point(&occupied_points);
            self.food.row = new_food_row;
            self.food.col = new_food_col;
        }
        
        // check if outside walls
        if (self.snake.head.col >= self.board.block_width) || (self.snake.head.row >= self.board.block_height) {
            self.gameover = true;
        }
        
        // update tail
        self.snake.tail.push_back(old_head_as_tail);
        if !food_collision {
            self.snake.tail.pop_front();
        }
        
        // check tail collisions
        if self.snake.tail.iter().any(|piece| piece.same_pos(&self.snake.head)) {
            self.gameover=true;
        }
    }

    fn draw_pieces(&self, ctx: &CanvasRenderingContext2d) {
        self.board.clear_board(ctx);
        self.board.draw_piece(ctx, &self.snake.head);
        for item in &self.snake.tail{
            self.board.draw_piece(ctx, item);
        }
        self.board.draw_piece(ctx, &self.food);
    }

    fn random_unoccupied_point(&self, occupied_points: &Vec<(u32, u32)>) -> (u32, u32) {

        let mut unoccupied_points = Vec::new();
        for i in 0..self.board.block_height {
            for j in 0..self.board.block_width {
                if !occupied_points.contains(&(i, j)) {
                    unoccupied_points.push((i, j));
                }
            }
        }

        let mut rng = thread_rng();
        match unoccupied_points.choose(&mut rng) {
            Some(val) => *val,
            None => {
                test::log("Error with random number generation");
                (0, 0)
            }
        }
            
    }
}