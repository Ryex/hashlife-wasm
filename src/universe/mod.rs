use std::collections::HashMap;

use std::ops::Deref;

extern crate rand;

#[cfg(feature = "no-wasm")]
extern crate rand_chacha;
#[cfg(feature = "no-wasm")]
use rand::{Rng, SeedableRng};

// use bitvec::prelude::*;

pub mod morton;
pub mod node;
pub mod rect;

use super::universe::node::{BitSpace, BitSpaceSlice, Node, NodeId, SubNode};

type NodeMap = HashMap<Node, NodeId>;

#[derive(Debug, Clone)]
pub struct Universe {
    width: usize,
    height: usize,
    root: NodeId,
    arena: Vec<Box<Node>>,
    node_map: NodeMap,
    empty_node_map: HashMap<(usize, usize), NodeId>,
    non_empty_node_map: HashMap<Vec<u8>, NodeId>,
    next_node_map: HashMap<NodeId, NodeId>,
    morton_space: morton::MortonSpace,
}

impl Universe {
    const MIN_NODE_WIDTH: usize = 4;
    const MIN_NODE_HEIGHT: usize = 4;

    pub fn new(width: usize, height: usize) -> Self {
        let w = if width % 2 != 0 { width + 1 } else { width };
        let h = if height % 2 != 0 { height + 1 } else { height };

        if width % 2 != 0 {
            width + 1
        } else {
            width
        };

        let mut universe = Universe {
            width: w,
            height: h,
            root: NodeId::new(0),
            arena: Vec::with_capacity(w),
            node_map: HashMap::new(),
            empty_node_map: HashMap::new(),
            non_empty_node_map: HashMap::new(),
            next_node_map: HashMap::new(),
            morton_space: morton::MortonSpace::new(width, height),
        };

        let root = universe.node(width, height);
        universe.root = root;

        universe
    }

    fn canonicalize(&mut self, node: Box<Node>) -> NodeId {
        if let Some(canon) = self.node_map.get(&node) {
            return *canon;
        }
        let next_index = self.arena.len();
        let id = NodeId::new(next_index);
        self.node_map.insert(node.deref().clone(), id);
        self.arena.push(node);
        id
    }

    pub fn node(&mut self, width: usize, height: usize) -> NodeId {
        let key = (width, height);
        if let Some(node_id) = self.empty_node_map.get(&key) {
            *node_id
        } else if width <= Self::MIN_NODE_WIDTH || height <= Self::MIN_NODE_HEIGHT {
            let node_id = self.canonicalize(Box::new(Node::new(width, height)));
            self.empty_node_map.insert(key, node_id);
            node_id
        } else {
            let children = Box::new(SubNode::new(
                self.node(width / 2, height / 2),
                self.node(width / 2, height / 2),
                self.node(width / 2, height / 2),
                self.node(width / 2, height / 2),
            ));
            let pop = self.get_population_children(&children);
            let level = self.get_level_children(&children) + 1;
            let node_id = self.canonicalize(Box::new(Node::with_children(
                width, height, children, pop, level,
            )));
            self.empty_node_map.insert(key, node_id);
            node_id
        }
    }

    pub fn node_with_bits(&mut self, width: usize, height: usize, space: &BitSpaceSlice) -> NodeId {
        let key: Vec<u8> = space.to_bitvec().as_raw_slice().to_vec();
        if let Some(node_id) = self.non_empty_node_map.get(&key) {
            *node_id
        } else if width <= Self::MIN_NODE_WIDTH || height <= Self::MIN_NODE_HEIGHT {
            let node_id = self.canonicalize(Box::new(Node::with_bits(width, height, space)));
            self.non_empty_node_map.insert(key, node_id);
            node_id
        } else {
            let (w2, h2) = (width / 2, height / 2);
            let sw = w2 * h2;
            let children = Box::new(SubNode::new(
                self.node_with_bits(w2, h2, &space[0..sw]),
                self.node_with_bits(w2, h2, &space[sw..(sw * 2)]),
                self.node_with_bits(w2, h2, &space[(sw * 2)..(sw * 3)]),
                self.node_with_bits(w2, h2, &space[(sw * 3)..]),
            ));
            let pop = self.get_population_children(&children);
            let level = self.get_level_children(&children) + 1;
            let node_id = self.canonicalize(Box::new(Node::with_children(
                width, height, children, pop, level,
            )));
            self.non_empty_node_map.insert(key, node_id);
            node_id
        }
    }

