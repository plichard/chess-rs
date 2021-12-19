use crate::board2::*;

struct Game {
    board: Board,
}


impl Game {
    pub fn new_empty() -> Game {
        Game {
            board: Board::new()
        }
    }

    pub fn new_classic() -> Game {
        let mut game = Game::new_empty();
        for x in 0..7 {
            game.board.add_new_piece(Color::White, Type::Pawn, x, 1);
            game.board.add_new_piece(Color::Black, Type::Pawn, x, 6);
        }
        game
    }

    pub fn do_stuff(&mut self) {
        let mut buffer = vec![Move::none(); 1_000_000];
        let (moves, buffer) = self.board.insert_all_moves(Color::White, &mut buffer[0..]);
    }
}


// #[cfg(test)]
// mod tests {
//     #[test]
//     fn classic_game() {
//         use super::*;
//         let mut game = Game::new_classic();
//
//     }
// }