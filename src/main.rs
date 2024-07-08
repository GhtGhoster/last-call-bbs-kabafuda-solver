use std::{collections::HashSet, hash::Hash, thread::sleep, time::Duration};

use enigo::{Coordinate, Direction, Enigo, Key, Mouse, Keyboard, Settings};
use screenshots::{image::{io::Reader, DynamicImage}, Screen};

const CARD_X: i32 = 470;
const CARD_Y: i32 = 474;
const CARD_W: i32 = 128;
const CARD_H: i32 = 30;

#[derive(Debug, Clone, Copy)]
enum Difficulty {
    Easy = 0,
    Medium = 1,
    Hard = 2,
    Expert = 3,
}

impl Difficulty {
    fn get_coords(&self) -> (i32, i32) {
        let mut x = 925;
        let mut y = 555;
        if *self as usize % 2 == 1 {
            x = 1115;
        }
        if *self as usize > 1 {
            y = 685;
        }
        (1920 + x, y)
    }
}

#[derive(Clone)]
struct Matrix {
    stacks: Vec<Vec<u8>>,
    difficulty: Difficulty,
    slots: Vec<Vec<u8>>,
}

impl Matrix {
    fn from_screen(difficulty: Difficulty) -> Self {
        let screens = Screen::all().unwrap();
        let screen = screens[0];
        let mut stacks: Vec<Vec<u8>> = vec![];
        for j in 0..8 {
            stacks.push(vec![]);
            for i in 0..5 {
                let image = DynamicImage::ImageRgba8(
                    screen.capture_area(
                        CARD_X + (CARD_W * j),
                        CARD_Y + (CARD_H * i),
                        20,
                        20,
                    ).unwrap()
                );
                for n in 0..10 {
                    let comparison_image = Reader::open(format!("assets/{n}.png")).unwrap().decode().unwrap();
                    if comparison_image == image {
                        stacks.last_mut().unwrap().push(n);
                        break;
                    }
                }
            }
        }
        let mut slots = vec![];
        for _ in 0..(4-difficulty as usize) {
            slots.push(vec![]);
        }
        Self {
            stacks,
            difficulty,
            slots,
        }
    }

    fn available_moves(&self) -> Vec<Move> {
        let mut moves = vec![];

        // determine moves from stacks
        for (from, stack_from) in self.stacks.iter().enumerate() {
            // skip empty stacks
            if stack_from.is_empty() {
                continue;
            }

            // how many cards can I move at once from this stack
            let mut max_count = 0;
            for i in 0..4.min(stack_from.len()) {
                if stack_from.iter().rev().nth(i).unwrap() != stack_from.last().unwrap() {
                    break;
                }
                max_count += 1;
            }

            // skip locked stacks
            if max_count == 4 && stack_from.len() == 4 {
                continue;
            }

            // what stacks can I move to
            for (to, stack_to) in self.stacks.iter().enumerate() {
                if to == from {
                    continue;
                }
                if stack_to.is_empty() || stack_to.last().unwrap() == stack_from.last().unwrap() {
                    for i in 0..max_count {
                        let count = i + 1;
                        moves.push(Move {
                            from,
                            to,
                            count,
                        });
                    }
                }
            }

            // what slots can I move to
            for (to, slot_to) in self.slots.iter().enumerate() {
                let to = 8 + to;
                if slot_to.is_empty() {
                    // move one card
                    moves.push(Move {
                        from,
                        to,
                        count: 1,
                    });

                    // move whole locked stack
                    if max_count == 4 {
                        moves.push(Move {
                            from,
                            to,
                            count: 4,
                        });
                    }
                }
            }
        }

        // determine moves from slots
        for (from, slot_from) in self.slots.iter().enumerate() {
            let from = 8 + from;

            // skip empty and full slots
            if slot_from.len() != 1 {
                continue;
            }

            // determine moves to stacks (skip moving to another slot)
            for (to, stack_to) in self.stacks.iter().enumerate() {
                if stack_to.is_empty() || stack_to.last().unwrap() == &slot_from[0] {
                    moves.push(Move {
                        from,
                        to,
                        count: 1,
                    });
                }
            }
        }
        moves
    }

    fn score(&self) -> usize {
        let mut score = 0;
        for stack in &self.stacks {
            let mut max_count = 0;
            for i in 0..4.min(stack.len()) {
                if stack.iter().rev().nth(i).unwrap() != stack.last().unwrap() {
                    break;
                }
                max_count += 1;
            }
            score += 2usize.pow(max_count);
        }
        for slot in &self.slots {
            // if slot.len() == 4 {
            //     score += 4;
            // }
            score += 2usize.pow(slot.len() as u32);
        }
        score
    }

