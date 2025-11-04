use eframe::{egui, App, Frame, NativeOptions, egui::ViewportBuilder};
use egui::Vec2;

/// Enum representing all possible chess pieces and empty squares
/// Each piece has a color variant (White/Black)
#[derive(Copy, Clone, PartialEq, Debug)]
enum Piece {
    Empty,
    PawnWhite,
    PawnBlack,
    RookWhite,
    RookBlack,
    KnightWhite,
    KnightBlack,
    BishopWhite,
    BishopBlack,
    QueenWhite,
    QueenBlack,
    KingWhite,
    KingBlack,
}

impl Piece {
    /// Returns true if the piece is white
    fn is_white(&self) -> bool {
        match self {
            Piece::PawnWhite | Piece::RookWhite | Piece::KnightWhite | 
            Piece::BishopWhite | Piece::QueenWhite | Piece::KingWhite => true,
            _ => false,
        }
    }

    /// Returns true if the piece is black
    fn is_black(&self) -> bool {
        match self {
            Piece::PawnBlack | Piece::RookBlack | Piece::KnightBlack | 
            Piece::BishopBlack | Piece::QueenBlack | Piece::KingBlack => true,
            _ => false,
        }
    }

    /// Returns true if the square is empty
    fn is_empty(&self) -> bool {
        *self == Piece::Empty
    }

    /// Returns true if both pieces are the same color
    fn is_same_color(&self, other: &Piece) -> bool {
        (self.is_white() && other.is_white()) || (self.is_black() && other.is_black())
    }
}

/// Struct to track game state for special moves (castling, en passant)
/// This is necessary to enforce chess rules properly
#[derive(Clone)]
struct GameState {
    // Castling rights - track if kings and rooks have moved
    white_king_moved: bool,
    black_king_moved: bool,
    white_rook_queenside_moved: bool,  // Queen's side rook (a1/a8)
    white_rook_kingside_moved: bool,   // King's side rook (h1/h8)
    black_rook_queenside_moved: bool,
    black_rook_kingside_moved: bool,
    
    // En passant target square - where an en passant capture is possible
    en_passant_target: Option<(usize, usize)>,
}

impl Default for GameState {
    fn default() -> Self {
        // Initial game state - no pieces have moved yet
        GameState {
            white_king_moved: false,
            black_king_moved: false,
            white_rook_queenside_moved: false,
            white_rook_kingside_moved: false,
            black_rook_queenside_moved: false,
            black_rook_kingside_moved: false,
            en_passant_target: None,
        }
    }
}

/// Main board structure containing the game state
struct Board {
    squares: [[Piece; 8]; 8],  // 8x8 chess board
    white_to_move: bool,       // Whose turn it is
    game_state: GameState,     // Special move tracking
}

impl Board {
    /// Creates a new chess board with standard starting position
    fn new() -> Self {
        use Piece::*;
        // Set up the standard chess starting position
        // Row 0 = rank 8 (black back rank), Row 7 = rank 1 (white back rank)
        let squares = [
            [RookBlack, KnightBlack, BishopBlack, QueenBlack, KingBlack, BishopBlack, KnightBlack, RookBlack],
            [PawnBlack; 8],     // Black pawns on rank 7
            [Empty; 8],         // Empty ranks
            [Empty; 8],
            [Empty; 8],
            [Empty; 8],
            [PawnWhite; 8],     // White pawns on rank 2
            [RookWhite, KnightWhite, BishopWhite, QueenWhite, KingWhite, BishopWhite, KnightWhite, RookWhite],
        ];
        Board { 
            squares, 
            white_to_move: true,  // White moves first
            game_state: GameState::default(),
        }
    }

