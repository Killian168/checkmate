use chess::{Board, ChessMove, MoveGen, Piece, Square};
use std::str::FromStr;

use crate::{
    models::{
        game_session::{GameSession, GameStatus, Turn},
        move_request::MoveRequest,
    },
    services::errors::chess_service_errors::ChessServiceError,
};

#[derive(Clone)]
pub struct ChessService;

impl ChessService {
    pub fn new() -> Self {
        ChessService
    }

    /// Validate and make a move on the game session.
    /// Updates the FEN, move history, turn, and game status.
    pub fn validate_and_make_move(
        &self,
        game_session: &mut GameSession,
        move_request: &MoveRequest,
        player_id: &str,
    ) -> Result<(), ChessServiceError> {
        // Check if it's the player's turn
        let current_turn = &game_session.whose_turn;
        let is_player_white = game_session.player_white_id == player_id;

        match (current_turn, is_player_white) {
            (Turn::White, true) | (Turn::Black, false) => {}
            _ => {
                return Err(ChessServiceError::ValidationError(
                    "Not your turn".to_string(),
                ));
            }
        }

        // Parse the board from FEN
        let board = Board::from_str(&game_session.fen_board)
            .map_err(|e| ChessServiceError::InvalidPosition(format!("Invalid FEN: {}", e)))?;

        // Check if game is already over
        let status = board.status();
        if status != chess::BoardStatus::Ongoing {
            return Err(ChessServiceError::GameOver(
                "Game is already over".to_string(),
            ));
        }

        // Parse the move
        let from_sq = Square::from_str(&move_request.from_square)
            .map_err(|_| ChessServiceError::ValidationError("Invalid from square".to_string()))?;
        let to_sq = Square::from_str(&move_request.to_square)
            .map_err(|_| ChessServiceError::ValidationError("Invalid to square".to_string()))?;

        // Determine promotion piece
        let promotion = match &move_request.promotion_piece {
            Some(p) => match p.as_str() {
                "q" => Some(Piece::Queen),
                "r" => Some(Piece::Rook),
                "b" => Some(Piece::Bishop),
                "n" => Some(Piece::Knight),
                _ => {
                    return Err(ChessServiceError::ValidationError(
                        "Invalid promotion piece".to_string(),
                    ))
                }
            },
            None => None,
        };

        let chess_move = ChessMove::new(from_sq, to_sq, promotion);

        // Validate the move by checking if it's in legal moves
        let legal_moves: Vec<ChessMove> = MoveGen::new_legal(&board).collect();
        if !legal_moves.contains(&chess_move) {
            return Err(ChessServiceError::IllegalMove(
                "Move is not legal".to_string(),
            ));
        }

        // Make the move
        let mut new_board = board.clone();
        board.make_move(chess_move, &mut new_board);

        // Update the game session
        game_session.fen_board = format!("{}", new_board);
        game_session.move_history.push(format!(
            "{} to {}",
            move_request.from_square, move_request.to_square
        ));

        // Switch turn
        game_session.whose_turn = match game_session.whose_turn {
            Turn::White => Turn::Black,
            Turn::Black => Turn::White,
        };

        // Check new game status
        let new_status = new_board.status();
        game_session.status = match new_status {
            chess::BoardStatus::Ongoing => GameStatus::Ongoing,
            chess::BoardStatus::Stalemate => GameStatus::Stalemate,
            chess::BoardStatus::Checkmate => {
                // Determine winner: the player who just moved caused checkmate, so they won
                if game_session.whose_turn == Turn::White {
                    // Black just moved, white is in checkmate
                    game_session.winner = Some(game_session.player_black_id.clone());
                } else {
                    // White just moved, black is in checkmate
                    game_session.winner = Some(game_session.player_white_id.clone());
                }
                GameStatus::Checkmate
            }
        };

        Ok(())
    }

    /// Get the current game status for a session
    pub fn get_game_status(game_session: &GameSession) -> Result<GameStatus, ChessServiceError> {
        let board = Board::from_str(&game_session.fen_board)
            .map_err(|e| ChessServiceError::InvalidPosition(format!("Invalid FEN: {}", e)))?;

        let status = board.status();
        Ok(match status {
            chess::BoardStatus::Ongoing => GameStatus::Ongoing,
            chess::BoardStatus::Stalemate => GameStatus::Stalemate,
            chess::BoardStatus::Checkmate => GameStatus::Checkmate,
        })
    }

    /// Get legal moves for the current position (for UI hints)
    pub fn get_legal_moves(game_session: &GameSession) -> Result<Vec<String>, ChessServiceError> {
        let board = Board::from_str(&game_session.fen_board)
            .map_err(|e| ChessServiceError::InvalidPosition(format!("Invalid FEN: {}", e)))?;

        let legal_moves: Vec<String> = MoveGen::new_legal(&board)
            .map(|m| format!("{}{}", m.get_source(), m.get_dest()))
            .collect();

        Ok(legal_moves)
    }
}
