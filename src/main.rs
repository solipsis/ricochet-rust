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
    precomputed_target_moves: Vec<i32>,
    move_stack: Vec<Move>,
}


impl Solver {

    fn new(mut board: Board) -> Self {

        let mut robots: HashMap<char, Robot> = HashMap::new();
        let target_robot_id = board.goal.robot_id;
        for robot in &mut board.initial_robots {
            robots.insert(robot.id, robot.clone());
            board.tiles[robot.position as usize] |= ROBOT;

        }

        let mut solver = Solver {
            board,
            robots,
            cache: HashMap::new(),
            move_stack: Vec::new(),
            target_robot_id,
            precomputed_target_moves: Vec::new(),
        };
        solver.precompute_target_moves();
        return solver;


    }

    // calculate the minimum number of moves it would take the target to get to the goal
    // if the piece could move like a rook (stopping arbitrarily). This helps dramatically
    // prune the search space
    fn precompute_target_moves(&mut self) {

        let mut optimal_moves: Vec<i32> = Vec::with_capacity(self.board.tiles.len());
        let mut active_cells: Vec<bool> = Vec::with_capacity(self.board.tiles.len());

        let infinity = 999999;
        for _ in 0..self.board.tiles.len() {
            optimal_moves.push(infinity);
            active_cells.push(false);
        }

        optimal_moves[self.board.goal.position as usize] = 0;
        active_cells[self.board.goal.position as usize] = true;

        let mut done = false;
        while !done {
            done = true;

            for idx in 0..self.board.tiles.len() {

                if !active_cells[idx] {
                    continue;
                }
                active_cells[idx] = false;

                let score = optimal_moves[idx] + 1;
                for dir in DIRECTIONS {

                    // move until we hit a wall
                    let mut current_tile = idx as i32;
                    loop {
                        if self.board.has_wall(current_tile, dir) {
                            break;
                        }

                        // is this the most efficient path we've found to this square
                        current_tile += self.board.offset(dir);
                        if score < optimal_moves[current_tile as usize] {
                            optimal_moves[current_tile as usize] = score;
                            active_cells[current_tile as usize] = true;
                            done = false;
                        }
                    }

                }
            }
        }

        self.precomputed_target_moves = optimal_moves;
    }

