use crate::locators;

use super::*;
use super::basic_tree::*;

/// The type that is used for bookkeeping.
/// `u8` is definitely enough, since the rank of the tree is logarithmic in the tree size.
type T = u8;
/// Used for rank differences
type TD = i8;

pub struct AVLTree<D : Data> {
    tree : BasicTree<D, T>,
}

/// For implementing `rank`, `rank_diff` and `rebuild_ranks` for
/// trees, nodes and walkers alike.
trait Rankable {
	fn rank(&self) -> T;

	/// Returns `true` if the rank of the current node had to be updated,
	/// `false` if it was correct.
	fn rebuild_ranks(&mut self) -> bool;

	/// Returns `right.rank() - left.rank()`
	fn rank_diff(&self) -> TD;
}

impl<D : Data> Rankable for BasicTree<D, T> {
	fn rank(&self) -> T {
		match self.node() {
			None => 0,
			Some(node) => node.rank(),
		}
    }
	
	fn rebuild_ranks(&mut self) -> bool {
		if let Some(node) = self.node_mut() {
			node.rebuild_ranks()
		}
		else {
			true
		}
	}

	/// Returns `right.rank() - left.rank()`
	fn rank_diff(&self) -> TD {
		match self.node() {
			None => 0,
			Some(node) => node.rank_diff(),
		}
	}
}


impl<D : Data> Rankable for BasicNode<D, T> {
	fn rank(&self) -> T {
		*self.alg_data()
    }

	/// Returns `right.rank() - left.rank()`
	fn rank_diff(&self) -> TD {
		self.right.rank() as TD - self.left.rank() as TD
	}

	fn rebuild_ranks(&mut self) -> bool {
		let new_rank = std::cmp::max(self.left.rank(), self.right.rank()) + 1;
		let changed = self.rank() != new_rank;
		self.alg_data = new_rank;
		changed
	}
}

impl<D : Data> AVLTree<D> {
    pub fn new() -> Self {
        AVLTree { tree : BasicTree::Empty }
    }

    fn rank(&self) -> T {
		self.tree.rank()
    }

	/// Returns `right.rank() - left.rank()`
	fn rank_diff(&self) -> TD {
		self.tree.rank_diff()
	}

    // TODO: fix
    /// Iterates over the whole tree.
	///```
	/// use orchard::avl::*;
	/// use orchard::example_data::StdNum;
	///
	/// let mut tree : AVLTree<StdNum> = (17..=89).collect();
	///
	/// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (17..=89).collect::<Vec<_>>());
	/// # tree.assert_correctness();
	///```
	pub fn iter(&mut self) -> impl Iterator<Item=&D::Value> {
		self.tree.iter()
	}

    // TODO: fix
    /// Iterates over the given segment.
	///```
	/// use orchard::avl::*;
	/// use orchard::example_data::StdNum;
	/// use orchard::methods;
	///
	/// let mut tree : AVLTree<StdNum> = (20..80).collect();
	/// let segment_iter = tree.iter_segment(3..13);
	///
	/// assert_eq!(segment_iter.cloned().collect::<Vec<_>>(), (23..33).collect::<Vec<_>>());
	/// # tree.assert_correctness();
	///```
	pub fn iter_segment<L>(&mut self, loc : L) -> impl Iterator<Item=&D::Value> where
    	L : locators::Locator<D>
    {
        self.tree.iter_segment(loc)
    }

	fn rebuild(&mut self) {
		self.tree.rebuild();
		self.tree.rebuild_ranks();
	}

	/// Checks that the tree is well formed.
	/// Panics otherwise.
	pub fn assert_correctness(&self) where D::Summary : Eq {
		self.tree.assert_correctness(); // TODO: remove
		Self::assert_correctness_internal(&self.tree);
	}

	fn assert_correctness_internal(tree : &BasicTree<D, T>) where D::Summary : Eq {
		if let Some(node) = tree.node() {
			Self::assert_ranks_locally_internal(&node);
			//node.assert_correctness_locally();
			Self::assert_correctness_internal(&node.left);
			Self::assert_correctness_internal(&node.right);
		}
	}

	pub fn assert_correctness_locally(&self) where D::Summary : Eq {
		if let Some(node) = self.tree.node() {
			Self::assert_ranks_locally_internal(&node);
			//node.assert_correctness_locally();
		}
	}

