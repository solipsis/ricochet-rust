use std::collections::HashMap;

struct Board {
    tiles: Vec<i32>,
    initial_robots: Vec<Robot>,
    width: i32,
    height: i32,
    goal: Goal,
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
    target_robot_id: char,
    cache: HashMap<i32, i32>,
    move_stack: Vec<Move>,
}


impl Solver {

    fn new(board: Board) -> Self {

        let mut robots: HashMap<char, Robot> = HashMap::new();
        let target_robot_id = board.goal.robot_id;
        for robot in &board.initial_robots {
            robots.insert(robot.id, robot.clone());
        }

        return Solver {
            board,
            robots,
            cache: HashMap::new(),
            move_stack: Vec::new(),
            target_robot_id,
        }
    }

    fn solve(&mut self, max_depth: u8) -> bool {
        // iterative deepening DFS 
        let mut current_max_depth = 1;
        while current_max_depth < max_depth {
            let solved = self.dfs(0, current_max_depth as i32);
            if solved {
                println!("Solved");
                return true
            }

            current_max_depth += 1;
        }
        println!("failed");
        return false;
    }

    fn dfs(&mut self, depth: i32, max_depth: i32) -> bool {
        // are we done

        // TODO: Precompute 
        //

        //let mut depth: i32 = 0;
        while depth < max_depth {

            // are we done
            if self.robots[&self.target_robot_id].position == self.board.goal.postion {
                return true;
            }

            // TODO: Are we within precompute bounds
            
            // TODO: Check state cache

           // if !self.cache.contains_key(self.compute_hash()) || self.cache
            
            // check cache of previously seen states
            let hash = self.compute_hash();
         //   let previous_best = self.cache.get(&hash).get_or_insert(&0);
            let previous_best = 1;
            if previous_best >= max_depth - depth {
                // We have already been to this state in the past with more moves remainng
                return false; 
            }
            self.cache.insert(hash, max_depth - depth);




            for id in &IDS {
         //  self.robots.
         //   for (id, robot) in &(self.robots) {
                for direction in DIRECTIONS {

                 //   let robot = self.robots.get(id).unwrap();
                    let previous_position = self.robots.get(id).unwrap().position;
                   // let previous_position = robot.position;

                    if !self.move_robot(*id, direction) {
                        continue;
                    }

                    let m = Move{ robot_id: *id, direction: direction};
                    self.move_stack.push(m);

                    // recurse
                    if self.dfs(depth+1, max_depth) {
                        return true;
                    }

                    // undo this move
                    let robot = self.robots.get_mut(id).unwrap();
                    self.board.tiles[previous_position as usize] ^= ROBOT;
                    self.board.tiles[robot.position as usize] ^= ROBOT;
                    robot.position = previous_position;
                    self.move_stack.pop();
                }
            }

        }

        return false;
    }

    fn compute_hash(&self) -> i32 {
        let mut hash = self.robots[&'R'].position;
        hash |= self.robots[&'B'].position << 8;
        hash |= self.robots[&'G'].position << 16;
        hash |= self.robots[&'Y'].position << 24;

        hash
    }

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
            let is_same_robot = robot.id == last_move.robot_id;
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


#[test]
fn test_fail_solve() {
    let tiles = vec![ UP | LEFT,      UP,        UP | RIGHT,
                           LEFT,       0,             RIGHT,
                    LEFT | DOWN,    DOWN,      DOWN | RIGHT];
    let initial_robots = vec![Robot{ id: 'A', position: 0}, Robot{ id: 'B', position: 1}, Robot{ id: 'Y', position: 2}, Robot{ id: 'G', position: 3}];
    let goal = Goal{ robot_id: 'A', postion: 8 };
    let width = 3;
    let height = 3;

    let board = Board{
        tiles,
        initial_robots,
        width,
        height,
        goal,
    };

    let mut solver = Solver::new(board);
    solver.solve(1);


}

#[derive(Debug, Clone)]
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

const DIRECTIONS: [i32; 4] = [UP, DOWN, LEFT, RIGHT];
const IDS: [char; 4] = ['R', 'G', 'B', 'Y'];

fn main() {
    println!("Hello, world!");
}
