use rand::Rng;

/// Type of spot in a minefield
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SpotKind {
    /// This spot is a mine
    Mine,

    /// This is an empty spot, surrounded by N mines
    Empty(i32),
}

/// State of the spot in a minefield
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SpotState {
    /// This spot has not been visited
    Hidden,

    /// This spot has been visited
    Revealed,

    /// This spot has been flagged as being a mine
    Flagged,
}

/// Spot struct describing the characteristics of the minefield at a particular position
#[derive(Copy, Clone, Debug)]
pub struct Spot {
    kind: SpotKind,
    state: SpotState,
}

impl Spot {
    pub fn kind(&self) -> &SpotKind {
        &self.kind
    }

    pub fn state(&self) -> &SpotState {
        &self.state
    }
}

impl Default for Spot {
    fn default() -> Self {
        Self { kind: SpotKind::Empty(0), state: SpotState::Hidden }
    }
}

/// The result of steppin on a spot in the minefield
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum StepResult {
    /// Stepped on empty spot
    Phew,

    /// Stepped on a mine
    Boom,

    /// Step not taken
    Invalid
}

/// The characteristics of the minefield
#[derive(Clone, Debug)]
pub struct Minefield {
    field: Vec<Spot>,

    /// Number of mines in the field
    mines: i32,

    /// Width of field grid
    width: i32,

    /// Height of field grid
    height: i32,
}

impl Minefield {
    /// Create an empty minefield grid (with all spots hidden), with the given width and height
    pub fn new(width: u16, height: u16) -> Self {
        // Enforce a minimum number of spots
        let width = if width < 3 { 3 } else { width as i32 };
        let height = if height == 0 { 1 } else { height as i32 };

        // Total number of spots in our field
        let spot_count = width as usize * height as usize;

        // Create empty field, with all spots hidden
        let field = vec![Spot::default(); spot_count];

        // Create empty Minefield
        Minefield {
            field,
            mines: 0,
            width,
            height,
        }
    }

    /// Build an existing minefield with the given number of mines randomly placed in it
    pub fn with_mines(mut self, mines: u16) -> Self {
        // Total number of spots in our field
        let spot_count = self.width as usize * self.height as usize;

        // Limit the max number of mines to the number of available spots
        let mines = if mines as usize <= spot_count { mines as i32 } else { spot_count as i32 };

        // Add mines to minefield

        // We could just start randomly picking indices in the field and hope we haven't picked them before, but if a
        // user desires a field full of mines, then waiting for the last mines to be placed might take a long time
        // (e.g. if the field is very large).
        // That's a problem for an immediate GUI.
        // So, instead, we'll use some memory in order to ensure that the user can step on a mine as soon as humanly
        // possible.
        let mut spots_remaining: Vec<usize> = (0..spot_count).collect();
        let mut rng = rand::thread_rng();

        // Place mines
        for _ in 0..mines {
            let index_rm = rng.gen_range(0..spots_remaining.len());
            self.place_mine(spots_remaining.swap_remove(index_rm));
        }

        self
    }

    /// Step on a given spot of the field. Coordinates [x=0, y=0] represent the top-left point of the field grid
    pub fn step(&mut self, x: u16, y: u16) -> StepResult {
        if let Some(index) = self.spot_index(x as i32, y as i32) {
            match self.field[index].kind {
                SpotKind::Mine => {
                    // Reveal the spot
                    self.field[index].state = SpotState::Revealed;

                    // Stepped on a mine
                    StepResult::Boom
                },

                SpotKind::Empty(n) => {
                    // Reveal the spot
                    self.field[index].state = SpotState::Revealed;

                    // flood reveal if this is an empty spot with no neighboring mines
                    if n == 0 {
                        self.flood_neighbors_reveal(index);
                    }

                    // Stepped on empty field
                    StepResult::Phew
                },
            }
        } else {
            // Step is outside minefield
            StepResult::Invalid
        }
    }