	pub fn assert_ranks_locally(&self) {
		if let Some(node) = self.tree.node() {
			Self::assert_ranks_locally_internal(&node);
		}
	}

	fn assert_ranks_locally_internal(node : &BasicNode<D, T>) {
		assert!(node.rank() == node.left.rank() + 1 || node.rank() == node.right.rank() + 1);
		assert!(node.left.rank() == node.rank() - 1 || node.left.rank() == node.rank() - 2);
		assert!(node.right.rank() == node.rank() - 1 || node.right.rank() == node.rank() - 2);
	}
}

impl<D : Data> Default for AVLTree<D> {
    fn default() -> Self {
        AVLTree::new()
    }
}


impl<D : Data> SomeTree<D> for AVLTree<D> {
    fn segment_summary<L>(&mut self, locator : L) -> D::Summary where
        L : crate::Locator<D> {
            methods::segment_summary(self, locator)
    }

    fn act_segment<L>(&mut self, action : D::Action, locator : L) where
        L : crate::Locator<D>
    {
        if action.to_reverse() == false {
            methods::act_segment(self, action, locator)
        } else {
            todo!();
        }
    }
}


impl<'a, D : Data> SomeTreeRef<D> for &'a mut AVLTree<D> {
    type Walker = AVLWalker<'a, D>;

    fn walker(self) -> Self::Walker {
        AVLWalker { walker : self.tree.walker() }
    }
}


impl<'a, D : Data> ModifiableTreeRef<D> for &'a mut AVLTree<D> {
    type ModifiableWalker = AVLWalker<'a, D>;
}

impl<D : Data> SomeEntry<D> for AVLTree<D> {
    fn with_value<F, R>(&mut self, f : F) -> Option<R> where 
        F : FnOnce(&mut D::Value) -> R {
        self.tree.with_value(f)
    }

    fn node_summary(&self) -> D::Summary {
        self.tree.node_summary()
    }

    fn subtree_summary(&self) -> D::Summary {
        self.tree.subtree_summary()
    }

    fn left_subtree_summary(&self) -> Option<D::Summary> {
        self.tree.left_subtree_summary()
    }

    fn right_subtree_summary(&self) -> Option<D::Summary> {
        self.tree.right_subtree_summary()
    }

    fn act_subtree(&mut self, action : D::Action) {
        self.tree.act_subtree(action);
    }

    fn act_node(&mut self, action : D::Action) -> Option<()> {
        self.tree.act_node(action)
    }

    fn act_left_subtree(&mut self, action : D::Action) -> Option<()> {
        self.tree.act_left_subtree(action)
    }

    fn act_right_subtree(&mut self, action : D::Action) -> Option<()> {
        self.tree.act_right_subtree(action)
    }
}

impl<D : Data> std::iter::FromIterator<D::Value> for AVLTree<D> {
    /// This takes [`O(n)`] worst-case time.
    fn from_iter<T: IntoIterator<Item = D::Value>>(iter: T) -> Self {
		// TODO: check if inserting is O(1) amortized. if it is, we can do this by
		// just calling insert.
        
		let mut tree : AVLTree<D> = Default::default();
		let mut walker = tree.walker();
		for val in iter.into_iter() {
			// note: this relies on the assumption, that after we insert a node, the new position of the locator
			// will be an ancestor of the location where the value was inserted.
			// TODO: check.
			while let Ok(_) = walker.go_right()
				{}
			walker.insert(val);
		}
		drop(walker);
		tree
    }
}

impl<D : Data> IntoIterator for AVLTree<D> {
    type Item = D::Value;
    type IntoIter = iterators::OwningIterator<D, std::ops::RangeFull, T>;

    fn into_iter(self) -> Self::IntoIter {
        iterators::OwningIterator::new(self.tree, ..)
    }
}



pub struct AVLWalker<'a, D : Data> {
	walker : BasicWalker<'a, D, T>,
}


impl<'a, D : Data> SomeWalker<D> for AVLWalker<'a, D> {
    fn go_left(&mut self) -> Result<(), ()> {
        self.walker.go_left()
    }

