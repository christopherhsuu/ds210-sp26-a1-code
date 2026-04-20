use tic_tac_toe_stencil::agents::Agent;
use tic_tac_toe_stencil::board::Board;
use tic_tac_toe_stencil::player::Player;

pub struct SolutionAgent {}

impl SolutionAgent {
    fn opponent(player: Player) -> Player {
        match player {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }

    fn evaluate_terminal(board: &Board, root_player: Player, depth: i32) -> Option<i32> {
        if board.game_over() {
            let raw_score = board.score();

            let adjusted_score = match root_player {
                Player::X => raw_score,
                Player::O => -raw_score,
            };

            if adjusted_score > 0 {
                return Some(100 - depth);
            } else if adjusted_score < 0 {
                return Some(depth - 100);
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
    ) -> (i32, usize, usize) {
        if let Some(score) = Self::evaluate_terminal(board, root_player, depth) {
            return (score, 0, 0);
        }

        let moves = board.moves();

        if moves.is_empty() {
            return (0, 0, 0);
        }

        if current_player == root_player {
            let mut best_score = i32::MIN;
            let mut best_move = moves[0];

            for m in moves {
                board.apply_move(m, current_player);

                let (score, _, _) =
                    Self::minimax(board, Self::opponent(current_player), root_player, depth + 1);

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

                let (score, _, _) =
                    Self::minimax(board, Self::opponent(current_player), root_player, depth + 1);

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
        Self::minimax(board, player, player, 0)
    }
}
