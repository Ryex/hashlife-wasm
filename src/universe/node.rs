// use fixedbitset::FixedBitSet;

use bitvec::prelude as bv;

use super::rect::Rectangle;

use std::fmt::Debug;

use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct NodeId {
    index: usize,
}

impl NodeId {
    pub fn new(index: usize) -> Self {
        NodeId { index }
    }

    pub fn index(self) -> usize {
        self.index
    }
}

#[derive(Debug, Clone, Copy, Eq)]
pub struct SubNode {
    nw: NodeId,
    ne: NodeId,
    sw: NodeId,
    se: NodeId,
}

impl SubNode {
    pub fn nw(&self) -> NodeId {
        self.nw
    }

    pub fn ne(&self) -> NodeId {
        self.ne
    }

    pub fn sw(&self) -> NodeId {
        self.sw
    }

    pub fn se(&self) -> NodeId {
        self.se
    }

    pub fn new(nw: NodeId, ne: NodeId, sw: NodeId, se: NodeId) -> Self {
        SubNode { nw, ne, sw, se }
    }

}

impl Hash for SubNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nw.index.hash(state);
        self.ne.index.hash(state);
        self.sw.index.hash(state);
        self.se.index.hash(state);
    }
}

impl PartialEq for SubNode {
    fn eq(&self, other: &Self) -> bool {
        self.nw.index == other.nw.index &&
        self.ne.index == other.ne.index &&
        self.sw.index == other.sw.index &&
        self.se.index == other.se.index
    }
}

pub type BitSpace = bv::BitVec<bv::Msb0, u8>;
pub type BitSpaceSlice = bv::BitSlice<bv::Msb0, u8>;

#[derive(Debug, Clone, Eq)]
pub struct Node {
    rect: Rectangle,
    population: usize,
    level: usize,
    space: Option<Box<BitSpace>>,
    children: Option<Box<SubNode>>,
}

impl Node {
    pub fn new(width: usize, height: usize) -> Self {
        Node {
            rect: Rectangle::new(width, height),
            population: 0,
            level: 0,
            space: Some(Box::new(bv::BitVec::repeat(false, width * height))),
            children: None,
        }
    }

    pub fn with_bits(width: usize, height: usize, space: &BitSpaceSlice) -> Self {
        Node {
            rect: Rectangle::new(width, height),
            population: space.count_ones(),
            level: 0,
            space: Some(Box::new(bv::BitVec::from_bitslice(space))),
            children: None,
        }
    }

    pub fn with_children(
        width: usize,
        height: usize,
        children: Box<SubNode>,
        population: usize,
        level: usize,
    ) -> Self {
        Node {
            rect: Rectangle::new(width, height),
            population,
            level,
            space: None,
            children: Some(children),
        }
    }

    pub fn rect(&self) -> &Rectangle {
        &self.rect
    }

    pub fn children(&self) -> &Option<Box<SubNode>> {
        &self.children
    }

    pub fn space(&self) -> Box<BitSpace> {
        self.space.clone().expect("node to have a bit space")
    }

    pub fn population(&self) -> usize {
        self.population
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Result<bool, &'static str> {
        if self.has_space() {
            let space = self.space();
            if row >= self.rect.width() {
                Err("row out of range for width")
            } else if col >= self.rect.height() {
                Err("col out of range for height")
            } else {
                Ok(space[row * self.rect.width() + col])
            }
        } else {
            Err("Node doesn't have a bit space! ask a child.")
        }
    }

    pub fn has_space(&self) -> bool {
        self.space.is_some()
    }

    pub fn has_children(&self) -> bool {
        self.children.is_some()
    }

}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(children) = &self.children {
            children.hash(state);
        } else if let Some(space) = &self.space {
            space.hash(state);
        } else {
            self.rect.hash(state);
            self.population.hash(state);
            self.level.hash(state);
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        if let Some(sch) = &self.children {
            if let Some(och) = &other.children {
                sch == och
            } else {
                false
            }
        } else if let Some(space) = &self.space {
            if let Some(ospace) = &other.space {
                space == ospace
            } else {
                false
            }
        } else {
            self.rect == other.rect &&
            self.population == other.population &&
            self.level == other.level
        }

    }
}

impl Default for Node {
    fn default() -> Self {
        Node {
            rect: Rectangle::new(2, 2),
            population: 0,
            level: 0,
            space: Some(Box::new(bv::BitVec::repeat(false, 4))),
            children: None,
        }
    }
}