    fn go_right(&mut self) -> Result<(), ()> {
        self.walker.go_right()
    }

    fn go_up(&mut self) -> Result<bool, ()> {
        let res = self.walker.go_up()?;
		self.inner_mut().rebuild_ranks();
		Ok(res)
    }

    fn depth(&self) -> usize {
        self.walker.depth()
    }

    fn far_left_summary(&self) -> D::Summary {
        self.walker.far_left_summary()
    }

    fn far_right_summary(&self) -> D::Summary {
        self.walker.far_left_summary()
    }

    fn value(&self) -> Option<&D::Value> {
        self.walker.value()
    }
}

impl<'a, D : Data> SomeEntry<D> for AVLWalker<'a, D> {
    fn with_value<F, R>(&mut self, f : F) -> Option<R> where 
        F : FnOnce(&mut D::Value) -> R {
        self.walker.with_value(f)
    }

    fn node_summary(&self) -> D::Summary {
        self.walker.node_summary()
    }

    fn subtree_summary(&self) -> D::Summary {
        self.walker.subtree_summary()
    }

    fn left_subtree_summary(&self) -> Option<D::Summary> {
        self.walker.left_subtree_summary()
    }

    fn right_subtree_summary(&self) -> Option<D::Summary> {
        self.walker.right_subtree_summary()
    }

    fn act_subtree(&mut self, action : D::Action) {
        self.walker.act_subtree(action);
    }

    fn act_node(&mut self, action : D::Action) -> Option<()> {
        self.walker.act_node(action)
    }

    fn act_left_subtree(&mut self, action : D::Action) -> Option<()> {
        self.walker.act_left_subtree(action)
    }

    fn act_right_subtree(&mut self, action : D::Action) -> Option<()> {
        self.walker.act_right_subtree(action)
    }
}

impl<'a, D : Data> Rankable for AVLWalker<'a, D> {
	/// Returns the priority of the current node. Lower numbers means 
    /// The node is closer to the root.
    fn rank(&self) -> T {
        match self.walker.node() {
			None => 0,
			Some(node) => *node.alg_data(),
		}
    }

	/// Returns `right.rank() - left.rank()`
	fn rank_diff(&self) -> TD {
		self.walker.inner().rank_diff()
	}
	
	fn rebuild_ranks(&mut self) -> bool {
		self.inner_mut().rebuild_ranks()
	}
}

impl<'a, D : Data> AVLWalker<'a, D> {
    

	fn inner(&self) -> &BasicTree<D, T> {
        self.walker.inner()
    }

    fn inner_mut(&mut self) -> &mut BasicTree<D, T> {
        self.walker.inner_mut()
    }

	fn rot_left(&mut self) -> Option<()> {
		let rebuilder = | node : &mut BasicNode<D, T> | {
			node.rebuild_ranks();
		};
		self.walker.rot_left_with_custom_rebuilder(rebuilder)
	}

	fn rot_right(&mut self) -> Option<()> {
		let rebuilder = | node : &mut BasicNode<D, T> | {
			node.rebuild_ranks();
		};
		self.walker.rot_right_with_custom_rebuilder(rebuilder)
	}

	fn rot_up(&mut self) -> Result<bool, ()> {
		let rebuilder = | node : &mut BasicNode<D, T> | {
			node.rebuild_ranks();
		};
		self.walker.rot_up_with_custom_rebuilder(rebuilder)
	}

	fn rot_side(&mut self, b : bool) -> Option<()> {
		let rebuilder = | node : &mut BasicNode<D, T> | {
			node.rebuild_ranks();
		};
		self.walker.rot_side_with_custom_rebuilder(b, rebuilder)
	}