    fn make_move(&self, game_move: Move) -> Self {
        let mut matrix = self.clone();

        // make the move (reap what you sow, stupid indexing system)
        let mut from = game_move.from;
        let mut to = game_move.to;
        if game_move.from >= 8 {
            from -= 8;
            if game_move.to >= 8 {
                to -=8;
                let card = matrix.slots[from].pop().unwrap();
                matrix.slots[to].push(card);
            } else {
                let card = matrix.slots[from].pop().unwrap();
                matrix.stacks[to].push(card);
            }
        } else {
            if game_move.to >= 8 {
                to -=8;
                for _ in 0..game_move.count {
                    let card = matrix.stacks[from].pop().unwrap();
                    matrix.slots[to].push(card);
                }
            } else {
                for _ in 0..game_move.count {
                    let card = matrix.stacks[from].pop().unwrap();
                    matrix.stacks[to].push(card);
                }
            }
        }

        matrix
    }

    fn to_hash_string(&self) -> String {
        // collect stack and slot data
        let mut stack_strings: Vec<String> = vec![];
        for stack in &self.stacks {
            let mut buff = String::new();
            for item in stack {
                buff += item.to_string().as_str();
            }
            buff += "T";
            stack_strings.push(buff);
        }
        let mut slot_strings: Vec<String> = vec![];
        for slot in &self.slots {
            let mut buff = String::new();
            for item in slot {
                buff += item.to_string().as_str();
            }
            buff += "T";
            slot_strings.push(buff);
        }

        // disregard order of stacks and slots
        stack_strings.sort();
        slot_strings.sort();

        // generate hashable string
        let mut ret = String::new();
        for stack_string in stack_strings {
            ret += &stack_string;
        }
        for slot_string in slot_strings {
            ret += &slot_string;
        }
        ret
    }
}

impl Hash for Matrix {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_hash_string().hash(state);
    }
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool {
        self.to_hash_string() == other.to_hash_string()
    }
}

impl Eq for Matrix{}

#[derive(Debug, Clone, Copy)]
struct Move {
    from: usize,
    to: usize,
    count: usize,
}

fn main() {
    // settings
    let difficulty = Difficulty::Easy;
    
    // initialize
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    // gain focus
    enigo.move_mouse(1920 + 5, 5, Coordinate::Abs).unwrap();
    sleep(Duration::from_millis(50));
    enigo.button(enigo::Button::Left, enigo::Direction::Click).unwrap();
    sleep(Duration::from_millis(50));

    // start new game
    enigo.key(Key::Control, Direction::Press).unwrap();
    sleep(Duration::from_millis(50));
    enigo.key(Key::Unicode('n'), Direction::Click).unwrap();
    sleep(Duration::from_millis(50));
    enigo.key(Key::Control, Direction::Release).unwrap();
    sleep(Duration::from_millis(50));

    // select difficulty
    let (x, y) = difficulty.get_coords();
    enigo.move_mouse(x, y, Coordinate::Abs).unwrap();
    sleep(Duration::from_millis(50));
    enigo.button(enigo::Button::Left, enigo::Direction::Click).unwrap();
    sleep(Duration::from_millis(50));

    // move cursor out of the way and wait
    enigo.move_mouse(1920 + 5, 5, Coordinate::Abs).unwrap();
    sleep(Duration::from_millis(7000));

    // detect starting position
    let matrix = Matrix::from_screen(difficulty);

    // solve
    let mut past_matrices: HashSet<Matrix> = HashSet::new();
    past_matrices.insert(matrix.clone());
    let past_moves: Vec<Move> = vec![];
    let solution = solve(matrix, &mut past_matrices, &past_moves).unwrap();

    for past_move in solution {
        println!("{past_move:?}");
    }
}

fn solve(matrix: Matrix, past_matrices: &mut HashSet<Matrix>, past_moves: &Vec<Move>) -> Option<Vec<Move>> {
    let mut possible_futures = vec![];
    for available_move in matrix.available_moves() {
        let new_matrix = matrix.make_move(available_move);
        if !past_matrices.contains(&new_matrix) {
            let new_score = new_matrix.score();
            possible_futures.push((new_matrix, new_score, available_move));
        }
    }
    if possible_futures.is_empty() {
        return None;
    }
    possible_futures.sort_by(|a, b| b.1.cmp(&a.1));
    for (new_matrix, new_score, new_move) in possible_futures {
        let mut new_past_moves = past_moves.clone();
        new_past_moves.push(new_move);
        if new_score == 160 {
            return Some(new_past_moves);
        }
        past_matrices.insert(new_matrix.clone());
        let next_step = solve(new_matrix, past_matrices, past_moves);
        if next_step.is_some() {
            return next_step;
        }
    }
    None
}