    /// Returns the Unicode symbol for each piece type
    /// Uses Unicode chess symbols for visual representation
    fn piece_symbol(p: Piece) -> &'static str {
        match p {
            Piece::Empty => "·",
            Piece::PawnWhite => "♙",
            Piece::PawnBlack => "♟",
            Piece::RookWhite => "♖",
            Piece::RookBlack => "♜",
            Piece::KnightWhite => "♘",
            Piece::KnightBlack => "♞",
            Piece::BishopWhite => "♗",
            Piece::BishopBlack => "♝",
            Piece::QueenWhite => "♕",
            Piece::QueenBlack => "♛",
            Piece::KingWhite => "♔",
            Piece::KingBlack => "♚",
        }
    }

    /// Main move validation function - checks if a move is legal
    /// Combines piece movement rules with chess-specific constraints
    fn is_valid_move(&self, from_row: usize, from_col: usize, to_row: usize, to_col: usize) -> bool {
        let piece = self.squares[from_row][from_col];
        let target = self.squares[to_row][to_col];

        // Basic validation checks
        if piece.is_empty() {
            return false;  // Can't move an empty square
        }

        // Turn validation - can only move your own pieces
        if (self.white_to_move && piece.is_black()) || (!self.white_to_move && piece.is_white()) {
            return false;
        }

        // Can't capture your own pieces
        if piece.is_same_color(&target) {
            return false;
        }

        // Check if the move is valid for this piece type
        if !self.is_piece_move_valid(piece, from_row, from_col, to_row, to_col) {
            return false;
        }

        // Final check: ensure the move doesn't leave your king in check
        // This is crucial for chess legality
        if self.would_be_in_check_after_move(from_row, from_col, to_row, to_col) {
            return false;
        }

        true
    }

    /// Delegates piece movement validation to specific piece handlers
    fn is_piece_move_valid(&self, piece: Piece, from_row: usize, from_col: usize, to_row: usize, to_col: usize) -> bool {
        match piece {
            Piece::PawnWhite | Piece::PawnBlack => self.is_pawn_move_valid(piece, from_row, from_col, to_row, to_col),
            Piece::RookWhite | Piece::RookBlack => self.is_rook_move_valid(from_row, from_col, to_row, to_col),
            Piece::KnightWhite | Piece::KnightBlack => self.is_knight_move_valid(from_row, from_col, to_row, to_col),
            Piece::BishopWhite | Piece::BishopBlack => self.is_bishop_move_valid(from_row, from_col, to_row, to_col),
            Piece::QueenWhite | Piece::QueenBlack => self.is_queen_move_valid(from_row, from_col, to_row, to_col),
            Piece::KingWhite | Piece::KingBlack => self.is_king_move_valid(from_row, from_col, to_row, to_col),
            _ => false,
        }
    }

    /// Validates pawn movement - most complex piece due to special rules
    fn is_pawn_move_valid(&self, piece: Piece, from_row: usize, from_col: usize, to_row: usize, to_col: usize) -> bool {
        // Pawns move differently based on color
        let direction = if piece.is_white() { -1i32 } else { 1i32 };  // White moves up (decreasing row), black down
        let start_row = if piece.is_white() { 6 } else { 1 };         // Starting positions
        
        let row_diff = to_row as i32 - from_row as i32;
        let col_diff = (to_col as i32 - from_col as i32).abs();

        // Forward movement (no capture)
        if col_diff == 0 {
            // One square forward
            if row_diff == direction && self.squares[to_row][to_col].is_empty() {
                return true;
            }
            // Two squares forward from starting position
            if from_row == start_row && row_diff == 2 * direction && self.squares[to_row][to_col].is_empty() {
                return true;
            }
        }
        // Diagonal capture
        else if col_diff == 1 && row_diff == direction {
            // Normal capture
            if !self.squares[to_row][to_col].is_empty() {
                return true;
            }
            // En passant capture - special pawn capture rule
            if let Some((ep_row, ep_col)) = self.game_state.en_passant_target {
                if to_row == ep_row && to_col == ep_col {
                    return true;
                }
            }
        }

        false
    }

    /// Validates rook movement - horizontal and vertical lines
    fn is_rook_move_valid(&self, from_row: usize, from_col: usize, to_row: usize, to_col: usize) -> bool {
        // Rooks move only horizontally or vertically
        if from_row != to_row && from_col != to_col {
            return false;
        }

        // Check that the path is clear (no pieces blocking)
        self.is_path_clear(from_row, from_col, to_row, to_col)
    }

    /// Validates knight movement - L-shaped moves
    fn is_knight_move_valid(&self, from_row: usize, from_col: usize, to_row: usize, to_col: usize) -> bool {
        let row_diff = (to_row as i32 - from_row as i32).abs();
        let col_diff = (to_col as i32 - from_col as i32).abs();
        
        // Knight moves in L-shape: 2+1 or 1+2
        (row_diff == 2 && col_diff == 1) || (row_diff == 1 && col_diff == 2)
    }

    /// Validates bishop movement - diagonal lines
    fn is_bishop_move_valid(&self, from_row: usize, from_col: usize, to_row: usize, to_col: usize) -> bool {
        let row_diff = (to_row as i32 - from_row as i32).abs();
        let col_diff = (to_col as i32 - from_col as i32).abs();
        
        // Bishops move diagonally - row and column differences must be equal
        if row_diff != col_diff {
            return false;
        }

        // Check that the diagonal path is clear
        self.is_path_clear(from_row, from_col, to_row, to_col)
    }

    /// Validates queen movement - combines rook and bishop moves
    fn is_queen_move_valid(&self, from_row: usize, from_col: usize, to_row: usize, to_col: usize) -> bool {
        // Queen can move like a rook OR like a bishop
        self.is_rook_move_valid(from_row, from_col, to_row, to_col) || 
        self.is_bishop_move_valid(from_row, from_col, to_row, to_col)
    }

    /// Validates king movement - one square in any direction + castling
    fn is_king_move_valid(&self, from_row: usize, from_col: usize, to_row: usize, to_col: usize) -> bool {
        let row_diff = (to_row as i32 - from_row as i32).abs();
        let col_diff = (to_col as i32 - from_col as i32).abs();
        
        // Normal king move - one square in any direction
        if row_diff <= 1 && col_diff <= 1 {
            return true;
        }

        // Castling - king moves two squares horizontally
        if row_diff == 0 && col_diff == 2 {
            return self.can_castle(from_row, from_col, to_row, to_col);
        }

        false
    }

    /// Validates castling - complex special move with many conditions
    fn can_castle(&self, from_row: usize, from_col: usize, _to_row: usize, to_col: usize) -> bool {
        let piece = self.squares[from_row][from_col];
        
        // Must be a king
        if !matches!(piece, Piece::KingWhite | Piece::KingBlack) {
            return false;
        }

        let is_white = piece.is_white();
        let expected_row = if is_white { 7 } else { 0 };
        
        // King must be in starting position (e1/e8)
        if from_row != expected_row || from_col != 4 {
            return false;
        }

        // King must not have moved before
        if (is_white && self.game_state.white_king_moved) || (!is_white && self.game_state.black_king_moved) {
            return false;
        }

        // Determine castling side
        let is_kingside = to_col == 6;  // King moves to g-file
        let rook_col = if is_kingside { 7 } else { 0 };  // Rook starting position
        
        // Check if the corresponding rook has moved
        let rook_moved = if is_white {
            if is_kingside { self.game_state.white_rook_kingside_moved } 
            else { self.game_state.white_rook_queenside_moved }
        } else {
            if is_kingside { self.game_state.black_rook_kingside_moved } 
            else { self.game_state.black_rook_queenside_moved }
        };

        if rook_moved {
            return false;
        }

        // Verify the rook is still in its starting position
        let expected_rook = if is_white { Piece::RookWhite } else { Piece::RookBlack };
        if self.squares[expected_row][rook_col] != expected_rook {
            return false;
        }

        // Check that squares between king and rook are empty
        let start_col = if is_kingside { 5 } else { 1 };
        let end_col = if is_kingside { 6 } else { 3 };
        
        for col in start_col..=end_col {
            if !self.squares[expected_row][col].is_empty() {
                return false;
            }
        }

        // King cannot castle through check - verify king's path is safe
        // This includes the king's current square, transit square, and destination
        for col in 4..=to_col.max(4).min(6) {
            if self.is_square_under_attack(expected_row, col, !is_white) {
                return false;
            }
        }
        for col in 4.min(to_col)..=4 {
            if self.is_square_under_attack(expected_row, col, !is_white) {
                return false;
            }
        }

        true
    }

    /// Checks if the path between two squares is clear of pieces
    /// Used for rook, bishop, and queen movement validation
    fn is_path_clear(&self, from_row: usize, from_col: usize, to_row: usize, to_col: usize) -> bool {
        // Calculate direction of movement
        let row_dir = if to_row > from_row { 1i32 } else if to_row < from_row { -1i32 } else { 0i32 };
        let col_dir = if to_col > from_col { 1i32 } else if to_col < from_col { -1i32 } else { 0i32 };
        
        // Start from the square after the starting position
        let mut current_row = from_row as i32 + row_dir;
        let mut current_col = from_col as i32 + col_dir;
        
        // Check each square in the path (excluding destination)
        while current_row != to_row as i32 || current_col != to_col as i32 {
            if !self.squares[current_row as usize][current_col as usize].is_empty() {
                return false;  // Path is blocked
            }
            current_row += row_dir;
            current_col += col_dir;
        }
        
        true  // Path is clear
    }

    /// Simulates a move to check if it would leave the king in check
    /// This is essential for move legality in chess
    fn would_be_in_check_after_move(&self, from_row: usize, from_col: usize, to_row: usize, to_col: usize) -> bool {
        // Create a temporary board with the move applied
        let mut temp_board = self.clone();
        temp_board.make_move_unchecked(from_row, from_col, to_row, to_col);
        
        // Find the king's position and check if it's under attack
        let king_pos = temp_board.find_king(self.white_to_move);
        if let Some((king_row, king_col)) = king_pos {
            temp_board.is_square_under_attack(king_row, king_col, !self.white_to_move)
        } else {
            false  // Should never happen in a valid game
        }
    }

    /// Locates the king of the specified color on the board
    fn find_king(&self, is_white: bool) -> Option<(usize, usize)> {
        let target_king = if is_white { Piece::KingWhite } else { Piece::KingBlack };
        
        // Search entire board for the king
        for row in 0..8 {
            for col in 0..8 {
                if self.squares[row][col] == target_king {
                    return Some((row, col));
                }
            }
        }
        None  // King not found (should never happen)
    }

    /// Determines if a square is under attack by the specified color
    /// Used for check detection and castling validation
    fn is_square_under_attack(&self, row: usize, col: usize, by_white: bool) -> bool {
        // Check all squares for attacking pieces
        for r in 0..8 {
            for c in 0..8 {
                let piece = self.squares[r][c];
                if piece.is_empty() {
                    continue;
                }
                
                // Check if this piece belongs to the attacking color and can attack the target square
                if (by_white && piece.is_white()) || (!by_white && piece.is_black()) {
                    if self.can_piece_attack(piece, r, c, row, col) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Determines if a specific piece can attack a target square
    /// Similar to movement validation but with some differences (especially for pawns)
    fn can_piece_attack(&self, piece: Piece, from_row: usize, from_col: usize, to_row: usize, to_col: usize) -> bool {
        match piece {
            // Pawns attack diagonally only (different from their movement)
            Piece::PawnWhite => {
                let row_diff = to_row as i32 - from_row as i32;
                let col_diff = (to_col as i32 - from_col as i32).abs();
                row_diff == -1 && col_diff == 1  // White pawns attack upward diagonally
            },
            Piece::PawnBlack => {
                let row_diff = to_row as i32 - from_row as i32;
                let col_diff = (to_col as i32 - from_col as i32).abs();
                row_diff == 1 && col_diff == 1   // Black pawns attack downward diagonally
            },
            // Other pieces attack the same way they move
            Piece::RookWhite | Piece::RookBlack => {
                self.is_rook_move_valid(from_row, from_col, to_row, to_col)
            },
            Piece::KnightWhite | Piece::KnightBlack => {
                self.is_knight_move_valid(from_row, from_col, to_row, to_col)
            },
            Piece::BishopWhite | Piece::BishopBlack => {
                self.is_bishop_move_valid(from_row, from_col, to_row, to_col)
            },
            Piece::QueenWhite | Piece::QueenBlack => {
                self.is_queen_move_valid(from_row, from_col, to_row, to_col)
            },
            Piece::KingWhite | Piece::KingBlack => {
                let row_diff = (to_row as i32 - from_row as i32).abs();
                let col_diff = (to_col as i32 - from_col as i32).abs();
                row_diff <= 1 && col_diff <= 1  // King attacks adjacent squares only (no castling in attack)
            },
            _ => false,
        }
    }

    /// Executes a move without validation (used for temporary board simulation)
    fn make_move_unchecked(&mut self, from_row: usize, from_col: usize, to_row: usize, to_col: usize) {
        let piece = self.squares[from_row][from_col];
        self.squares[to_row][to_col] = piece;
        self.squares[from_row][from_col] = Piece::Empty;
    }

    /// Executes a validated move and handles all special cases
    /// This is the main move execution function
    fn make_move(&mut self, from_row: usize, from_col: usize, to_row: usize, to_col: usize) -> bool {
        // Validate the move first
        if !self.is_valid_move(from_row, from_col, to_row, to_col) {
            return false;
        }

        let piece = self.squares[from_row][from_col];
        
        // Handle en passant captures
        if matches!(piece, Piece::PawnWhite | Piece::PawnBlack) {
            if let Some((ep_row, ep_col)) = self.game_state.en_passant_target {
                if to_row == ep_row && to_col == ep_col {
                    // Remove the captured pawn (not on the destination square)
                    let captured_pawn_row = if piece.is_white() { ep_row + 1 } else { ep_row - 1 };
                    self.squares[captured_pawn_row][ep_col] = Piece::Empty;
                }
            }
            
            // Set en passant target for next turn if pawn moves two squares
            let row_diff = (to_row as i32 - from_row as i32).abs();
            if row_diff == 2 {
                let ep_row = if piece.is_white() { from_row - 1 } else { from_row + 1 };
                self.game_state.en_passant_target = Some((ep_row, from_col));
            } else {
                self.game_state.en_passant_target = None;
            }
        } else {
            // Clear en passant if it's not a pawn move
            self.game_state.en_passant_target = None;
        }

        // Handle castling - move the rook as well
        if matches!(piece, Piece::KingWhite | Piece::KingBlack) {
            let col_diff = (to_col as i32 - from_col as i32).abs();
            if col_diff == 2 {
                // This is a castling move
                let is_kingside = to_col == 6;
                let rook_from_col = if is_kingside { 7 } else { 0 };
                let rook_to_col = if is_kingside { 5 } else { 3 };
                
                // Move the rook to its new position
                let rook_piece = self.squares[from_row][rook_from_col];
                self.squares[from_row][rook_to_col] = rook_piece;
                self.squares[from_row][rook_from_col] = Piece::Empty;
            }
        }

        // Update game state to track piece movements (for castling rights)
        self.update_game_state_after_move(piece, from_row, from_col);

        // Execute the main move
        self.squares[to_row][to_col] = piece;
        self.squares[from_row][from_col] = Piece::Empty;

        // Handle pawn promotion
        if matches!(piece, Piece::PawnWhite | Piece::PawnBlack) {
            if (piece.is_white() && to_row == 0) || (piece.is_black() && to_row == 7) {
                // Automatically promote to queen (simplification)
                self.squares[to_row][to_col] = if piece.is_white() { Piece::QueenWhite } else { Piece::QueenBlack };
            }
        }

        // Switch turns
        self.white_to_move = !self.white_to_move;
        true
    }

    /// Updates game state flags after a move (for castling rights tracking)
    fn update_game_state_after_move(&mut self, piece: Piece, from_row: usize, from_col: usize) {
        match piece {
            // Kings lose castling rights when they move
            Piece::KingWhite => self.game_state.white_king_moved = true,
            Piece::KingBlack => self.game_state.black_king_moved = true,
            // Rooks lose castling rights when they move from their starting squares
            Piece::RookWhite => {
                if from_row == 7 && from_col == 0 {
                    self.game_state.white_rook_queenside_moved = true;
                } else if from_row == 7 && from_col == 7 {
                    self.game_state.white_rook_kingside_moved = true;
                }
            },
            Piece::RookBlack => {
                if from_row == 0 && from_col == 0 {
                    self.game_state.black_rook_queenside_moved = true;
                } else if from_row == 0 && from_col == 7 {
                    self.game_state.black_rook_kingside_moved = true;
                }
            },
            _ => {}
        }
    }

    /// Checks if the specified color's king is currently in check
    fn is_in_check(&self, is_white: bool) -> bool {
        if let Some((king_row, king_col)) = self.find_king(is_white) {
            self.is_square_under_attack(king_row, king_col, !is_white)
        } else {
            false  // No king found (should never happen)
        }
    }

    /// Determines if the current player is in checkmate
    /// Checkmate = in check AND no legal moves available
    fn is_checkmate(&self) -> bool {
        // Must be in check to be checkmate
        if !self.is_in_check(self.white_to_move) {
            return false;
        }

        // Try all possible moves to see if any can escape check
        for from_row in 0..8 {
            for from_col in 0..8 {
                let piece = self.squares[from_row][from_col];
                if piece.is_empty() {
                    continue;
                }
                
                // Only consider pieces belonging to the current player
                if (self.white_to_move && piece.is_white()) || (!self.white_to_move && piece.is_black()) {
                    // Try all possible destinations
                    for to_row in 0..8 {
                        for to_col in 0..8 {
                            if self.is_valid_move(from_row, from_col, to_row, to_col) {
                                return false;  // Found a legal move, not checkmate
                            }
                        }
                    }
                }
            }
        }
        
        true  // No legal moves found while in check = checkmate
    }

    /// Determines if the game is in stalemate
    /// Stalemate = NOT in check but no legal moves available
    fn is_stalemate(&self) -> bool {
        // Cannot be stalemate if in check
        if self.is_in_check(self.white_to_move) {
            return false;
        }

        // Check if any legal moves are available
        for from_row in 0..8 {
            for from_col in 0..8 {
                let piece = self.squares[from_row][from_col];
                if piece.is_empty() {
                    continue;
                }
                
                // Only consider pieces belonging to the current player
                if (self.white_to_move && piece.is_white()) || (!self.white_to_move && piece.is_black()) {
                    // Try all possible destinations
                    for to_row in 0..8 {
                        for to_col in 0..8 {
                            if self.is_valid_move(from_row, from_col, to_row, to_col) {
                                return false;  // Found a legal move, not stalemate
                            }
                        }
                    }
                }
            }
        }
        
        true  // No legal moves available while not in check = stalemate
    }
}

/// Clone implementation for Board to allow board simulation
impl Clone for Board {
    fn clone(&self) -> Self {
        Board {
            squares: self.squares,
            white_to_move: self.white_to_move,
            game_state: self.game_state.clone(),
        }
    }
}

/// Main application struct for the GUI chess game
struct ChessApp {
    board: Board,                              // The chess board state
    selected: Option<(usize, usize)>,          // Currently selected square (row, col)
    game_over: bool,                           // Whether the game has ended
    status_message: String,                    // Status/error messages to display
    square_rects: [[egui::Rect; 8]; 8],       // GUI rectangles for each board square (unused in current implementation)
}

impl Default for ChessApp {
    /// Creates a new chess application with initial state
    fn default() -> Self {
        Self {
            board: Board::new(),                // Start with standard chess position
            selected: None,                     // No square selected initially
            game_over: false,                   // Game is active
            status_message: String::new(),      // No status message
            square_rects: [[egui::Rect::NOTHING; 8]; 8],  // Initialize empty rectangles
        }
    }
}

/// Implementation of the eframe App trait for the GUI
impl App for ChessApp {
    /// Main update function called every frame by the GUI framework
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("GK Chess Engine");
            
            // Display current game status
            let current_player = if self.board.white_to_move { "Bianco" } else { "Nero" };
            if !self.game_over {
                if self.board.is_in_check(self.board.white_to_move) {
                    ui.label(format!("Turno: {} (SCACCO!)", current_player));
                } else {
                    ui.label(format!("Turno: {}", current_player));
                }
            }
            
            // Display any status or error messages
            if !self.status_message.is_empty() {
                ui.colored_label(egui::Color32::RED, &self.status_message);
            }

            ui.separator();

            // Variable to track potential drop targets (for future drag-and-drop implementation)
            let mut drop_target: Option<(usize, usize)> = None;

            // Main chess board GUI using a grid layout
            let grid_response = egui::Grid::new("chess_board").spacing([2.0, 2.0]).show(ui, |ui| {
                // Create 8x8 grid of buttons representing the chess board
                for row in 0..8 {
                    for col in 0..8 {
                        let piece = self.board.squares[row][col];
                        let is_light_square = (row + col) % 2 == 0;  // Checkerboard pattern
                        
                        // Create button text with chess piece symbol
                        let mut piece_text = egui::RichText::new(Board::piece_symbol(piece))
                            .size(50.0)
                            .strong();
                        
                        let mut button = egui::Button::new(piece_text)
                            .min_size(egui::Vec2::splat(65.0));

                        // Set base square colors (light and dark squares)
                        let base_color = if is_light_square {
                            egui::Color32::from_rgb(240, 217, 181)  // Light beige
                        } else {
                            egui::Color32::from_rgb(181, 136, 99)   // Dark brown
                        };

                        button = button.fill(base_color);

                        // Highlight the currently selected square
                        if let Some((sr, sc)) = self.selected {
                            if sr == row && sc == col {
                                button = button.fill(egui::Color32::LIGHT_BLUE);
                            }
                        }

                        // Highlight valid move destinations in green
                        if let Some((sel_row, sel_col)) = self.selected {
                            if self.board.is_valid_move(sel_row, sel_col, row, col) {
                                button = button.fill(egui::Color32::LIGHT_GREEN);
                            }
                        }

                        let response = ui.add(button);
                        
                        // Store the rectangle position for potential future use (drag-and-drop)
                        self.square_rects[row][col] = response.rect;
                        
                        // Handle square clicks if game is not over
                        if !self.game_over {
                            if response.clicked() {
                                self.handle_square_click(row, col);
                            }
                        }
                    }
                    ui.end_row();  // End this row of the grid
                }
            });

            // New Game button
            ui.separator();
            if ui.button("Nuova Partita").clicked() {
                *self = ChessApp::default();  // Reset to initial state
            }
            
            // Display instructions for the user
            ui.separator();
            ui.label("Istruzioni:");
            ui.label("• Click per selezionare un pezzo, poi click sulla casella di destinazione");
        });
    }
}

