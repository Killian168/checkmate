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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::game_session::{GameSession, GameStatus, Turn};
    use crate::models::move_request::MoveRequest;

    #[test]
    fn test_validate_and_make_move_valid_move() {
        let mut game_session = GameSession::new("white_player", "black_player");
        let chess_service = ChessService::new();
        let move_request = MoveRequest::new(
            game_session.session_id.clone(),
            "e2".to_string(),
            "e4".to_string(),
        );

        let result =
            chess_service.validate_and_make_move(&mut game_session, &move_request, "white_player");

        assert!(result.is_ok());
        assert_eq!(game_session.whose_turn, Turn::Black);
        assert_eq!(game_session.status, GameStatus::Ongoing);
        assert!(game_session.move_history.contains(&"e2 to e4".to_string()));
        // FEN should have changed
        assert_ne!(
            game_session.fen_board,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    fn test_validate_and_make_move_invalid_move() {
        let mut game_session = GameSession::new("white_player", "black_player");
        let chess_service = ChessService::new();
        let move_request = MoveRequest::new(
            game_session.session_id.clone(),
            "e2".to_string(),
            "e5".to_string(), // Invalid move
        );

        let result =
            chess_service.validate_and_make_move(&mut game_session, &move_request, "white_player");

        assert!(result.is_err());
        match result.unwrap_err() {
            ChessServiceError::IllegalMove(_) => {}
            _ => panic!("Expected IllegalMove error"),
        }
    }

    #[test]
    fn test_validate_and_make_move_wrong_turn() {
        let mut game_session = GameSession::new("white_player", "black_player");
        let chess_service = ChessService::new();
        let move_request = MoveRequest::new(
            game_session.session_id.clone(),
            "e7".to_string(),
            "e5".to_string(),
        );

        let result = chess_service.validate_and_make_move(
            &mut game_session,
            &move_request,
            "black_player", // Trying to move as black on white's turn
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            ChessServiceError::ValidationError(msg) => assert_eq!(msg, "Not your turn"),
            _ => panic!("Expected ValidationError with 'Not your turn'"),
        }
    }

    #[test]
    fn test_get_game_status_ongoing() {
        let game_session = GameSession::new("white_player", "black_player");

        let status = ChessService::get_game_status(&game_session).unwrap();

        assert_eq!(status, GameStatus::Ongoing);
    }

    #[test]
    fn test_get_legal_moves_starting_position() {
        let game_session = GameSession::new("white_player", "black_player");

        let legal_moves = ChessService::get_legal_moves(&game_session).unwrap();

        // Starting position should have 20 legal moves for white
        assert_eq!(legal_moves.len(), 20);
        // Check some expected moves
        assert!(legal_moves.contains(&"e2e3".to_string()));
        assert!(legal_moves.contains(&"d2d4".to_string()));
        assert!(legal_moves.contains(&"b1c3".to_string()));
    }

    #[test]
    fn test_validate_and_make_move_with_promotion() {
        // Set up a position where a pawn can promote
        let mut game_session = GameSession::new("white_player", "black_player");
        // Position: white pawn on a7, ready to promote to a8
        game_session.fen_board = "8/P7/8/8/8/8/8/K6k w - - 0 1".to_string();
        let chess_service = ChessService::new();
        let move_request = MoveRequest::with_promotion(
            game_session.session_id.clone(),
            "a7".to_string(),
            "a8".to_string(),
            "q".to_string(),
        );

        let result =
            chess_service.validate_and_make_move(&mut game_session, &move_request, "white_player");

        assert!(result.is_ok());
        // After promotion, the FEN should contain the queen
        assert!(game_session.fen_board.contains('Q'));
    }
}
