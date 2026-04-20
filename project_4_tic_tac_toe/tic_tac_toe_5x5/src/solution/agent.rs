use tic_tac_toe_stencil::agents::Agent;
use tic_tac_toe_stencil::board::{Board, Cell};
use tic_tac_toe_stencil::player::Player;

pub struct SolutionAgent {}

impl SolutionAgent {
    const MAX_DEPTH: i32 = 4;

    fn opponent(player: Player) -> Player {
        match player {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }

    fn playable_cell_count(board: &Board) -> usize {
        let mut count = 0;

        for row in board.get_cells() {
            for cell in row {
                match cell {
                    Cell::X | Cell::O | Cell::Empty => count += 1,
                    Cell::Wall => {}
                }
            }
        }

        count
    }

    fn heuristic(board: &Board, root_player: Player) -> i32 {
        let cells = board.get_cells();
        let n = cells.len();

        let mut total = board.score() * 100;

        for i in 0..n {
            for j in 0..n {
                let dist_i = (i as i32 - 2).abs();
                let dist_j = (j as i32 - 2).abs();
                let center_bonus = 4 - (dist_i + dist_j);

                match cells[i][j] {
                    Cell::X => {
                        if root_player == Player::X {
                            total += center_bonus;
                        } else {
                            total -= center_bonus;
                        }
                    }
                    Cell::O => {
                        if root_player == Player::O {
                            total += center_bonus;
                        } else {
                            total -= center_bonus;
                        }
                    }
                    Cell::Empty | Cell::Wall => {}
                }
            }
        }

        for i in 0..n {
            for j in 0..n {
                if j + 2 < n {
                    total += Self::score_window(
                        [&cells[i][j], &cells[i][j + 1], &cells[i][j + 2]],
                        root_player,
                    );
                }

                if i + 2 < n {
                    total += Self::score_window(
                        [&cells[i][j], &cells[i + 1][j], &cells[i + 2][j]],
                        root_player,
                    );
                }

                if i + 2 < n && j + 2 < n {
                    total += Self::score_window(
                        [&cells[i][j], &cells[i + 1][j + 1], &cells[i + 2][j + 2]],
                        root_player,
                    );
                }

                if i + 2 < n && j >= 2 {
                    total += Self::score_window(
                        [&cells[i][j], &cells[i + 1][j - 1], &cells[i + 2][j - 2]],
                        root_player,
                    );
                }
            }
        }

        total
    }

    fn score_window(window: [&Cell; 3], root_player: Player) -> i32 {
        let mut root_count = 0;
        let mut opp_count = 0;
        let mut empty_count = 0;
        let mut wall_count = 0;

        for cell in window {
            match cell {
                Cell::X => {
                    if root_player == Player::X {
                        root_count += 1;
                    } else {
                        opp_count += 1;
                    }
                }
                Cell::O => {
                    if root_player == Player::O {
                        root_count += 1;
                    } else {
                        opp_count += 1;
                    }
                }
                Cell::Empty => empty_count += 1,
                Cell::Wall => wall_count += 1,
            }
        }

        if wall_count > 0 || (root_count > 0 && opp_count > 0) {
            return 0;
        }

        if root_count == 3 {
            return 120;
        }
        if root_count == 2 && empty_count == 1 {
            return 25;
        }
        if root_count == 1 && empty_count == 2 {
            return 4;
        }

        if opp_count == 3 {
            return -120;
        }
        if opp_count == 2 && empty_count == 1 {
            return -30;
        }
        if opp_count == 1 && empty_count == 2 {
            return -4;
        }

        0
    }

    fn evaluate_terminal(board: &Board, root_player: Player, depth: i32) -> Option<i32> {
        if board.game_over() {
            let raw_score = board.score();

            let adjusted_score = match root_player {
                Player::X => raw_score,
                Player::O => -raw_score,
            };

            if adjusted_score > 0 {
                return Some(100000 - depth);
            } else if adjusted_score < 0 {
                return Some(depth - 100000);
            } else {
                return Some(0);
            }
        }

        None
    }

    fn minimax(
        board: &mut Board,
        current_player: Player,
        root_player: Player,
        depth: i32,
        max_depth: i32,
    ) -> (i32, usize, usize) {
        if let Some(score) = Self::evaluate_terminal(board, root_player, depth) {
            return (score, 0, 0);
        }

        if depth >= max_depth {
            return (Self::heuristic(board, root_player), 0, 0);
        }

        let moves = board.moves();

        if moves.is_empty() {
            return (Self::heuristic(board, root_player), 0, 0);
        }

        if current_player == root_player {
            let mut best_score = i32::MIN;
            let mut best_move = moves[0];

            for m in moves {
                board.apply_move(m, current_player);

                let (score, _, _) = Self::minimax(
                    board,
                    Self::opponent(current_player),
                    root_player,
                    depth + 1,
                    max_depth,
                );

                board.undo_move(m, current_player);

                if score > best_score {
                    best_score = score;
                    best_move = m;
                }
            }

            (best_score, best_move.0, best_move.1)
        } else {
            let mut best_score = i32::MAX;
            let mut best_move = moves[0];

            for m in moves {
                board.apply_move(m, current_player);

                let (score, _, _) = Self::minimax(
                    board,
                    Self::opponent(current_player),
                    root_player,
                    depth + 1,
                    max_depth,
                );

                board.undo_move(m, current_player);

                if score < best_score {
                    best_score = score;
                    best_move = m;
                }
            }

            (best_score, best_move.0, best_move.1)
        }
    }
}

impl Agent for SolutionAgent {
    fn solve(board: &mut Board, player: Player, _time_limit: u64) -> (i32, usize, usize) {
        let playable = Self::playable_cell_count(board);

        let max_depth = if playable == 9 {
            9
        } else {
            Self::MAX_DEPTH
        };

        Self::minimax(board, player, player, 0, max_depth)
    }
}