impl ChessApp {
    /// Handles user clicks on board squares
    /// Implements the two-click interface: first click selects, second click moves
    fn handle_square_click(&mut self, row: usize, col: usize) {
        if let Some((from_row, from_col)) = self.selected {
            // A square is already selected
            if (from_row, from_col) == (row, col) {
                // Clicked on the same square - deselect
                self.selected = None;
            } else {
                // Clicked on a different square - attempt to make a move
                if self.board.make_move(from_row, from_col, row, col) {
                    // Move was successful
                    self.selected = None;
                    
                    // Check for game ending conditions
                    if self.board.is_checkmate() {
                        let winner = if self.board.white_to_move { "Nero" } else { "Bianco" };
                        self.status_message = format!("SCACCO MATTO! {} vince!", winner);
                        self.game_over = true;
                    } else if self.board.is_stalemate() {
                        self.status_message = "STALLO! La partita è patta!".to_string();
                        self.game_over = true;
                    } else {
                        self.status_message.clear();  // Clear any previous messages
                    }
                } else {
                    // Move was invalid - try to select the new square instead
                    let piece = self.board.squares[row][col];
                    if !piece.is_empty() && 
                       ((self.board.white_to_move && piece.is_white()) || 
                        (!self.board.white_to_move && piece.is_black())) {
                        // Valid piece for current player - select it
                        self.selected = Some((row, col));
                        self.status_message.clear();
                    } else {
                        // Invalid selection
                        self.status_message = "Mossa non valida!".to_string();
                    }
                }
            }
        } else {
            // No square currently selected - try to select this square
            let piece = self.board.squares[row][col];
            if !piece.is_empty() {
                if (self.board.white_to_move && piece.is_white()) || 
                   (!self.board.white_to_move && piece.is_black()) {
                    // Valid piece for current player
                    self.selected = Some((row, col));
                    self.status_message.clear();
                } else {
                    // Trying to select opponent's piece
                    self.status_message = "Non puoi muovere i pezzi dell'avversario!".to_string();
                }
            }
            // If clicking on empty square with nothing selected, do nothing
        }
    }
}

fn main() {
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(750.0, 900.0))
            .with_title("GK Chess Engine"),
        ..Default::default()
    };
    eframe::run_native("GK Chess", native_options, Box::new(|_cc| Box::new(ChessApp::default())));
}