    pub fn node_with_children(
        &mut self,
        width: usize,
        height: usize,
        nw: NodeId,
        ne: NodeId,
        sw: NodeId,
        se: NodeId,
    ) -> NodeId {
        let children = Box::new(SubNode::new(nw, ne, sw, se));
        let pop = self.get_population_children(&children);
        let level = self.get_level_children(&children) + 1;
        self.canonicalize(Box::new(Node::with_children(
            width, height, children, pop, level,
        )))
    }

    #[inline]
    pub fn get_node(&self, id: NodeId) -> &Node {
        self.arena
            .get(id.index())
            .expect("NodeId to be valid")
            .deref()
    }

    pub fn get_population(&self, id: NodeId) -> usize {
        self.get_node(id).population()
    }

    pub fn get_population_children(&self, children: &SubNode) -> usize {
        let mut pop: usize = 0;
        pop += self.get_node(children.nw()).population();
        pop += self.get_node(children.ne()).population();
        pop += self.get_node(children.sw()).population();
        pop += self.get_node(children.se()).population();
        pop
    }

    pub fn get_level(&self, id: NodeId) -> usize {
        self.get_node(id).level()
    }

    pub fn get_level_children(&self, children: &SubNode) -> usize {
        self.get_node(children.nw()).level()
    }

    pub fn get_morton(&self, row: usize, col: usize) -> usize {
        self.morton_space.morton2(row, col)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn set_width(&mut self, width: usize) {
        self.width = width;
        self.reset();
    }

    pub fn set_height(&mut self, height: usize) {
        self.height = height;
        self.reset();
    }

    pub fn fill_cells_random(&mut self) {
        let mut space: BitSpace = BitSpace::with_capacity(self.width * self.height);

        #[cfg(feature = "no-wasm")]
        {
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(10);
            for _ in 0..(self.width * self.height) {
                space.push(rng.gen());
            }
        }

        #[cfg(not(feature = "no-wasm"))]
        {
            for _ in 0..(self.width * self.height) {
                space.push(rand::random());
            }
        }

        self.root = self.node_with_bits(self.width, self.height, &space);
    }

    pub fn get_cells(&self) -> BitSpace {
        // #[cfg(not(feature = "no-wasm"))]
        // let _timer = Timer::new("Universe::get_cells");

        // let mut out: BitSpace = BitSpace::with_capacity(self.width * self.height);

        // self.build_bitspace_from_node(self.root, &mut out);
        self.build_bitspace_fast(self.root)

        // out
    }

    pub fn build_bitspace_from_node(&self, id: NodeId, space_out: &mut BitSpace) {
        // #[cfg(not(feature = "no-wasm"))]
        // let _timer = Timer::new("Universe::build_bitspace_from_node");
        let node = self.get_node(id);

        if let Some(children) = node.children() {
            self.build_bitspace_from_node((*children).nw(), space_out);
            self.build_bitspace_from_node(children.deref().ne(), space_out);
            self.build_bitspace_from_node((*children).sw(), space_out);
            self.build_bitspace_from_node((*children).se(), space_out);
        } else {
            space_out.extend(node.space().into_iter());
        }
    }

    pub fn build_bitspace_from_node_fast(&self, id: NodeId, ele_out: &mut Vec<u8>) {
        // #[cfg(not(feature = "no-wasm"))]
        // let _timer = Timer::new("Universe::build_bitspace_from_node");
        let node = self.get_node(id);

        if let Some(children) = node.children() {
            self.build_bitspace_from_node_fast((*children).nw(), ele_out);
            self.build_bitspace_from_node_fast(children.deref().ne(), ele_out);
            self.build_bitspace_from_node_fast((*children).sw(), ele_out);
            self.build_bitspace_from_node_fast((*children).se(), ele_out);
        } else {
            ele_out.extend(node.space().as_raw_slice());
        }
    }

    pub fn build_bitspace_fast(&self, id: NodeId) -> BitSpace {
        let mut elems: Vec<u8> = vec![];
        self.build_bitspace_from_node_fast(id, &mut elems);

        BitSpace::from_vec(elems)
    }

    pub fn set_cells(&mut self, cells: &[(usize, usize)]) {
        let mut space: BitSpace = BitSpace::with_capacity(self.width * self.height);
        self.build_bitspace_from_node(self.root, &mut space);

        for (row, col) in cells.iter().cloned() {
            let idx = self.get_morton(row, col);
            space.set(idx, true);
        }

        self.root = self.node_with_bits(self.width, self.height, &space);
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Result<bool, &'static str> {
        self.get_cell_node(row, col, self.root)
    }

    fn get_cell_node(&self, row: usize, col: usize, id: NodeId) -> Result<bool, &'static str> {
        let node = self.get_node(id);

        if let Some(children) = node.children() {
            let pivot_w = node.rect().width() / 2;
            let pivot_h = node.rect().height() / 2;
            if row < pivot_w {
                if col < pivot_h {
                    self.get_cell_node(row, col, children.nw())
                } else {
                    self.get_cell_node(row, col % pivot_h, children.sw())
                }
            } else if col < pivot_h {
                self.get_cell_node(row % pivot_w, col, children.deref().ne())
            } else {
                self.get_cell_node(row % pivot_w, col % pivot_h, children.se())
            }
        } else {
            node.get_cell(row, col)
        }
    }

