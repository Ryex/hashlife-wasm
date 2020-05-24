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
type NodeMap = HashMap<QTreeNode, Box<QTreeNode>>;

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

    pub fn new(alive: bool, map: Box<NodeMap>) -> BoxedNode {
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

    pub fn new_with_children(nw: BoxedNode, ne: BoxedNode, sw: BoxedNode, se: BoxedNode,  map: Box<NodeMap>) -> BoxedNode {
        let population = nw.population + ne.population +
                         sw.population + se.population;
        let level = nw.level + 1;
        QTreeNode {
            nw: Some(nw),
            ne: Some(ne),
            sw: Some(sw),
            se: Some(se),
            level,
            alive: population > 0,
            population,
            result: None,
            map
        }.intern()
    }

    pub fn new_empty_tree (level: u32, map: Box<NodeMap>) -> BoxedNode {
        if level == 0 {
            return QTreeNode::new(false, map);
        }

        let n = QTreeNode::new_empty_tree(level - 1, map.clone());
        QTreeNode::new_with_children(n.clone(), n.clone(), n.clone(), n, map)
    }

    fn intern(&mut self) -> BoxedNode {
        if let Some(cannon) = self.map.get(self) {
            return cannon.clone();
        }
        let boxed = Box::new(self.clone());
        self.map.insert(self.clone(), boxed.clone());
        boxed
    }

    pub fn get_bit(&self, x: i32, y: i32) -> i32 {
        if self.level == 0 {
            return self.alive as i32;
        }
        let offset = 1 << (self.level - 2) ;
        if x < 0 {
            if y < 0 {
                self.nw.clone().unwrap().get_bit(x+offset, y+offset)
            } else {
                self.sw.clone().unwrap().get_bit(x+offset, y-offset)
            }
        } else if y < 0 {
            self.ne.clone().unwrap().get_bit(x-offset, y+offset)
        } else {
            self.se.clone().unwrap().get_bit(x-offset, y-offset)
        }

    }

    pub fn set_bit(&self, x: i32, y: i32) -> BoxedNode {
        if self.level == 0 {
            return QTreeNode::new(true, self.map.clone());
        }

        // distance from center of this node to center of subnode is
        // one fourth the size of this node.
        let offset = 1 << (self.level - 2) ;
        if x < 0 {
            if y < 0 {
               QTreeNode::new_with_children(
                    self.nw.clone().unwrap().set_bit(x + offset, y + offset), 
                    self.ne.clone().unwrap(), 
                    self.sw.clone().unwrap(), 
                    self.se.clone().unwrap(),
                    self.map.clone())
            } else {
                QTreeNode::new_with_children(
                    self.nw.clone().unwrap(), 
                    self.ne.clone().unwrap(), 
                    self.sw.clone().unwrap().set_bit(x + offset, y - offset), 
                    self.se.clone().unwrap(),
                    self.map.clone())
            }
        } else if y < 0 {
            QTreeNode::new_with_children(
                self.nw.clone().unwrap(), 
                self.ne.clone().unwrap().set_bit(x - offset, y + offset), 
                self.sw.clone().unwrap(), 
                self.se.clone().unwrap(),
                self.map.clone())
        } else {
            QTreeNode::new_with_children(
                self.nw.clone().unwrap(), 
                self.ne.clone().unwrap(), 
                self.sw.clone().unwrap(), 
                self.se.clone().unwrap().set_bit(x - offset, y - offset),
                self.map.clone())
        }
     }


     pub fn one_generation(&self, bitmask: i32) -> BoxedNode {
        if bitmask == 0 {
            return QTreeNode::new(false, self.map.clone());
        }
        let s = (bitmask >> 5) & 1;
        let mut b = bitmask & 0x757 ; // mask out bits we don't care about
        let mut neighbor_count = 0 ;
        while b != 0 {
            neighbor_count += 1;
            b &= b - 1 ; // clear least significant bit
        }
        if neighbor_count == 3 || (neighbor_count == 2 && s != 0) {
            QTreeNode::new(true, self.map.clone())
        } else {
            QTreeNode::new(false, self.map.clone())
        }
     }

    //  TreeNode slowSimulation() {
    //     int allbits = 0 ;
    //     for (int y=-2; y<2; y++)
    //        for (int x=-2; x<2; x++)
    //           allbits = (allbits << 1) + getBit(x, y) ;
    //     return create(oneGen(allbits>>5), oneGen(allbits>>4),
    //                   oneGen(allbits>>1), oneGen(allbits)) ;
    //  }
    pub fn slow_simulaiton(&self) -> BoxedNode {
        let mut all_bits = 0;
        for y in -2..2 {
            for x in -2..2 {
                all_bits = (all_bits << 1) + self.get_bit(x, y) ;
            }
        }

        QTreeNode::new_with_children(
            self.one_generation(all_bits >> 5), 
            self.one_generation(all_bits >> 4), 
            self.one_generation(all_bits >> 1), 
            self.one_generation(all_bits),
            self.map.clone())

    }



}


impl Default for QTreeNode {
    
    fn default() -> Self {
        let map = Box::new(NodeMap::new());
        QTreeNode::new_empty_tree(3, map).as_ref().clone()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QTreeUniverse {
    root: QTreeNode,
    map: NodeMap,
    generation_count: u64,
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