    /// Try to reveal neighboring spots, if user has placed enough flags.
    /// 
    /// If the current empty revealed spot has `N` neighboring mines (`N > 0`), and if the user has flagged `N` mines, 
    /// then this method reveals all neighboring hidden spots. If the user misplaced a flag, then this will result in a
    /// `Boom`.
    pub fn try_resolve_step(&mut self, x: u16, y: u16) -> StepResult {
        if let Some(index) = self.spot_index(x as i32, y as i32) {
            if let SpotState::Revealed = self.field[index].state {
                if let SpotKind::Empty(n) = self.field[index].kind {
                    if n > 0 {
                        let flag_count: i32 = self
                            .neighbor_indices(index)
                            .filter(|i| {
                                self.field[*i].state == SpotState::Flagged
                            })
                            .map(|_| 1)
                            .sum();
                        
                        if n == flag_count {
                            for neighbor_index in self.neighbor_indices(index) {
                                if self.field[neighbor_index].state == SpotState::Hidden {
                                    let (x, y) = self.spot_coords(neighbor_index);
                                    let step_result = self.step(x as u16, y as u16);
                                    if step_result == StepResult::Boom {
                                        return step_result;
                                    }
                                }
                            }

                            return  StepResult::Phew;
                        }
                    }
                }
            }
        }

        StepResult::Invalid
    }

    // Set a flag on a hidden spot, or clear the flag if the spot had one
    pub fn flag(&mut self, x: u16, y: u16) {
        if let Some(index) = self.spot_index(x as i32, y as i32) {
            match self.field[index].state {
                SpotState::Hidden => {
                    self.field[index].state = SpotState::Flagged;
                },
                SpotState::Flagged => {
                    self.field[index].state = SpotState::Hidden;
                },
                SpotState::Revealed => {},
            }
        }
    }

    pub fn width(&self) -> u16 {
        self.width as u16
    }

    pub fn height(&self) -> u16 {
        self.height as u16
    }

    pub fn mines(&self) -> u16 {
        self.mines as u16
    }    

    pub fn spot(&self, x: u16, y: u16) -> Option<&Spot> {
        if let Some(index) = self.spot_index(x as i32, y as i32) {
            Some(&self.field[index])
        } else {
            None
        }
    }

    /// Flood reveal the neighbors of the spot corresponding to the given `index`
    fn flood_neighbors_reveal(&mut self, index: usize) {
        let mut neighbors_to_visit = vec![index];

        while let Some(index) = neighbors_to_visit.pop() {
            for neighbor_index in self.neighbor_indices(index) {
                if let SpotState::Hidden = self.field[neighbor_index].state {
                    if let SpotKind::Empty(n) = self.field[neighbor_index].kind {
                        self.field[neighbor_index].state = SpotState::Revealed;

                        if n == 0 {
                            neighbors_to_visit.push(neighbor_index);
                        }
                    }
                }
            }
        }
    }

    /// Place a mine at a given field index, and update neighboring spots
    fn place_mine(&mut self, index: usize) {
        assert!(index < self.field.len());

        // Only place a mine in an emty field
        if let SpotKind::Empty(_) = self.field[index].kind {
            // place the mine
            self.field[index].kind = SpotKind::Mine;

            // update neighboring empty spots
            for neighbor_index in self.neighbor_indices(index) {
                if let SpotKind::Empty(n) = &mut self.field[neighbor_index as usize].kind {
                    // increment count of neighboring mines for this spot
                    *n += 1;
                }
            }
        }
    }

    /// Get an iterator over the indices neighboring a given index in the minefield grid
    fn neighbor_indices(&self, index: usize) -> impl Iterator<Item = usize> {
        assert!(index < self.field.len());

        let width = self.width;

        let base_index = index as i32;
        let index_start = base_index - (width + 1);
        let index_end = base_index + (width + 1);

        let high_index_start = index_start;
        let high_index_end = index_start + 2;
        let high_iter = high_index_start..=high_index_end;

        let mid_index_start = base_index - 1;
        let mid_index_end = base_index + 1;
        let mid_iter = mid_index_start..=mid_index_end;

        let low_index_start = index_end - 2;
        let low_index_end = index_end;
        let low_iter = low_index_start..=low_index_end;

        let index_max = self.field.len() as i32;
        let y = base_index / width;
        let x = base_index % width;

        // Return the neighboring spots iterator
        high_iter.chain(mid_iter.chain(low_iter))
            .filter(move |i| {
                let ny = *i / width;
                let nx = *i % width;

                // the index is within the field vector
                (*i >= 0 && *i < index_max)
                // the index corresponds to a neighbor
                && (*i != base_index)

                // the index corresponds to a set of coordinates which is within 1 unit far from the coords of `base_index`
                && ((ny >= (y - 1)) && (ny <= (y + 1)))
                && ((nx >= (x - 1)) && (nx <= (x + 1)))
            })
            .map(|i| {
                i as usize
            })
    }

