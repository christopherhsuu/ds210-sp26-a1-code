use tic_tac_toe_stencil::agents::Agent;
use tic_tac_toe_stencil::board::{Board, Cell};
use tic_tac_toe_stencil::player::Player;

pub struct SolutionAgent {}

impl SolutionAgent {
    const MAX_DEPTH: i32 = 6;

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

        // Completed 3-in-a-row = actual points
        if root_count == 3 {
            return 100_000;
        }
        if opp_count == 3 {
            return -100_000;
        }

        // Strong offensive/defensive threats
        if root_count == 2 && empty_count == 1 {
            return 4_000;
        }
        if opp_count == 2 && empty_count == 1 {
            return -4_500;
        }

        // Early setup potential
        if root_count == 1 && empty_count == 2 {
            return 80;
        }
        if opp_count == 1 && empty_count == 2 {
            return -90;
        }

        0
    }

    fn heuristic(board: &Board, root_player: Player) -> i32 {
        let cells = board.get_cells();
        let n = cells.len();

        let mut total = board.score() * 1000;

        let center = n as i32 / 2;

        // Center control matters because center pieces can be part of many 3-in-a-rows.
        for i in 0..n {
            for j in 0..n {
                let dist_i = (i as i32 - center).abs();
                let dist_j = (j as i32 - center).abs();
                let center_bonus = 20 - (dist_i + dist_j);

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

        // Score every possible 3-in-a-row window.
        for i in 0..n {
            for j in 0..n {
                // Horizontal
                if j + 2 < n {
                    total += Self::score_window(
                        [&cells[i][j], &cells[i][j + 1], &cells[i][j + 2]],
                        root_player,
                    );
                }

                // Vertical
                if i + 2 < n {
                    total += Self::score_window(
                        [&cells[i][j], &cells[i + 1][j], &cells[i + 2][j]],
                        root_player,
                    );
                }

                // Diagonal down-right
                if i + 2 < n && j + 2 < n {
                    total += Self::score_window(
                        [&cells[i][j], &cells[i + 1][j + 1], &cells[i + 2][j + 2]],
                        root_player,
                    );
                }

                // Diagonal down-left
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

    fn minimax(
        board: &mut Board,
        current_player: Player,
        root_player: Player,
        depth: i32,
        max_depth: i32,
        mut alpha: i32,
        mut beta: i32,
    ) -> (i32, usize, usize) {
        if board.game_over() {
            return (Self::heuristic(board, root_player), 0, 0);
        }

        if depth >= max_depth {
            return (Self::heuristic(board, root_player), 0, 0);
        }

        let mut moves = board.moves();

        if moves.is_empty() {
            return (Self::heuristic(board, root_player), 0, 0);
        }

        // Move ordering: try moves that immediately improve heuristic first.
        moves.sort_by(|a, b| {
            board.apply_move(*a, current_player);
            let score_a = Self::heuristic(board, root_player);
            board.undo_move(*a, current_player);

            board.apply_move(*b, current_player);
            let score_b = Self::heuristic(board, root_player);
            board.undo_move(*b, current_player);

            if current_player == root_player {
                score_b.cmp(&score_a)
            } else {
                score_a.cmp(&score_b)
            }
        });

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
                    alpha,
                    beta,
                );

                board.undo_move(m, current_player);

                if score > best_score {
                    best_score = score;
                    best_move = m;
                }

                alpha = alpha.max(best_score);

                if beta <= alpha {
                    break;
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
                    alpha,
                    beta,
                );

                board.undo_move(m, current_player);

                if score < best_score {
                    best_score = score;
                    best_move = m;
                }

                beta = beta.min(best_score);

                if beta <= alpha {
                    break;
                }
            }

            (best_score, best_move.0, best_move.1)
        }
    }
}

impl Agent for SolutionAgent {
    fn solve(board: &mut Board, player: Player, _time_limit: u64) -> (i32, usize, usize) {
        let playable = Self::playable_cell_count(board);

        let max_depth = if playable <= 9 {
            9
        } else if playable <= 16 {
            7
        } else {
            Self::MAX_DEPTH
        };

        Self::minimax(
            board,
            player,
            player,
            0,
            max_depth,
            i32::MIN,
            i32::MAX,
        )
    }
}