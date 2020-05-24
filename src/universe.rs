use std::hash::{Hash, Hasher};
use std::collections::HashMap;

use fixedbitset::FixedBitSet;

extern crate web_sys;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Debug, Clone, Eq)]
struct QTreeNode {
    nw: Option<BoxedNode>,
    ne: Option<BoxedNode>,
    sw: Option<BoxedNode>,
    se: Option<BoxedNode>,
    level: u32,
    population: u32,
    alive: bool,
    result: Option<BoxedNode>,
    map: Box<NodeMap>
}

type BoxedNode = Box<QTreeNode>;
type NodeMap = HashMap<QTreeNode, QTreeNode>;

impl Hash for QTreeNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.level == 0 {
            self.population.hash(state);
        } else {
            self.nw.hash(state);
            self.ne.hash(state);
            self.sw.hash(state);
            self.se.hash(state);
        }
        
    }
}

impl PartialEq for QTreeNode {
    fn eq(&self, other: &Self) -> bool {
        if self.level == 0 {
            if other.population == 0 {
                self.population == other.population
            } else {
                false
            }
        } else if other.level == 0 {
                false
        } else {
            self.nw == other.nw &&
            self.ne == other.ne &&
            self.se == other.se &&
            self.sw == other.sw
        }
    }
}

impl QTreeNode {

    fn new(alive: bool, map: Box<NodeMap>) -> Self {
        QTreeNode {
            nw: None,
            ne: None,
            sw: None,
            se: None,
            level: 0,
            alive,
            population: if alive { 1 } else { 0 },
            result: None,
            map,
        }.intern()
    }

    fn new_with_children(nw: BoxedNode, ne: BoxedNode, sw: BoxedNode, se: BoxedNode,  map: Box<NodeMap>) -> Self {
        let population = nw.population + ne.population +
                         sw.population + se.population;
        QTreeNode {
            nw: Some(nw.clone()),
            ne: Some(ne.clone()),
            sw: Some(sw.clone()),
            se: Some(se.clone()),
            level: nw.level + 1,
            alive: population > 0,
            population,
            result: None,
            map
        }.intern()
    }

    fn intern(&mut self) -> Self {
        if let Some(cannon) = self.map.get(self) {
            return cannon.clone();
        }
        self.map.insert(self.clone(), self.clone());
        self.clone()
    }

}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QTreeUniverse {
    root: QTreeNode,
    hashMap: NodeMap,
    generationCount: u64,
    width: u32,
    height: u32,
    population: u32
}

pub struct Universe {
    width: usize,
    height: usize,
    cells: FixedBitSet,
}

impl Default for Universe {
    fn default() -> Universe {
        Universe::new(64, 64)
    }
}

impl Universe {
    pub fn new(width: usize, height: usize) -> Universe {
        let cells = Universe::fill_cells_random(width, height);

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn get_index(&self, row: usize, column: usize) -> usize {
        row * self.width + column
    }

    // fast version
    fn live_neighbor_count(&self, row: usize, column: usize) -> usize {
        let mut count = 0;

        let north = if row == 0 { self.height - 1 } else { row - 1 };

        let south = if row == self.height - 1 { 0 } else { row + 1 };

        let west = if column == 0 {
            self.width - 1
        } else {
            column - 1
        };

        let east = if column == self.width - 1 {
            0
        } else {
            column + 1
        };

        let nw = self.get_index(north, west);
        count += self.cells[nw] as usize;

        let n = self.get_index(north, column);
        count += self.cells[n] as usize;

        let ne = self.get_index(north, east);
        count += self.cells[ne] as usize;

        let w = self.get_index(row, west);
        count += self.cells[w] as usize;

        let e = self.get_index(row, east);
        count += self.cells[e] as usize;

        let sw = self.get_index(south, west);
        count += self.cells[sw] as usize;

        let s = self.get_index(south, column);
        count += self.cells[s] as usize;

        let se = self.get_index(south, east);
        count += self.cells[se] as usize;

        count
    }

    pub fn fill_cells_random(width: usize, height: usize) -> FixedBitSet {
        let size = width * height;
        let mut cells = FixedBitSet::with_capacity(size);

        for i in 0..size {
            cells.set(i, js_sys::Math::random() < 0.5)
        }

        cells
    }

    pub fn get_cells(&self) -> &FixedBitSet {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.

    pub fn set_cells(&mut self, cells: &[(usize, usize)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, true);
        }
    }

    pub fn step(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                next.set(
                    idx,
                    match (cell, live_neighbors) {
                        // Rule 1: Any live cell with fewer than two live neighbours
                        // dies, as if caused by underpopulation.
                        (true, x) if x < 2 => false,
                        // Rule 2: Any live cell with two or three live neighbours
                        // lives on to the next generation.
                        (true, 2) | (true, 3) => true,
                        // Rule 3: Any live cell with more than three live
                        // neighbours dies, as if by overpopulation.
                        (true, x) if x > 3 => false,
                        // Rule 4: Any dead cell with exactly three live neighbours
                        // becomes a live cell, as if by reproduction.
                        (false, 3) => true,
                        // All other cells remain in the same state.
                        (otherwise, _) => otherwise,
                    },
                );
            }
        }