    /// Try to get the field index corresponding to the given field grid coordiantes
    fn spot_index(&self, x: i32, y: i32) -> Option<usize> {
        if (x >= 0) && (x < self.width) && (y >= 0) && (y < self.height) {
            Some((y as usize * self.width as usize) + x as usize)
        } else {
            // Coords are outside of field
            None
        }
    }

    /// Calculate the field grid coordinated corresponding to the given field index
    fn spot_coords(&self, index: usize) -> (i32, i32) {
        let index = index as i32;
        let width = self.width;

        (index % width, index / width)
    }
}

 #[cfg(test)]
 mod tests {
     use super::*;

     #[test]
     fn new_minefield() {
        // Create empty test minefield:
        //     0 1 2
        // 0 [       ]
        // 1 [       ]
        // 2 [       ]
        // 3 [       ]
        //
        let width = 3;
        let height = 4;
        let minefield = Minefield::new(width, height);

        for spot in &minefield.field {
            assert_eq!(spot.kind, SpotKind::Empty(0));
            assert_eq!(spot.state, SpotState::Hidden);
        }
     }

     #[test]
     fn place_mines() {
         // Create empty minefield
        let width = 3;
        let height = 4;
        let mut minefield = Minefield::new(width, height);

        // Place Mine
        //     0 1 2
        // 0 [   1 ☢ ]
        // 1 [   1 1 ]
        // 2 [       ]
        // 3 [       ]
        //
        let mine_x = 2;
        let mine_y = 0;
        minefield.place_mine(minefield.spot_index(mine_x, mine_y).unwrap());

        let mine_index = minefield.spot_index(mine_x, mine_y);
        assert_eq!(mine_index, Some(2));

        // Was mine placed correctly?
        let mine_index = mine_index.unwrap();
        assert_eq!(minefield.field[mine_index].kind, SpotKind::Mine);

        // Were the neighbors updated correctly?
        for neighbor_index in minefield.neighbor_indices(mine_index) {
            assert_eq!(minefield.field[neighbor_index].kind, SpotKind::Empty(1));
        }

        // Place another mine
        //     0 1 2
        // 0 [   1 ☢ ]
        // 1 [   1 1 ]
        // 2 [ 1 1   ]
        // 3 [ ☢ 1   ]
        let mine_x = 0;
        let mine_y = 3;
        minefield.place_mine(minefield.spot_index(mine_x, mine_y).unwrap());

        let mine_index = minefield.spot_index(mine_x, mine_y);
        assert_eq!(mine_index, Some(9));

        // Was mine placed correctly?
        let mine_index = mine_index.unwrap();
        assert_eq!(minefield.field[mine_index].kind, SpotKind::Mine);

        // Were the neighbors updated correctly?
        for neighbor_index in minefield.neighbor_indices(mine_index) {
            assert_eq!(minefield.field[neighbor_index].kind, SpotKind::Empty(1));
        }

        // Place another mine
        //     0 1 2
        // 0 [ 1 2 ☢ ]
        // 1 [ ☢ 2 1 ]
        // 2 [ 2 2   ]
        // 3 [ ☢ 1   ]
        let mine_x = 0;
        let mine_y = 1;
        minefield.place_mine(minefield.spot_index(mine_x, mine_y).unwrap());

        let mine_index = minefield.spot_index(mine_x, mine_y);
        assert_eq!(mine_index, Some(3));

        // Was mine placed correctly?
        let mine_index = mine_index.unwrap();
        assert_eq!(minefield.field[mine_index].kind, SpotKind::Mine);

        // Were the neighbors updated correctly?
        for neighbor_index in minefield.neighbor_indices(mine_index) {
            let n_coords = minefield.spot_coords(neighbor_index);
            let expected_mine_count = if n_coords == (0, 0) { 1 } else { 2 };
            assert_eq!(minefield.field[neighbor_index].kind, SpotKind::Empty(expected_mine_count));
        }
     }

     #[test]
     fn step() {
         // Create empty minefield
         let width = 3;
         let height = 4;
         let mut minefield = Minefield::new(width, height);

        // Place mines
        //     0 1 2
        // 0 [   1 ☢ ]
        // 1 [   1 1 ]
        // 2 [ 1 1   ]
        // 3 [ ☢ 1   ]
        let mine_x = 2;
        let mine_y = 0;
        minefield.place_mine(minefield.spot_index(mine_x, mine_y).unwrap());
        let mine_x = 0;
        let mine_y = 3;
        minefield.place_mine(minefield.spot_index(mine_x, mine_y).unwrap());

        // Step on spot neighboring mine
        let step_x = 1;
        let step_y = 2;
        let step_result = minefield.step(step_x, step_y);

        // Step was success, and only one spot was revealed
        //     0 1 2
        // 0 [ • • • ]
        // 1 [ • • • ]
        // 2 [ • 1 • ]
        // 3 [ • • • ]
        assert_eq!(step_result, StepResult::Phew);
        let step_index = minefield.spot_index(step_x as i32, step_y as i32).unwrap();
        assert_eq!(minefield.field[step_index].state, SpotState::Revealed);
        for neighbor_index in minefield.neighbor_indices(step_index) {
            assert_eq!(minefield.field[neighbor_index].state, SpotState::Hidden);
        }

        // Step on spot with no neighboring mines
        let step_x = 0;
        let step_y = 1;
        let step_result = minefield.step(step_x, step_y);

        // Step was success, and neighbors were flood revealed
        //     0 1 2
        // 0 [   1 • ]
        // 1 [   1 • ]
        // 2 [ 1 1 • ]
        // 3 [ • • • ]
        assert_eq!(step_result, StepResult::Phew);
        let step_index = minefield.spot_index(step_x as i32, step_y as i32).unwrap();
        assert_eq!(minefield.field[step_index].state, SpotState::Revealed);
        for neighbor_index in minefield.neighbor_indices(step_index) {
            assert_eq!(minefield.field[neighbor_index].state, SpotState::Revealed);
        }

        // Step on mine
        let step_x = 2;
        let step_y = 0;
        let step_result = minefield.step(step_x, step_y);

        // Step was Boom, and only mine spot was newly revealed
        //     0 1 2
        // 0 [   1 ☢ ]
        // 1 [   1 • ]
        // 2 [ 1 1 • ]
        // 3 [ • • • ]
        assert_eq!(step_result, StepResult::Boom);
        let step_index = minefield.spot_index(step_x as i32, step_y as i32).unwrap();
        assert_eq!(minefield.field[step_index].state, SpotState::Revealed);
        for neighbor_index in minefield.neighbor_indices(step_index) {
            let n_coords = minefield.spot_coords(neighbor_index);
            let expected_spot_state= if n_coords == (2, 1) { SpotState::Hidden} else { SpotState::Revealed };
            assert_eq!(minefield.field[neighbor_index].state, expected_spot_state);
        }
     }

     #[test]
     fn flood_reveal() {
        // Create empty bigger minefield
        //     0 1 2 3 4 5 6 7 8 9
        // 0 [     1 ☢ 1           ]
        // 1 [     1 1 1           ]
        // 2 [           1 1 1     ]
        // 3 [   1 1 1   1 ☢ 1 1 1 ]
        // 4 [   1 ☢ 1   1 1 1 1 ☢ ]
        // 5 [   1 1 1         1 1 ]
        // 6 [         1 1 2 1 1   ]
        // 7 [         1 ☢ 2 ☢ 1   ]
        // 8 [         1 1 2 1 1   ]
        // 9 [                     ]
        let width = 10;
        let height = 10;
        let mut minefield = Minefield::new(width, height);

        let mine_coords = [(2, 4), (5, 7), (7, 7), (9, 4), (6, 3), (3, 0)];
        for (x, y) in mine_coords {
            minefield.place_mine(minefield.spot_index(x, y).unwrap());
        }

        // Place a flag
        //     0 1 2 3 4 5 6 7 8 9
        // 0 [ • • • • • • • • • • ]
        // 1 [ • • • • • ⚐ • • • • ]
        // 2 [ • • • • • • • • • • ]
        // 3 [ • • • • • • • • • • ]
        // 4 [ • • • • • • • • • • ]
        // 5 [ • • • • • • • • • • ]
        // 6 [ • • • • • • • • • • ]
        // 7 [ • • • • • • • • • • ]
        // 8 [ • • • • • • • • • • ]
        // 9 [ • • • • • • • • • • ]
        let flag_x = 5;
        let flag_y = 1;
        let flag_index = minefield.spot_index(flag_x, flag_y).unwrap();
        minefield.field[flag_index].state = SpotState::Flagged;

        // Step on spot (x=9, y=6)
        //     0 1 2 3 4 5 6 7 8 9
        // 0 [     1 • • • • • • • ]
        // 1 [     1 1 1 ⚐ • • • • ]
        // 2 [           1 • • • • ]
        // 3 [   1 1 1   1 • • • • ]
        // 4 [   1 • 1   1 1 1 1 • ]
        // 5 [   1 1 1         1 1 ]
        // 6 [         1 1 2 1 1   ]
        // 7 [         1 • • • 1   ]
        // 8 [         1 1 2 1 1   ]
        // 9 [                     ]
        let step_x = 9;
        let step_y = 6;
        let step_result = minefield.step(step_x, step_y);
        assert_eq!(step_result, StepResult::Phew);

        // All mines are still hidden
        for (x, y) in mine_coords {
            let index = minefield.spot_index(x, y).unwrap();
            assert_eq!(minefield.field[index].state, SpotState::Hidden);
        }

        // Flood revealed the entire maze
        let index = minefield.spot_index(7, 5).unwrap();
        assert_eq!(minefield.field[index].state, SpotState::Revealed);

        // Flag is still there
        let index = minefield.spot_index(flag_x, flag_y).unwrap();
        assert_eq!(minefield.field[index].state, SpotState::Flagged);

        // Insulated portion of field is still hidden
        let index = minefield.spot_index(9, 0).unwrap();
        assert_eq!(minefield.field[index].state, SpotState::Hidden);
        let index = minefield.spot_index(7, 1).unwrap();
        assert_eq!(minefield.field[index].state, SpotState::Hidden);
     }

     #[allow(dead_code)]
     fn print_minefield(minefield: &Minefield) {
        // X axis
        println!();
        print!("   ");
        for y in 0..minefield.width {
            print!(" {}", y);
        }
        println!();

        for y in 0..minefield.height {
            // Y Axis
            print!("{:?} [", y);
            for x in 0..minefield.width {
                if let Some(index) = minefield.spot_index(x as i32, y as i32) {
                    match minefield.field[index].kind {
                        SpotKind::Mine => {
                            print!(" ☢");
                        },
                        SpotKind::Empty(n) => {
                            if n > 0 {
                                print!(" {}", n);
                            } else {
                                print!("  ");
                            }
                        },
                    }
                }
            }
            println!(" ]");
        }
     }

     #[allow(dead_code)]
     fn print_minefield_state(minefield: &Minefield) {
        // X axis
        println!();
        print!("   ");
        for y in 0..minefield.width {
            print!(" {}", y);
        }
        println!();

        for y in 0..minefield.height {
            // Y Axis
            print!("{:?} [", y);
            for x in 0..minefield.width {
                if let Some(index) = minefield.spot_index(x as i32, y as i32) {
                    match minefield.field[index].state {
                        SpotState::Hidden => {
                            print!(" •");
                        },
                        SpotState::Flagged => {
                            print!(" ⚐");
                        },
                        SpotState::Revealed => {
                            match minefield.field[index].kind {
                                SpotKind::Mine => {
                                    print!(" ☢");
                                },
                                SpotKind::Empty(n) => {
                                    if n > 0 {
                                        print!(" {}", n);
                                    } else {
                                        print!("  ");
                                    }
                                },
                            }
                        },
                    }
                }
            }
            println!(" ]");
        }
     }
 }