    pub fn toggle_cell(&mut self, row: usize, col: usize) {
        let mut space: BitSpace = BitSpace::with_capacity(self.width * self.height);
        self.build_bitspace_from_node(self.root, &mut space);

        let val = self
            .get_cell(row, col)
            .expect("rol and col to be valid for a node");
        let idx = self.get_morton(row, col);
        space.set(idx, !val);

        self.root = self.node_with_bits(self.width, self.height, &space);
    }

    pub fn randomize(&mut self) {
        self.fill_cells_random();
    }

    pub fn clear(&mut self) {
        self.root = self.node(self.width, self.height);
    }

    pub fn reset(&mut self) {
        self.clear();
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
                     (r-4, c-6), (r-3, c-6), (r-2, c-6),             (r+2, c-6), (r+3, c-6), (r+4, c-6),

            (r-6, c-4),                          (r-1, c-4),    (r+1, c-4),                           (r+6, c-4),
            (r-6, c-3),                          (r-1, c-3),    (r+1, c-3),                           (r+6, c-3),
            (r-6, c-2),                          (r-1, c-2),    (r+1, c-2),                           (r+6, c-2),
                     (r-4, c-1), (r-3, c-1), (r-2, c-1),             (r+2, c-1), (r+3, c-1), (r+4, c-1),

                     (r-4, c+1), (r-3, c+1), (r-2, c+1),             (r+2, c+1), (r+3, c+1), (r+4, c+1),
            (r-6, c+2),                          (r-1, c+2),    (r+1, c+2),                           (r+6, c+2),
            (r-6, c+3),                          (r-1, c+3),    (r+1, c+3),                           (r+6, c+3),
            (r-6, c+4),                          (r-1, c+4),    (r+1, c+4),                           (r+6, c+4),

                     (r-4, c+6), (r-3, c+6), (r-2, c+6),             (r+2, c+6), (r+3, c+6), (r+4, c+6),
        ];
        self.set_cells(&cells);
    }

    #[allow(dead_code)]
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

        count += self.get_cell(north, west).expect("valid cell") as usize;
        count += self.get_cell(north, column).expect("valid cell") as usize;
        count += self.get_cell(north, east).expect("valid cell") as usize;
        count += self.get_cell(row, west).expect("valid cell") as usize;
        count += self.get_cell(row, east).expect("valid cell") as usize;
        count += self.get_cell(south, west).expect("valid cell") as usize;
        count += self.get_cell(south, column).expect("valid cell") as usize;
        count += self.get_cell(south, east).expect("valid cell") as usize;

        count
    }

    fn live_neighbor_count_fast(&mut self, row: usize, column: usize, space: &BitSpaceSlice) -> u8 {
        let mut count = 0;
        let ms = &mut self.morton_space;

        let north = row - 1;
        let south = row + 1;
        let west = column - 1;
        let east = column + 1;

        count += space[ms.morton2_cache(north, west)] as u8;
        count += space[ms.morton2_cache(north, column)] as u8;
        count += space[ms.morton2_cache(north, east)] as u8;
        count += space[ms.morton2_cache(row, west)] as u8;
        count += space[ms.morton2_cache(row, east)] as u8;
        count += space[ms.morton2_cache(south, west)] as u8;
        count += space[ms.morton2_cache(south, column)] as u8;
        count += space[ms.morton2_cache(south, east)] as u8;

        count
    }

    pub fn expand_and_wrap(&mut self, id: NodeId) -> NodeId {
        let root = self.get_node(id);
        let (w, h) = (self.width, self.height);
        let children = root.children().clone().expect("root to have children");

        let br = self.node(w / 2, h / 2);

        let (nw, ne, sw, se) = (
            children.nw(),
            children.deref().ne(),
            children.sw(),
            children.se(),
        );

        let nw_ex = self.node_with_children(w, h, br, sw, ne, nw);
        let ne_ex = self.node_with_children(w, h, se, br, ne, nw);
        let sw_ex = self.node_with_children(w, h, se, sw, br, nw);
        let se_ex = self.node_with_children(w, h, se, sw, ne, br);

        self.node_with_children(w * 2, h * 2, nw_ex, ne_ex, sw_ex, se_ex)
    }

    pub fn expand(&mut self, id: NodeId) -> NodeId {
        let root = self.get_node(id).clone();
        let br = self.node(self.width / 2, self.height / 2);

        let children = root.children().clone().expect("root to have children");
        let (nw, ne, sw, se) = (
            children.nw(),
            children.deref().ne(),
            children.sw(),
            children.se(),
        );
        let (w, h) = (self.width, self.height);

        let nw_ex = self.node_with_children(w, h, br, br, br, nw);
        let ne_ex = self.node_with_children(w, h, br, br, ne, br);
        let sw_ex = self.node_with_children(w, h, br, sw, br, br);
        let se_ex = self.node_with_children(w, h, se, br, br, br);

        self.node_with_children(w * 2, h * 2, nw_ex, ne_ex, sw_ex, se_ex)
    }

    pub fn step(&mut self) {
        let mut root_level = self.get_node(self.root).level();
        let mut root_id = self.root;

        root_id = self.expand_and_wrap(root_id);

        // do extra expansions to make sure we have enough space
        let mut exp = 0;
        while root_level < 3 {
            root_id = self.expand(root_id);
            root_level = self.get_node(root_id).level();
            exp += 1;
        }

        // step node and shrink one size down
        root_id = self.step_node(root_id);

        // unwrap any extra expansions
        for _ in 0..exp {
            root_id = self.centered_subnode(root_id);
        }

        self.root = root_id;
    }

    pub fn step_node(&mut self, id: NodeId) -> NodeId {
        // return early if we know the result of this node
        if let Some(next) = self.next_node_map.get(&id) {
            return *next;
        }

        let node = self.get_node(id);

        let population = node.population();
        let level = node.level();
        let (width, height) = (node.rect().width(), node.rect().height());
        // if the population of this region is 0 then obiously we don't need to simulate it will still be 0
        let next = if population == 0 {
            node.children().clone().expect("node to have children").nw()
        }
        // if the population is < 3 we know that the population will be zero next round
        else if population < 3 {
            self.node(width / 2, height / 2)
        } else if level == 2 {
            // #[cfg(not(feature = "no-wasm"))]
            // let _timer = Timer::new("slow simulation");

            self.slow_sim(id)
        } else {
            // #[cfg(not(feature = "no-wasm"))]
            // let _timer = Timer::new("building subnodes");

            let (w, h) = (width / 2, height / 2);
            let ch = node.children().clone().expect("node to have children");

            let n00 = self.centered_subnode(ch.nw());
            let n01 = self.centered_horizontal(ch.nw(), ch.deref().ne());
            let n02 = self.centered_subnode(ch.deref().ne());
            let n10 = self.centered_vertical(ch.nw(), ch.sw());
            let n11 = self.centered_sub_subnode(id);
            let n12 = self.centered_vertical(ch.deref().ne(), ch.se());
            let n20 = self.centered_subnode(ch.sw());
            let n21 = self.centered_horizontal(ch.sw(), ch.se());
            let n22 = self.centered_subnode(ch.se());

            let nw_pre = self.node_with_children(w, h, n00, n01, n10, n11);
            let ne_pre = self.node_with_children(w, h, n01, n02, n11, n12);
            let sw_pre = self.node_with_children(w, h, n10, n11, n20, n21);
            let se_pre = self.node_with_children(w, h, n11, n12, n21, n22);

            let nw = self.step_node(nw_pre);
            let ne = self.step_node(ne_pre);
            let sw = self.step_node(sw_pre);
            let se = self.step_node(se_pre);

            self.node_with_children(w, h, nw, ne, sw, se)
        };

        self.next_node_map.insert(id, next);

        next
    }

    pub fn slow_sim(&mut self, id: NodeId) -> NodeId {
        // let _timer = Timer::new("Universe::slow_sim");
        let node = self.get_node(id);
        let (w, h) = (node.rect().width(), node.rect().height());
        let (w2, h2) = (w / 2, h / 2);
        let (w22, h22) = (w2 / 2, h2 / 2);

        // let mut space: BitSpace = BitSpace::with_capacity(w * h);
        // self.build_bitspace_from_node(id, &mut space);

        let space = self.build_bitspace_fast(id);

        let mut next: BitSpace = BitSpace::repeat(false, w2 * h2);

        let range: Vec<_> = (0..(w2 * h2)).collect();

        let slice = next.as_raw_mut_slice();
        for (cur, chunk) in range.chunks(8).enumerate() {
            let mut ele = 0u8;
            let mut shifts: u8 = 0;
            for index in chunk {
                let (x, y) = morton::unravel_point(*index);
                let s_index = morton::morton2(x + w22, y + h22);
                let count = self.live_neighbor_count_fast(x + w22, y + h22, &space);
                let alive = count == 3 || (count == 2 && space[s_index]);
                ele = (ele << 1) | (alive as u8);
                shifts += 1;
            }
            while shifts < 8 {
                ele <<= 1;
                shifts += 1;
            }
            slice[cur] = ele;
        }

        self.node_with_bits(w / 2, h / 2, &next)
    }

    fn get_children(&mut self, id: NodeId) -> Box<SubNode> {
        let node = self.get_node(id);
        node.children().clone().expect("node to have children")
    }

    fn centered_subnode(&mut self, id: NodeId) -> NodeId {
        let node = self.get_node(id);

        let (w, h) = (node.rect().width() / 2, node.rect().height() / 2);
        let ch = node.children().clone().expect("node to have children");

        let nw = self.get_children(ch.nw()).se();
        let ne = self.get_children(ch.deref().ne()).sw();
        let sw = self.get_children(ch.sw()).deref().ne();
        let se = self.get_children(ch.se()).nw();

        self.node_with_children(w, h, nw, ne, sw, se)
    }

    fn centered_horizontal(&mut self, w: NodeId, e: NodeId) -> NodeId {
        let w_node = self.get_node(w);
        let e_node = self.get_node(e);

        // assert_eq!(w_node.rect().width(), e_node.rect().width());
        // assert_eq!(w_node.rect().height(), e_node.rect().height());

        let (w, h) = (w_node.rect().width() / 2, w_node.rect().height() / 2);
        let w_ch = w_node.children().clone().expect("node to have children");
        let e_ch = e_node.children().clone().expect("node to have children");

        let nw = self.get_children(w_ch.deref().ne()).se();
        let ne = self.get_children(e_ch.nw()).sw();
        let sw = self.get_children(w_ch.se()).deref().ne();
        let se = self.get_children(e_ch.sw()).nw();

        self.node_with_children(w, h, nw, ne, sw, se)
    }

    fn centered_vertical(&mut self, n: NodeId, s: NodeId) -> NodeId {
        let n_node = self.get_node(n);
        let s_node = self.get_node(s);

        // assert_eq!(n_node.rect().width(), s_node.rect().width());
        // assert_eq!(n_node.rect().height(), s_node.rect().height());

        let (w, h) = (n_node.rect().width() / 2, n_node.rect().height() / 2);
        let n_ch = n_node.children().clone().expect("node to have children");
        let s_ch = s_node.children().clone().expect("node to have children");

        let nw = self.get_children(n_ch.sw()).se();
        let ne = self.get_children(n_ch.se()).sw();
        let sw = self.get_children(s_ch.nw()).deref().ne();
        let se = self.get_children(s_ch.deref().ne()).nw();

        self.node_with_children(w, h, nw, ne, sw, se)
    }

    fn centered_sub_subnode(&mut self, id: NodeId) -> NodeId {
        let node = self.get_node(id);

        let (w, h) = (node.rect().width() / 2 / 2, node.rect().height() / 2 / 2);
        let ch = node.children().clone().expect("node to have children");

        let nw_c = self.get_children(ch.nw()).se();
        let ne_c = self.get_children(ch.deref().ne()).sw();
        let sw_c = self.get_children(ch.sw()).deref().ne();
        let se_c = self.get_children(ch.se()).nw();

        let nw = self.get_children(nw_c).se();
        let ne = self.get_children(ne_c).sw();
        let sw = self.get_children(sw_c).deref().ne();
        let se = self.get_children(se_c).nw();

        self.node_with_children(w, h, nw, ne, sw, se)
    }
}

impl Default for Universe {
    fn default() -> Self {
        Universe::new(64, 64)
    }
}

extern crate web_sys;
use web_sys::console;

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.name);
    }
}
