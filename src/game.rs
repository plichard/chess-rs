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
        for x in 0..7 {
            game.board.add_new_piece(Color::White, Type::Pawn, x, 1);
            game.board.add_new_piece(Color::Black, Type::Pawn, x, 6);
        }
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

    pub fn do_testing(&mut self) {
        let mut buffer = vec![Move::none(); 1_000];
        while true {
            let (moves, buffer) = self.board.insert_all_moves(Color::White, &mut buffer[0..]);
            if moves.is_empty() {
                println!("no more moves, breaking");
                break;
            }
            println!("pushing {}", moves[0]);
            self.push_move(moves[0]);
        }

        while self.pop_move().is_some() {}
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn moves() {
        use super::{Move, Game, Color};
        let mut game = Game::new_pawn_only();
        let mut buffer = vec![Move::none(); 1_000];
        let board = game.board.clone();
        while true {
            let (moves, buffer) = game.board.insert_all_moves(Color::White, &mut buffer[0..]);
            if moves.is_empty() {
                break;
            }
            game.push_move(moves[0]);
        }

        while game.pop_move().is_some() {}

        assert_eq!(board, game.board);
    }
}