        self.cells = next;
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }

    //#[allow(dead_code)]
    pub fn randomize(&mut self) {
        let size = (self.width * self.height) as usize;

        for i in 0..size {
            self.cells.set(i, rand::random())
        }
            
    }

    pub fn clear(&mut self) {
        let size = (self.width * self.height) as usize;

        for i in 0..size {
            self.cells.set(i, false)
        }
    }

    pub fn reset(&mut self) {
        let size = (self.width * self.height) as usize;
        self.cells = FixedBitSet::with_capacity(size);
    }

    /// Set the width of the universe.
    ///
    /// Resets all cells to the dead state.

    pub fn set_width(&mut self, width: usize) {
        self.width = width;
        self.reset();
    }

    /// Set the height of the universe.
    ///
    /// Resets all cells to the dead state.

    pub fn set_height(&mut self, height: usize) {
        self.height = height;
        self.reset();
    }

    pub fn toggle_cell(&mut self, idx: usize) {
        let val = self.cells[idx];
        self.cells.set(idx, !val);
        let column = idx % self.width;
        let row = (idx - column) / idx;
        log!(
            "Toggled cell at: [{:?}, {:?}] {:?} -> {:?}",
            row,
            column,
            val,
            !val
        );
    }

    #[rustfmt::skip]
    pub fn set_flyer(&mut self, row: usize, col: usize) {
  
        let cells = [
            (row-1, col-1),
                            (row, col), (row, col+1),
            (row+1, col-1), (row+1, col)
        ];
        self.set_cells(&cells);
    }

    #[rustfmt::skip]
    pub fn set_pulsar(&mut self, r: usize, c: usize) {

        let cells = [
                    (r-4, c-6), (r-3, c-6), (r-2, c-6),                 (r+2, c-6), (r+3, c-6), (r+4, c-6),
            
            (r-6, c-4),                          (r-1, c-4),    (r+1, c-4),                             (r+6, c-4),
            (r-6, c-3),                          (r-1, c-3),    (r+1, c-3),                             (r+6, c-3),
            (r-6, c-2),                          (r-1, c-2),    (r+1, c-2),                             (r+6, c-2),
                    
                    (r-4, c-1), (r-3, c-1), (r-2, c-1),                 (r+2, c-1), (r+3, c-1), (r+4, c-1),

                    (r-4, c+1), (r-3, c+1), (r-2, c+1),                 (r+2, c+1), (r+3, c+1), (r+4, c+1),
            
            (r-6, c+2),                          (r-1, c+2),    (r+1, c+2),                             (r+6, c+2),
            (r-6, c+3),                          (r-1, c+3),    (r+1, c+3),                             (r+6, c+3),
            (r-6, c+4),                          (r-1, c+4),    (r+1, c+4),                             (r+6, c+4),

                    (r-4, c+6), (r-3, c+6), (r-2, c+6),                 (r+2, c+6), (r+3, c+6), (r+4, c+6),
        ];
        self.set_cells(&cells);
    }
}
