//! The trees module.
//! This module contains:
//! * Traits that all implementations of trees should implement
//! * Specific implementations of trees
//!
//! The Walker trait implements walking through a tree. This includes dealing with the borrow
//! checking problems of recursive structures (using Telescope), and rebalancing the tree.
//! Therefore, walkers can't guarantee that the tree won't change as you walk through them.
//! 
//! Currently this module is limited to trees which are based on the BasicTree type.

pub mod basic_tree;
pub mod splay;

use crate::data::*;
pub trait SomeTree<A : Data> where
    for<'a> &'a mut Self : SomeTreeRef<A> {

    fn into_inner(self) -> basic_tree::BasicTree<A>;
    fn new() -> Self;
    fn from_inner(tree : basic_tree::BasicTree<A>) -> Self;

}

pub trait SomeTreeRef<A : Data> {
    type Walker : SomeWalker<A>;
    fn walker(self) -> Self::Walker;
}



/// The Walker trait implements walking through a tree.
/// This includes dealing with the borrow checking problems of recursive structures (using Telescope),
/// and rebalancing the tree.
/// Therefore, walkers can't guarantee that the tree won't change as you walk through them.
///
/// The walker should be able to walk on any of the existing nodes, or any empty position just near them.
/// i.e., The walker can also be in the position of a son of an existing node, where there isn't
/// a node yet.
/// The method is_empty() can tell whether you are at an empty position. Trying to move downward from an
/// empty position produces an error value.
pub trait SomeWalker<A : Data> : SomeEntry<A> {
    /// return `Err(())` if it is in an empty spot.
    fn go_left(&mut self) -> Result<(), ()>;
    /// returns `Err(())` if it is in an empty spot.
    fn go_right(&mut self) -> Result<(), ()>;

    /// If successful, returns whether or not the previous current value was the left son.
    /// If already at the root of the tree, returns `Err(())`.
    /// If you have a SplayTree, you shouldn't use this method too much, or you might lose the
    /// SplayTree's complexity properties - see documentation aboud splay tree.
    
    fn go_up(&mut self) -> Result<bool, ()>;

    /*
    fn segment_value(&self) -> A::Value {
        self.inner().segment_value()
    }
    */

    fn depth(&self) -> usize;

    /// Returns a summary of all the values to the left of this point,
    /// That are not children of this point.
    fn far_left_summary(&self) -> A::Summary;
    /// Returns a summary of all the values to the right of this point,
    /// That are not children of this point.
    fn far_right_summary(&self) -> A::Summary;

    /// Returns a summary of all the values to the left of this point.
    /// If the walker is in a non empty spot, this does not include the current node.
    fn left_summary(&self) -> A::Summary {
        let left = self.far_left_summary();
        if let basic_tree::BasicTree::Root(node) = self.inner() {
            left + node.left.subtree_summary()
        } else {
            left
        }
    }
    /// Returns a summary of all the values to the right of this point.
    /// If the walker is in a non empty spot, this does not include the current node.
    fn right_summary(&self) -> A::Summary {
        let right = self.far_right_summary();
        if let basic_tree::BasicTree::Root(node) = self.inner() {
            node.left.subtree_summary() + right
        } else {
            right
        }
    }
    

    // these functions are here instead of Deref and DerefMut. 
    fn inner_mut(&mut self) -> &mut basic_tree::BasicTree<A>;
    fn inner(&self) -> &basic_tree::BasicTree<A>;
}

/// Things that allow access to a maybe-missing value, as if it is an Option<A>.
/// Currently there are no actual Entry types, and the walkers themselves
/// act as the entries. However, the traits are still separated.
pub trait SomeEntry<A : Data> {
    fn value_mut(&mut self) -> Option<&mut A::Value>;
    fn value(&self) -> Option<&A::Value>;

    /// Returns a summary of just the current node.
    /// Returns the empty summary if at an empty position.
    fn node_summary(&self) -> A::Summary;
    

    fn is_empty(&self) -> bool {
        self.value().is_none()
    }

    // there is no point of running access() or rebuild() qafter writing,
    // because the node can't access any other part of the tree except itself,
    // so the user could have just done the calls himself,
    // ergo, the user can be required to provide a value of data
    // that doesn't need access() or rebuild() to be called

    /// only writes if it is in an empty position. if the position isn't empty,
    /// return Err(()).
    fn insert_new(&mut self, value : A::Value) -> Result<(), ()>;
}

