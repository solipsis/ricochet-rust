use std::collections::HashMap;

struct Board {
    tiles: Vec<i32>,
    initial_robots: Vec<Robot>,
    width: i32,
    height: i32,
}

impl Board {
    fn offset(&self, direction: i32) -> i32 {
        match direction {
            UP => -1 * self.width,
            DOWN => self.width,
            LEFT => -1,
            RIGHT => 1,
            _ => panic!("Invalid Direction"),
        }
    }

    fn has_wall(&self, location: i32, direction: i32) -> bool {
        self.tiles[location as usize] & direction != 0
    }

    fn has_robot(&self, location: i32) -> bool {
        self.tiles[location as usize] & ROBOT != 0
    }

}

struct Solver {
    board: Board,
    robots: HashMap<char, Robot>,
    active_robot_id: char,
    goal: Goal,
    cache: HashMap<i32, i32>,
    move_stack: Vec<Move>,
}

impl Solver {

    fn move_robot(&mut self, id: char, direction: i32) -> bool {

        let robot = self.robots.get_mut(&id).unwrap();
        let start_tile = robot.position;

        /*
        // can't move
        if self.board.has_wall(robot.position, direction) {
            return false;
        }
        */

        // disallow undoing previous move because it is never optimal
        if let Some(last_move) = self.move_stack.last() {
            let is_same_robot = id == last_move.robot_id;
            let is_reverse_movement = reverse(last_move.direction) == direction;

            if is_same_robot && is_reverse_movement {
                return false;
            }
        }

        /*
        // if next square has a robot, can't move
        let mut next_tile = robot.position + self.board.offset(direction);
        if self.board.has_robot(next_tile) {
            return false;
        }
        */

        // move until the current square has a wall or the next square has a robot
        //let mut next_tile = robot.position + self.board.offset(direction);
        let mut end_tile = robot.position;
        let offset = self.board.offset(direction);
        loop {
            if self.board.has_wall(end_tile, direction) {
                break;
            }

            if self.board.has_robot(end_tile + offset) {
                break;
            }
            end_tile += offset;
        }

        // did we move
        if end_tile == start_tile {
            return false;
        }

        self.board.tiles[start_tile as usize] ^= ROBOT;
        self.board.tiles[end_tile as usize] ^= ROBOT;
        robot.position = end_tile;

        return true
    }
}

struct Robot {
    id: char,
    position: i32,
}

struct Goal {
    robot_id: char,
    postion: i32,
}

struct Move {
    robot_id: char,
    direction: i32,
}


fn reverse(direction: i32) -> i32 {
    match direction {
        UP => DOWN,
        DOWN => UP,
        LEFT => RIGHT,
        RIGHT => LEFT,
        _ => panic!("Invalid Direction"),
    }
}



const UP: i32 = 0x01;
const DOWN: i32 = 0x02;
const LEFT: i32 = 0x03;
const RIGHT: i32 = 0x04;
const ROBOT: i32 = 0x05;


fn main() {
    println!("Hello, world!");
}
