use rand::Rng;

#[derive(Clone, Debug)]
pub enum Spot {
    Mine,
    Field(i32),
}

pub struct Minefield {
    minefield: Vec<Vec<Spot>>,
    mines: usize,
    width: usize,
    height: usize
}

impl Minefield {
    pub fn new(mines: u16, width: u16, height: u16) -> Self {
        let mines = mines as usize;
        
        let width = width as usize;
        let height = height as usize;

        // Create empty field
        let mut minefield = vec![vec![Spot::Field(0); width]; height];

        let max_spots = width * height;
        let mines = if mines <= max_spots { mines } else { max_spots };

        let mut spots_remaining: Vec<usize> = (0..max_spots).collect();
        let mut rng = rand::thread_rng();

        for _ in 0..mines {
            let index_rm = rng.gen_range(0..spots_remaining.len());
            let index = spots_remaining.remove(index_rm);

            let x = index % width;
            let y = index / width;

            Minefield::add_mine(x, y, &mut minefield);
        }

        Self { minefield, mines, width, height }
    }

    fn add_mine(x: usize, y: usize, minefield: &mut Vec<Vec<Spot>>) {
        // Don't place anything in an empty 2d array
        if minefield.is_empty() || minefield.len() >= (std::i16::MAX as usize) {
            return
        } else if minefield[0].is_empty() || minefield[0].len() >= (std::i16::MAX as usize) {
            return
        } else if let Spot::Mine = minefield[y][x] {
            // Don't place a mine on top of another mine. The caller is responsible for ensuring this.
            return 
        } else {
            let y_max = (minefield.len() - 1) as i32;
            let x_max = (minefield[0].len() - 1) as i32;
            let x: i32 = x as i32;
            let y: i32 = y as i32;
    
            // update neighboring spots
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let ix = (x + dx) as usize;
                    let iy = (y + dy) as usize;
    
                    if (dx == 0) && (dy == 0) {
                        // place the mine
                        minefield[iy][ix] = Spot::Mine;
                    } else if (((x == 0) && (dx < 0)) || ((x == x_max) && (dx > 0))) ||
                              (((y == 0) && (dy < 0)) || ((y == y_max) && (dy > 0))) {
                        // spot is outside minefield, ignore
                    } else if let Spot::Field(n) = &mut minefield[iy][ix] {
                        // increment count of neighboring mines for this spot
                        *n += 1;
                    }
                    
                }
            }            
        }
    }
}

 #[cfg(test)]
 mod tests {
     use super::*;
 
     #[test]
     fn new_minefield() {
         let minefield = Minefield::new(3, 3, 6);
         println!(" ... ");
         print_minefield(&minefield.minefield);

         let minefield = Minefield::new(0, 0, 0);
         println!(" ... ");
         print_minefield(&minefield.minefield);         

         let minefield = Minefield::new(1, 0, 0);
         println!(" ... ");
         print_minefield(&minefield.minefield);    
         
         let minefield = Minefield::new(1, 1, 0);
         println!(" ... ");
         print_minefield(&minefield.minefield);     
         
         let minefield = Minefield::new(0, 1, 1);
         println!(" ... ");
         print_minefield(&minefield.minefield);

         let minefield = Minefield::new(1, 1, 1);
         println!(" ... ");
         print_minefield(&minefield.minefield);

         let minefield = Minefield::new(0, 100, 100);
         println!(" ... ");
         print_minefield(&minefield.minefield);      
         
         let minefield = Minefield::new(1, 100, 100);
         println!(" ... ");
         print_minefield(&minefield.minefield);
         
        // This crashes VSCode on my computer :)
        //  let minefield = Minefield::new(std::u16::MAX, std::u16::MAX, std::u16::MAX);

     }

     fn print_minefield(minefield: &Vec<Vec<Spot>>) {
        for row in minefield.iter() {
            print!("[");
            for s in row.iter() {
                match s {
                    Spot::Mine => print!(" *"),
                    Spot::Field(n) => print!(" {}", n),
                }
            }
            println!(" ]");
        }
     }
 }