	/// This function gets called when a node is deleted or inserted,
	/// at the current position.
	fn rebalance(&mut self) {
		if self.is_empty() { return; }

		loop {
			let node = self.inner().node().unwrap();
			match self.rank_diff() {
				-2 => { // -2, left is deeper
					if node.left.rank_diff() <= 0 { // right right case
						self.rot_right().unwrap();
					} else { // left.rank() = 1, right left case
						self.go_left().unwrap();
						self.rot_left().unwrap(); // TODO
						let res = self.walker.rot_up();
						assert!(res == Ok(true));
					}
				},

				-1..=1 => {}, // do nothing, the current node is now balanced.

				2 => { // 2, left is shallower
					if node.right.rank_diff() >= 0 { // right right case
						self.rot_left().unwrap();
					} else { // right.rank() = -1, right left case
						self.go_right().unwrap();
						self.rot_right().unwrap();
						let res = self.rot_up();
						assert!(res == Ok(false));
					}
				},

				rd => panic!("illegal rank difference: {}", rd)
			}

			// current node has been balanced. now go up a node,
			// and check if we need to confinue rebalancing.
			let res = self.walker.go_up();
			let changed = self.inner_mut().rebuild_ranks();
			if !changed { // tree is now balanced correctly
				break;
			}
			if res.is_err() { // reached root
				break;
			}
		}
	}

	fn rebuild(&mut self) {
		self.inner_mut().rebuild();
		self.rebuild_ranks();
	}
}

impl<'a, D : Data> ModifiableWalker<D> for AVLWalker<'a, D> {
	/// Inserts the value into the tree at the current empty position.
    /// If the current position is not empty, return [`None`].
    /// When the function returns, the walker will be at a position which is an ancestor of the
	/// newly inserted node.
    fn insert(&mut self, val : D::Value) -> Option<()> {
        self.walker.insert_with_alg_data(val, 1 /* rank of a node with no sons */)?;
		let _ = self.go_up();
		self.rebalance();
		Some(())
    }

	// TODO: specify where the walker will be.
    fn delete(&mut self) -> Option<D::Value> {
		// the delete implementation is copied from `BasicTree`,
        // in order for rebalancing to be done properly.
        let tree = self.walker.take_subtree();
		let mut node = tree.into_node()?;
		if node.right.is_empty() {
			self.walker.put_subtree(node.left).unwrap();
			self.rebalance();
		} else { // find the next node and move it to the current position
			let mut walker = node.right.walker();
			while let Ok(_) = walker.go_left()
				{}
			let _ = walker.go_up();

			let tree2 = walker.take_subtree();

			let mut boxed_replacement_node = tree2.into_node_boxed().unwrap();
			assert!(boxed_replacement_node.left.is_empty());
			walker.put_subtree(boxed_replacement_node.right).unwrap();
			AVLWalker { walker : walker }.rebalance(); // rebalance here

			boxed_replacement_node.left = node.left;
			boxed_replacement_node.right = node.right;
			boxed_replacement_node.rebuild();
			boxed_replacement_node.rebuild_ranks();
			self.walker.put_subtree(BasicTree::Root(boxed_replacement_node)).unwrap();
			self.go_right().unwrap();
			self.rebalance(); // rebalance here
		}
		Some(node.node_value)
    }
}


#[test]
fn avl_delete() {
    let arr : Vec<_> =(0..500).collect();
	for i in 0..arr.len() {
		let mut tree : AVLTree<example_data::StdNum> = arr.iter().cloned().collect();
		let mut walker = methods::search(&mut tree, (i,));
		assert_eq!(walker.value().cloned(), Some(arr[i]));
		let res = walker.delete();
		assert_eq!(res, Some(arr[i]));
		drop(walker);
		assert_eq!(tree.into_iter().collect::<Vec<_>>(),
			arr[..i].iter()
			.chain(arr[i+1..].iter())
			.cloned().collect::<Vec<_>>());
	}
}


#[test]
fn avl_insert() {
    let arr : Vec<_> =(0..500).collect();
	for i in 0 ..= arr.len() {
		let new_val = 13;
		let mut tree : AVLTree<example_data::StdNum> = arr.iter().cloned().collect();
		let mut walker = methods::search(&mut tree, i..i);
		walker.insert(new_val);
		// after inserting, the walker can move, because of rebalancing.
		// however, in avl trees, the walker should be in an ancestor of the inserted value.
		// therefore, we check with `search_subtree`.
		methods::search_subtree(&mut walker, (i,));
		assert_eq!(walker.value().cloned(), Some(new_val));
		drop(walker);
		tree.assert_correctness();
		assert_eq!(tree.into_iter().collect::<Vec<_>>(),
			arr[..i].iter()
			.chain([new_val].iter())
			.chain(arr[i..].iter())
			.cloned().collect::<Vec<_>>());
	}
}