    fn solve(&mut self, max_depth: u8) -> bool {

        // iterative deepening DFS 
        let mut current_max_depth = 1;
        while current_max_depth <= max_depth {
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

        if depth > max_depth {
            return false;
        }

        // are we done
        if self.robots[&self.target_robot_id].position == self.board.goal.position {
            return true;
        }

        // is the target within the upper bound of theoretical moves remaining
        let best_possible = self.precomputed_target_moves[self.robots[&self.target_robot_id].position as usize];
        if best_possible > max_depth - depth {
            return false;
        }

        // check cache of previously seen states
        let hash = self.compute_hash();
        let previous_best = match self.cache.get(&hash) {
            Some(best) => *best,
            None => 0,
        };

        if previous_best >= max_depth - depth {
            // We have already been to this state in the past with more moves remainng
            return false; 
        }
        self.cache.insert(hash, max_depth - depth);



        for id in &IDS {
            for direction in DIRECTIONS {

                let previous_position = self.robots.get(id).unwrap().position;

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

        // disallow undoing previous move because it is never optimal
        if let Some(last_move) = self.move_stack.last() {
            let is_same_robot = robot.id == last_move.robot_id;
            let is_reverse_movement = reverse(last_move.direction) == direction;

            if is_same_robot && is_reverse_movement {
                return false;
            }
        }

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
fn test_move() {
    let tiles = vec![ UP | LEFT,      UP,        UP | RIGHT,
                           LEFT,       0,             RIGHT,
                    LEFT | DOWN,    DOWN,      DOWN | RIGHT];
    let initial_robots = vec![Robot{ id: 'R', position: 0}, Robot{ id: 'B', position: 1}, Robot{ id: 'Y', position: 2}, Robot{ id: 'G', position: 3}];
    let goal = Goal{ robot_id: 'R', position: 8 };
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

    // initial locations
    assert_eq!(solver.robots.get(&'R').unwrap().position, 0);
    assert_eq!(solver.robots.get(&'B').unwrap().position, 1);
    assert_eq!(solver.robots.get(&'Y').unwrap().position, 2);
    assert_eq!(solver.robots.get(&'G').unwrap().position, 3);

    // can't move
    assert_eq!(solver.move_robot('R', UP), false);
    assert_eq!(solver.move_robot('R', DOWN), false);
    assert_eq!(solver.move_robot('R', LEFT), false);
    assert_eq!(solver.move_robot('R', RIGHT), false);
    assert_eq!(solver.robots.get(&'R').unwrap().position, 0);
    
    assert_eq!(solver.move_robot('G', DOWN), true);
    assert_eq!(solver.robots.get(&'G').unwrap().position, 6);

    assert_eq!(solver.move_robot('R', DOWN), true);
    assert_eq!(solver.robots.get(&'R').unwrap().position, 3);

    assert_eq!(solver.move_robot('R', RIGHT), true);
    assert_eq!(solver.robots.get(&'R').unwrap().position, 5);

    assert_eq!(solver.move_robot('R', DOWN), true);
    assert_eq!(solver.robots.get(&'R').unwrap().position, 8);

}


#[test]
fn test_solve() {
    let tiles = vec![ UP | LEFT,      UP,        UP | RIGHT,
                           LEFT,       0,             RIGHT,
                    LEFT | DOWN,    DOWN,      DOWN | RIGHT];
    let initial_robots = vec![Robot{ id: 'R', position: 0}, Robot{ id: 'B', position: 1}, Robot{ id: 'Y', position: 2}, Robot{ id: 'G', position: 3}];
    let goal = Goal{ robot_id: 'R', position: 8 };
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
    let solved = solver.solve(3);
    assert_eq!(solved, true);
}

#[test]
fn test_big_solve() {

    let tiles = vec![5,1,9,5,1,1,1,1,1,1,1,9,5,1,1,9,4,0,0,0,0,0,2,0,0,10,4,0,0,0,0,8,4,0,0,0,0,10,5,0,0,1,0,0,0,0,0,10,4,0,0,0,0,1,0,0,0,0,8,6,0,0,0,9,4,0,0,0,0,0,0,0,0,0,0,1,0,0,2,8,12,6,0,0,0,0,0,0,0,0,2,0,0,8,5,8,6,1,0,0,2,0,0,2,2,0,9,4,0,0,0,8,5,0,0,0,9,4,8,5,9,4,0,0,8,6,16,8,4,0,2,0,8,6,8,6,10,4,0,0,2,1,0,8,4,8,5,0,0,1,0,1,1,0,0,0,9,6,0,10,4,0,0,0,0,0,0,0,0,2,0,0,0,1,0,9,6,0,0,0,0,0,0,0,8,5,0,0,0,0,0,8,5,0,0,0,2,0,0,0,0,0,0,0,0,0,10,12,20,0,0,0,9,4,0,0,0,0,0,16,0,0,1,8,4,10,4,0,0,0,0,0,0,16,0,0,0,0,0,8,6,3,2,2,2,10,6,2,2,10,6,2,2,2,2,10];
    let initial_robots = vec![Robot{ id: 'R', position: 208},Robot{ id: 'B', position: 126},Robot{ id: 'G', position: 219},Robot{ id: 'Y', position: 233}];
    let goal = Goal { robot_id: 'R', position: 225 };

    let width = 16;
    let height = 16;

    let board = Board{
        tiles,
        initial_robots,
        width,
        height,
        goal,
    };

    let mut solver = Solver::new(board);
    let solved = solver.solve(5);
    assert_eq!(solved, true);
}

#[test]
fn test_big_solve2() {

    let tiles = vec![5,1,9,5,1,1,1,1,9,5,1,1,1,1,1,9,4,0,0,0,0,0,2,0,0,8,6,0,0,0,0,8,4,0,0,0,0,10,5,0,0,0,1,0,10,4,0,8,4,0,0,0,0,1,0,0,2,0,0,8,5,0,0,8,4,0,0,0,0,0,0,0,9,4,0,0,0,0,0,8,12,6,0,0,0,0,0,0,0,0,0,0,0,2,0,8,6,1,0,0,2,0,0,2,2,16,0,0,0,25,4,8,5,0,0,0,9,4,8,5,9,4,0,0,0,0,0,8,4,0,2,16,8,6,8,6,10,4,0,0,2,0,0,8,4,8,5,0,0,1,0,1,1,0,0,0,9,4,0,10,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,9,6,0,0,0,0,0,0,0,0,2,0,0,0,0,0,8,5,0,0,0,2,0,0,0,8,5,0,0,0,0,0,8,4,0,0,0,9,4,0,0,0,0,0,0,0,8,6,8,4,10,4,16,0,0,0,0,0,0,0,10,4,0,1,8,6,3,2,2,2,10,6,2,2,2,2,3,2,10,6,10];
    let initial_robots = vec![Robot{ id: 'B', position: 109},Robot{ id: 'G', position: 227},Robot{ id: 'Y', position: 131},Robot{ id: 'R', position: 105}];
    let goal = Goal { robot_id: 'Y', position: 133 };

    let width = 16;
    let height = 16;

    let board = Board{
        tiles,
        initial_robots,
        width,
        height,
        goal,
    };

    let mut solver = Solver::new(board);
    let solved = solver.solve(10);
    assert_eq!(solved, true);
}

#[derive(Debug, Clone)]
struct Robot {
    id: char,
    position: i32,
}

struct Goal {
    robot_id: char,
    position: i32,
}

#[derive(Debug, Clone)]
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

fn direction_name(direction: i32) -> char {
    match direction {
        UP => 'u',
        DOWN => 'd',
        LEFT => 'l',
        RIGHT => 'r',
        _ => panic!("Invalid Direction"),
    }
}



const UP: i32 = 1 << 0;
const DOWN: i32 = 1 << 1;
const LEFT: i32 = 1 << 2;
const RIGHT: i32 = 1 << 3;
const ROBOT: i32 = 1 << 4;

const DIRECTIONS: [i32; 4] = [UP, DOWN, LEFT, RIGHT];
const IDS: [char; 4] = ['R', 'G', 'B', 'Y'];

fn main() {

    /*
    let tiles = vec![ UP | LEFT,      UP,        UP | RIGHT,
                           LEFT,       0,             RIGHT,
                    LEFT | DOWN,    DOWN,      DOWN | RIGHT];
    let initial_robots = vec![Robot{ id: 'R', position: 0}, Robot{ id: 'B', position: 1}, Robot{ id: 'Y', position: 2}, Robot{ id: 'G', position: 3}];
    let goal = Goal{ robot_id: 'R', position: 8 };
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
    let solved = solver.solve(3);
    */

    let tiles = vec![5,1,9,5,1,1,1,1,9,5,1,1,1,1,1,9,4,0,0,0,0,0,2,0,0,8,6,0,0,0,0,8,4,0,0,0,0,10,5,0,0,0,1,0,10,4,0,8,4,0,0,0,0,1,0,0,2,0,0,8,5,0,0,8,4,0,0,0,0,0,0,0,9,4,0,0,0,0,0,8,12,6,0,0,0,0,0,0,0,0,0,0,0,2,0,8,6,1,0,0,2,0,0,2,2,16,0,0,0,25,4,8,5,0,0,0,9,4,8,5,9,4,0,0,0,0,0,8,4,0,2,16,8,6,8,6,10,4,0,0,2,0,0,8,4,8,5,0,0,1,0,1,1,0,0,0,9,4,0,10,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,9,6,0,0,0,0,0,0,0,0,2,0,0,0,0,0,8,5,0,0,0,2,0,0,0,8,5,0,0,0,0,0,8,4,0,0,0,9,4,0,0,0,0,0,0,0,8,6,8,4,10,4,16,0,0,0,0,0,0,0,10,4,0,1,8,6,3,2,2,2,10,6,2,2,2,2,3,2,10,6,10];
    let initial_robots = vec![Robot{ id: 'B', position: 109},Robot{ id: 'G', position: 227},Robot{ id: 'Y', position: 131},Robot{ id: 'R', position: 105}];
    let goal = Goal { robot_id: 'Y', position: 133 };

    let width = 16;
    let height = 16;

    let board = Board{
        tiles,
        initial_robots,
        width,
        height,
        goal,
    };

    let mut solver = Solver::new(board);
    let solved = solver.solve(10);

    if solved {
        for m in solver.move_stack {
            print!("{}{}-", m.robot_id, direction_name(m.direction));
        }
    }
}
