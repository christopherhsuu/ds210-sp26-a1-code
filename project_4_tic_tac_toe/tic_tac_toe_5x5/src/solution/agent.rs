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

        // Aggressive scoring: make winning easier
        if root_count == 3 {
            return 10_000;
        }
        if root_count == 2 && empty_count == 1 {
            return 900;
        }
        if root_count == 1 && empty_count == 2 {
            return 30;
        }

        // Still block opponent threats
        if opp_count == 3 {
            return -9_000;
        }
        if opp_count == 2 && empty_count == 1 {
            return -700;
        }
        if opp_count == 1 && empty_count == 2 {
            return -10;
        }

        0
    }

    fn heuristic(board: &Board, root_player: Player) -> i32 {
        let cells = board.get_cells();
        let n = cells.len();

        let mut total = board.score() * 100;

        let center = n as i32 / 2;

        // Prefer center squares
        for i in 0..n {
            for j in 0..n {
                let dist_i = (i as i32 - center).abs();
                let dist_j = (j as i32 - center).abs();
                let center_bonus = 10 - (dist_i + dist_j);

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

        // Check rows, columns, and diagonals in windows of 3
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

    fn evaluate_terminal(board: &Board, root_player: Player, depth: i32) -> Option<i32> {
        if board.game_over() {
            let raw_score = board.score();

            let adjusted_score = match root_player {
                Player::X => raw_score,
                Player::O => -raw_score,
            };

            if adjusted_score > 0 {
                return Some(100_000 - depth);
            } else if adjusted_score < 0 {
                return Some(depth - 100_000);
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
        mut alpha: i32,
        mut beta: i32,
    ) -> (i32, usize, usize) {
        if let Some(score) = Self::evaluate_terminal(board, root_player, depth) {
            return (score, 0, 0);
        }

        if depth >= max_depth {
            return (Self::heuristic(board, root_player), 0, 0);
        }

        let mut moves = board.moves();

        if moves.is_empty() {
            return (Self::heuristic(board, root_player), 0, 0);
        }

        // Try center moves first
        let cells = board.get_cells();
        let n = cells.len() as i32;
        let center = n / 2;

        moves.sort_by_key(|&(x, y)| {
            let dx = x as i32 - center;
            let dy = y as i32 - center;
            dx.abs() + dy.abs()
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
        let moves = board.moves();

        // 1. If we can win immediately, take it.
        for m in &moves {
            board.apply_move(*m, player);

            if board.game_over() {
                let result = (100_000, m.0, m.1);
                board.undo_move(*m, player);
                return result;
            }

            board.undo_move(*m, player);
        }

        // 2. If opponent can win immediately, block it.
        let opponent = Self::opponent(player);

        for m in &moves {
            board.apply_move(*m, opponent);

            if board.game_over() {
                let result = (90_000, m.0, m.1);
                board.undo_move(*m, opponent);
                return result;
            }

            board.undo_move(*m, opponent);
        }

        // 3. Prefer strong attacking moves before minimax.
        let mut best_attack_score = i32::MIN;
        let mut best_attack_move = None;

        for m in &moves {
            board.apply_move(*m, player);

            let attack_score = Self::heuristic(board, player);

            board.undo_move(*m, player);

            if attack_score > best_attack_score {
                best_attack_score = attack_score;
                best_attack_move = Some(*m);
            }
        }

        if let Some(m) = best_attack_move {
            if best_attack_score > 1200 {
                return (best_attack_score, m.0, m.1);
            }
        }

        // 4. Otherwise use minimax.
        let playable = Self::playable_cell_count(board);

        let max_depth = if playable == 9 {
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