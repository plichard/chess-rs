use crate::board2::*;


pub struct Game {
    board: Board,
    move_stack: Vec<Move>,
}


impl Game {
    pub fn new_empty() -> Game {
        Game {
            board: Board::new(),
            move_stack: Vec::with_capacity(100),
        }
    }

    pub fn new_pawn_only() -> Game {
        let mut game = Game::new_empty();
        for x in 0..8 {
            game.board.add_new_piece(Color::White, Type::Pawn, x, 1);
            game.board.add_new_piece(Color::Black, Type::Pawn, x, 6);
        }
        game
    }

    pub fn new_classic_game() -> Game {
        use Color::*;
        use Type::*;
        let mut game = Game::new_empty();
        for x in 0..8 {
            game.add_piece(White, Pawn, x, 1);
            game.add_piece(Black, Pawn, x, 6);
        }

        game.add_piece(White, Rook, 0, 0);
        game.add_piece(White, Rook, 7, 0);
        game.add_piece(Black, Rook, 0, 7);
        game.add_piece(Black, Rook, 7, 7);

        game.add_piece(White, Knight, 1, 0);
        game.add_piece(White, Knight, 6, 0);
        game.add_piece(Black, Knight, 1, 7);
        game.add_piece(Black, Knight, 6, 7);

        game.add_piece(White, Bishop, 2, 0);
        game.add_piece(White, Bishop, 5, 0);
        game.add_piece(Black, Bishop, 2, 7);
        game.add_piece(Black, Bishop, 5, 7);


        game.add_piece(White, Queen, 3, 0);
        game.add_piece(Black, Queen, 3, 7);

        game.add_piece(White, King, 4, 0);
        game.add_piece(Black, King, 4, 7);

        game
    }

    pub fn push_move(&mut self, m: Move) {
        self.move_stack.push(m);
        self.board.make_move(&m);
    }

    pub fn pop_move(&mut self) -> Option<Move> {
        let maybe_move = self.move_stack.pop();
        if let Some(m) = &maybe_move {
            self.board.revert_move(m);
        }

        maybe_move
    }

    pub fn add_piece(&mut self, color: Color, t: Type, x: i8, y: i8) {
        self.board.add_new_piece(color, t, x, y);
    }

    pub fn get_all_moves(&self, color: Color) -> Vec<Move> {
        let mut moves = vec![Move::none(); 1000];
        let count = self.board.insert_all_moves(color, &mut moves[0..]);
        Vec::from(&moves[0..count])
    }

    pub fn board(&self) -> &Board {
        &self.board
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_piece() {
        let mut game = Game::new_empty();
        game.add_piece(Color::White, Type::Pawn, 1, 2);

        let moves = game.get_all_moves(Color::White);
        assert_eq!(moves.len(), 1);
        assert!(game.board().piece_at(Position::new(1, 2)).is_some());
        game.push_move(moves[0]);
        assert!(game.board().piece_at(Position::new(1, 2)).is_none());
        game.pop_move();
        assert!(game.board().piece_at(Position::new(1, 2)).is_some());
    }

    #[test]
    fn capture_and_promote_piece() {
        let mut game = Game::new_empty();
        let p1 = Position::new(1, 6);
        let p2 = Position::new(2, 7);
        game.add_piece(Color::White, Type::Pawn, 1, 7);
        game.add_piece(Color::White, Type::Pawn, p1.x(), p1.y());
        game.add_piece(Color::Black, Type::Pawn, p2.x(), p2.y());

        let piece1 = *game.board().piece_at(p1).unwrap();
        let piece2 = *game.board().piece_at(p2).unwrap();


        let moves = game.get_all_moves(Color::White);
        assert_eq!(moves.len(), 1);
        assert!(game.board().piece_at(p1).is_some());
        assert!(game.board().piece_at(p2).is_some());
        game.push_move(moves[0]);
        assert!(game.board().piece_at(p1).is_none());
        assert!(game.board().piece_at(p2).is_some());
        assert_eq!(game.board().piece_at(p2).unwrap().t(), Type::Queen);
        game.pop_move();
        assert!(game.board().piece_at(p1).is_some());
        assert!(game.board().piece_at(p2).is_some());
        assert_eq!(game.board().piece_at(p1).unwrap().t(), Type::Pawn);

        assert_eq!(*game.board().piece_at(p1).unwrap(), piece1);
        assert_eq!(*game.board().piece_at(p2).unwrap(), piece2);
    }

    #[test]
    fn full_pawn_moves() {
        let mut game = Game::new_pawn_only();
        let mut initial_buffer = vec![Move::none(); 1_000];
        let board = game.board.clone();
        let count = game.board.insert_all_moves(Color::White, &mut initial_buffer[0..]);
        assert_eq!(count, 16);

        let mut color = Color::White;

        while true {
            let count = game.board.insert_all_moves(color, &mut initial_buffer[0..]);
            if count == 0 {
                break;
            }
            game.push_move(initial_buffer[0]);
            color = color.other();
        }

        println!("FINAL POSITION\n{:#?}", game.board());

        while game.pop_move().is_some() {}

        assert_eq!(board, game.board);
    }

    #[test]
    fn classic_game() {
        let mut game = Game::new_classic_game();
        let mut initial_buffer = vec![Move::none(); 1_000];
        let count = game.board.insert_all_moves(Color::White, &mut initial_buffer[0..]);
        assert_eq!(count, 20);
    }
}