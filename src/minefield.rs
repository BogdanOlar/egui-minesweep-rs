use rand::Rng;

#[derive(Copy, Clone, Debug)]
pub enum SpotKind {
    /// This spot is a mine
    Mine,

    /// This is an empty spot, surrounded by N mines
    Field(i32),
}

#[derive(Copy, Clone, Debug)]
pub enum SpotState {
    /// This spot has not been visited
    Hidden,

    /// This spot has been visited
    Revealed
}

#[derive(Copy, Clone, Debug)]
pub struct Spot {
    kind: SpotKind,
    state: SpotState,
}

impl Default for Spot {
    fn default() -> Self {
        Self { kind: SpotKind::Field(0), state: SpotState::Hidden }
    }
}

#[derive(Clone, Debug)]
pub struct Minefield {
    field: Vec<Spot>,
    mines: u32,
    width: u16,
    height: u16,
}

impl Minefield {
    pub fn new(mines: u16, width: u16, height: u16) -> Self {
        // Enforce a minimum number of spots to 1. No empty fields!
        let width = if width == 0 { 1 } else { width };
        let height = if height == 0 { 1 } else { height };

        // Total number of spots in our field
        let spot_count = width as usize * height as usize;

        // Create empty field
        let mut field = vec![Spot::default(); spot_count];

        // Limit the max number of mines to the number of available spots
        let mines = if mines as usize <= spot_count { mines as u32 } else { spot_count as u32 };

        // Create empty Minefield
        let mut minefield = Minefield {
            field,
            mines,
            width,
            height,
        };

        // Add mines to minefield
        let mut spots_remaining: Vec<usize> = (0..spot_count).collect();
        let mut rng = rand::thread_rng();

        for _ in 0..mines {
            let index_rm = rng.gen_range(0..spots_remaining.len());
            minefield.place_mine(spots_remaining.remove(index_rm));
        }

        minefield
    }

    fn place_mine(&mut self, index: usize) {
        assert!(index < self.field.len());
        
        // Only place a mine in an emty field
        if let SpotKind::Field(_) = self.field[index].kind {
            let y = (index as i32) / (self.width as i32);
            let x = (index as i32) % (self.width as i32);

            // place mine, and update neighboring spots
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let neighbor_x = x + dx;
                    let neighbor_y = y + dy;
    
                    if (dx == 0) && (dy == 0) {
                        // place the mine
                        self.field[index].kind = SpotKind::Mine;
                    } else if let Some(neighbor_index) = self.spot_index(neighbor_x, neighbor_y) {
                        if let SpotKind::Field(n) = &mut self.field[neighbor_index].kind {
                            // increment count of neighboring mines for this spot
                            *n += 1;
                        }
                    }
                    
                }
            }  
        }
    }
    
    fn spot_index(&self, x: i32, y: i32) -> Option<usize> {
        if (x >= 0) && (x < self.width as i32) && (y >= 0) && (y < self.height as i32) {
            Some((y as usize * self.width as usize) + x as usize)
        } else {
            None
        }
    }
}

 #[cfg(test)]
 mod tests {
     use super::*;
 
     #[test]
     fn new_minefield() {
        
        let mines = 2;
        let width = 2;
        let height = 2;
        let minefield = Minefield::new(mines, width, height);
        println!(" ... ");
        print_minefield(&minefield);

         let minefield = Minefield::new(3, 3, 6);
         println!(" ... ");
         print_minefield(&minefield);

         let minefield = Minefield::new(0, 0, 0);
         println!(" ... ");
         print_minefield(&minefield);         

         let minefield = Minefield::new(1, 0, 0);
         println!(" ... ");
         print_minefield(&minefield);    
         
         let minefield = Minefield::new(1, 1, 0);
         println!(" ... ");
         print_minefield(&minefield);     
         
         let minefield = Minefield::new(0, 1, 1);
         println!(" ... ");
         print_minefield(&minefield);

         let minefield = Minefield::new(1, 1, 1);
         println!(" ... ");
         print_minefield(&minefield);

         let minefield = Minefield::new(0, 100, 100);
         println!(" ... ");
         print_minefield(&minefield);      
         
         let minefield = Minefield::new(1, 100, 100);
         println!(" ... ");
         print_minefield(&minefield);
         
        // This crashes VSCode on my computer :)
        //  let minefield = Minefield::new(std::u16::MAX, std::u16::MAX, std::u16::MAX);

     }

     fn print_minefield(minefield: &Minefield) {
        for y in 0..minefield.height {
            print!("[");
            for x in 0..minefield.width {
                if let Some(index) = minefield.spot_index(x as i32, y as i32) {
                    match minefield.field[index].kind {
                        SpotKind::Mine => print!(" *"),
                        SpotKind::Field(n) => print!(" {}", n),
                    }
                }
            }
            println!(" ]");
        }
     }
 }