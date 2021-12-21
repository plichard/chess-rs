use std::cmp::Ordering;
use crate::board2::*;

#[derive(Clone)]
pub struct Game {
    board: Board,
    move_stack: Vec<Move>,
    move_count: usize,
}


impl Game {
    pub fn new_empty() -> Game {
        Game {
            board: Board::new(),
            move_stack: Vec::with_capacity(100),
            move_count: 0,
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
        self.move_count += 1;
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

    pub fn turn(&self) -> Color {
        if self.move_stack.len() % 2 == 0 {
            Color::White
        } else {
            Color::Black
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn evaluate_position(&mut self) -> i32 {
        // self.compute_attacked_cells();
        let mut black_value = {
            let mut sum: i32 = 0;
            for piece in self.board.black_pieces() {
                if piece.active() {
                    sum += piece.t().value();
                }
            }
            sum
        };
        let mut white_value = {
            let mut sum: i32 = 0;
            for piece in self.board.white_pieces() {
                if piece.active() {
                    sum += piece.t().value();
                }
            }
            sum
        };


        let perspective = if self.turn() == Color::White {
            1
        } else {
            -1
        };
        (white_value - black_value) * perspective
    }

    pub fn evaluate_move_value(&self, m: &Move) -> i32 {
        match m.action {
            Action::Move { .. } => 0,
            Action::Capture { target } => {
                self.board.piece_from_ref(target).t().value() / 8 -
                    self.board.piece_from_ref(m.piece_ref).t().value() / 16
            }
            Action::CaptureAndPromote { target, t } => {
                self.board.piece_from_ref(target).t().value() / 8 -
                    self.board.piece_from_ref(m.piece_ref).t().value() / 16 +
                    t.value()
            }
            Action::Evaluate => m.score,
            Action::Promote { t, .. } => t.value(),
            Action::Castle => 1000,
            Action::None => 0,
        }
    }

    pub fn sort_moves(&self, moves: &mut [Move]) {
        moves.sort_by(|lh, rh| {
            let v1 = self.evaluate_move_value(lh);
            let v2 = self.evaluate_move_value(rh);
            if v1 > v2 {
                Ordering::Less
            } else if v2 > v1 {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
    }

    pub fn find_best_move(&mut self, depth: i32, buffer: &mut [Move]) -> Option<Move> {
        self.move_count = 0;
        let mut root_node = Move::new_evaluate(0);
        let (m, count) = self.search(
            depth,
            Move::new_evaluate(-i32::MAX),
            Move::new_evaluate(i32::MAX),
            &mut root_node,
            &mut buffer[0..],
        );
        // if !m.is_valid() {
        //     return None;
        // }

        println!("score = {}, move count = {}/{}", m.score, self.move_count, count);

        return Some(m);
    }

    pub fn search(
        &mut self,
        depth: i32,
        mut alpha: Move,
        beta: Move,
        parent: &mut Move,
        buffer: &mut [Move],
    ) -> (Move, usize) {
        // if rx.try_recv().is_ok() {
        //     return Move::evaluate(self.evaluate_position());
        // }

        if let Action::Capture { target, .. } = &parent.action {
            if self.board.piece_from_ref(*target).t() == Type::King {
                return (Move::new_evaluate(self.evaluate_position()), 0);
            }
        }

        if depth == 0 {
            return (Move::new_evaluate(self.evaluate_position()), 0);
            // return self.search(depth - 1, alpha, beta, parent, buffer, true);
        }

        // if depth == 0 {
        //     return Move::evaluate(self.evaluate_position());
        // }

        // if only_captures && depth < -3 {
        //     // return Move{ score: 0, action: Action::Evaluation {score: self.evaluate_position()}};
        //     return Move::new_evaluate(self.evaluate_position());
        // }

        let mut count = self.board.insert_all_moves(self.turn(), buffer);
        let (moves, mut buffer) = buffer.split_at_mut(count);

        if moves.is_empty() {
            return (Move::new_evaluate(self.evaluate_position()), 0);
        }

        self.sort_moves(moves);

        // let moves = if only_captures {
        //     &mut moves
        // } else {
        //     parent.children = moves;
        //     &mut parent.children
        // };

        // let moves = &mut moves;

        for m in moves {
            self.push_move(*m);
            let (test_move, tmp) = self.search(depth - 1, -beta, -alpha, m, buffer);
            let test_move = -test_move;
            buffer = &mut buffer[tmp..];
            count += tmp;

            // println!("test move: {}", test_move.evaluation);
            self.pop_move();


            if test_move.score >= beta.score {
                // println!("Pruning");
                return (beta, count);
            }

            if test_move.score > alpha.score {
                alpha = *m;
                alpha.score = test_move.score;
            }
        }

        return (alpha, count);
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

        use rand::seq::SliceRandom;

        let rng = &mut rand::thread_rng();

        // play for 100 moves max
        for _ in 0..100 {
            let count = game.board.insert_all_moves(color, &mut initial_buffer[0..]);
            if count == 0 {
                break;
            }
            let m = initial_buffer[0..count].choose(rng).unwrap();
            println!("{}", m);
            game.push_move(*m);
            game.board.assert_consistency();
            println!("{:?}", game.board);
            color = color.other();
        }

        println!("FINAL POSITION\n{:#?}", game.board());

        while game.pop_move().is_some() {
            game.board.assert_consistency();
        }

        assert_eq!(board, game.board);
    }

    #[test]
    fn classic_game_start() {
        let mut game = Game::new_classic_game();
        let mut initial_buffer = vec![Move::none(); 1_000];
        let count = game.board.insert_all_moves(Color::White, &mut initial_buffer[0..]);
        assert_eq!(count, 20);
    }


    #[test]
    fn classic_game_full() {
        for _ in 0..20 {
            let mut game = Game::new_classic_game();
            let mut initial_buffer = vec![Move::none(); 1_000];
            let board = game.board.clone();
            let count = game.board.insert_all_moves(Color::White, &mut initial_buffer[0..]);
            assert_eq!(count, 20);

            let mut color = Color::White;

            use rand::seq::SliceRandom;

            let rng = &mut rand::thread_rng();

            // play for 1000 moves max

            for _ in 0..1000 {
                let count = game.board.insert_all_moves(color, &mut initial_buffer[0..]);
                if count == 0 {
                    break;
                }

                let moves = &initial_buffer[0..count];
                // let moves: Vec<&Move> = moves.iter().filter(|m| match m.action() {
                //     Action::Castle => false,
                //     _ => true
                // }).collect();
                // if moves.len() == 0 {
                //     break;
                // }
                // let m = moves[0];
                let m = if let Some(m) = moves.iter().find(|m| match m.action() {
                    Action::Castle => true,
                    _ => false
                }) {
                    m
                } else {
                    moves.choose(rng).unwrap()
                };

                println!("{}", m);
                game.push_move(*m);
                game.board.assert_consistency();
                println!("{:?}", game.board);
                color = color.other();
            }

            println!("FINAL POSITION\n{:#?}", game.board());

            while game.pop_move().is_some() {
                game.board.assert_consistency();
            }

            assert_eq!(board, game.board);
        }
    }
}