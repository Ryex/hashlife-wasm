// use fixedbitset::FixedBitSet;

use bitvec::prelude as bv;

use super::rect::Rectangle;

use std::fmt::Debug;
use std::hash::Hash;

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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

pub type BitSpace = bv::BitVec<bv::Msb0, u8>;
pub type BitSpaceSlice = bv::BitSlice<bv::Msb0, u8>;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Node {
    rect: Rectangle,
    population: usize,
    level: usize,
    space: Option<BitSpace>,
    children: Option<SubNode>,
}

impl Node {
    pub fn new(width: usize, height: usize) -> Self {
        Node {
            rect: Rectangle::new(width, height),
            population: 0,
            level: 0,
            space: Some(bv::BitVec::repeat(false, width * height)),
            children: None,
        }
    }

    pub fn with_bits(width: usize, height: usize, space: &BitSpaceSlice) -> Self {
        Node {
            rect: Rectangle::new(width, height),
            population: space.count_ones(),
            level: 0,
            space: Some(bv::BitVec::from_bitslice(space)),
            children: None,
        }
    }

    pub fn with_children(
        width: usize,
        height: usize,
        children: SubNode,
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

    pub fn children(&self) -> &Option<SubNode> {
        &self.children
    }

    pub fn space(&self) -> &Option<BitSpace> {
        &self.space
    }

    pub fn population(&self) -> usize {
        self.population
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Result<bool, &'static str> {
        if let Some(space) = self.space() {
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

    pub fn has_children(&self) -> bool {
        self.children.is_some()
    }
}

impl Default for Node {
    fn default() -> Self {
        Node {
            rect: Rectangle::new(2, 2),
            population: 0,
            level: 0,
            space: Some(bv::BitVec::repeat(false, 4)),
            children: None,
        }
    }
}
