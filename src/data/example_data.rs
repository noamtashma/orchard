//! A module for examples of possible instantiations for [`Data::Value`], [`Data::Summary`],
//! [`Data::Action`] and [`Data`] itself.
//!
//! Hopefully also some useful common ones.
//!
//! For example, [`Unit`] for instantiations without  actions or without summaries.

use super::*;
use std::marker::PhantomData;

/// Used for cases where no action or no summary is needed.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Default, PartialOrd, Ord)]
pub struct Unit {}

impl<V> Acts<V> for Unit {
    fn act_inplace(&self, _ref: &mut V) {}
}

impl Add for Unit {
    type Output = Unit;
    fn add(self, _b: Unit) -> Unit {
        Unit {}
    }
}

impl Action for Unit {
    fn is_identity(self) -> bool {
        self == Default::default()
    }
}

impl<V> FromSingletonValue<V> for Unit {
    fn to_summary(_value: &V) -> Self {
        Unit {}
    }
}

/// Storing the size of a subtree.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Size {
    /// The size of a subtree
    pub size: usize,
}

impl Add for Size {
    type Output = Size;
    fn add(self, b: Size) -> Size {
        Size {
            size: self.size + b.size,
        }
    }
}

impl Default for Size {
    fn default() -> Size {
        Size { size: 0 }
    }
}

impl SizedSummary for Size {
    fn size(self) -> usize {
        self.size
    }
}

impl<V> FromSingletonValue<V> for Size {
    fn to_summary(_value: &V) -> Self {
        Size { size: 1 }
    }
}

/// [`Data`] instance for plain values with segment size information, so that they can be accessed.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
struct SizeData<V> {
    phantom: PhantomData<V>,
}

impl<V> Data for SizeData<V> {
    type Action = Unit;
    type Summary = Size;
    type Value = V;
}

/// A trait for summary instances which keep track of the size of segments.
pub trait SizedSummary {
    /// The size of the segment
    fn size(self) -> usize;
}

/// A [`Data`] instance for straight values.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct NoAction<V> {
    phantom: PhantomData<V>,
}

impl<V: Eq + Copy> Data for NoAction<V> {
    type Summary = Unit;
    type Action = Unit;
    type Value = V;

    fn to_summary(_val: &Self::Value) -> Self::Summary {
        Unit {}
    }
}

/// Actions that either reverses a segment or keeps it as it is
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct RevAction {
    /// Whether to reverse the segment
    pub to_reverse: bool,
}

impl std::ops::Add for RevAction {
    type Output = RevAction;
    fn add(self, b: RevAction) -> RevAction {
        RevAction {
            to_reverse: self.to_reverse != b.to_reverse,
        }
    }
}

impl Default for RevAction {
    fn default() -> Self {
        RevAction { to_reverse: false }
    }
}

impl Action for RevAction {
    fn is_identity(self) -> bool {
        self == Default::default()
    }

    fn to_reverse(self) -> bool {
        self.to_reverse
    }
}

impl Acts<Size> for RevAction {
    fn act_inplace(&self, _val: &mut Size) {}
}

type I = i32;
/// A standard numerical summary
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct NumSummary {
    /// The maximum of all values in the segment. [`None`] is the segment is empty.
    pub max: Option<I>,
    /// The minimum of all values in the segment. [`None`] is the segment is empty.
    pub min: Option<I>,
    /// The size of the segment.
    pub size: I,
    /// The sum of all values in the segment.
    pub sum: I,
}

impl Add for NumSummary {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        NumSummary {
            max: match (self.max, other.max) {
                (Some(a), Some(b)) => Some(std::cmp::max(a, b)),
                (Some(a), _) => Some(a),
                (_, b) => b,
            },
            min: match (self.min, other.min) {
                (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
                (Some(a), _) => Some(a),
                (_, b) => b,
            },
            size: self.size + other.size,
            sum: self.sum + other.sum,
        }
    }
}

impl Default for NumSummary {
    fn default() -> NumSummary {
        NumSummary {
            max: None,
            min: None,
            size: 0,
            sum: 0,
        }
    }
}

impl SizedSummary for NumSummary {
    fn size(self) -> usize {
        self.size as usize
    }
}

impl FromSingletonValue<I> for NumSummary {
    fn to_summary(val: &I) -> Self {
        NumSummary {
            max: Some(*val),
            min: Some(*val),
            size: 1,
            sum: *val,
        }
    }
}

/// Actions of reversals and adding a constant
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct RevAddAction {
    /// whether to reverse the segment.
    pub to_reverse: bool,
    /// A constant to add to all the values in the segment.
    pub add: I,
}

impl Add for RevAddAction {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        RevAddAction {
            to_reverse: self.to_reverse != other.to_reverse,
            add: self.add + other.add,
        }
    }
}

impl Default for RevAddAction {
    fn default() -> Self {
        RevAddAction {
            to_reverse: false,
            add: 0,
        }
    }
}

impl Action for RevAddAction {
    fn is_identity(self) -> bool {
        self == Default::default()
    }

    fn to_reverse(self) -> bool {
        self.to_reverse
    }
}

impl Acts<I> for RevAddAction {
    fn act_inplace(&self, val: &mut I) {
        *val += self.add;
    }
}

impl Acts<NumSummary> for RevAddAction {
    fn act_inplace(&self, summary: &mut NumSummary) {
        summary.max = summary.max.map(|max: I| max + self.add);
        summary.min = summary.min.map(|max: I| max + self.add);
        summary.sum += self.add * summary.size;
    }
}

/// Actions of reversals, adding a constant, and multiplying by a constant.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct RevAffineAction {
    /// Whether to reverse the segment.
    pub to_reverse: bool,
    /// A constant to multiply all the values in the segment with.
    pub mul: I,
    /// A constant to add to all the values in the segment.
    pub add: I,
}

impl Action for RevAffineAction {
    fn is_identity(self) -> bool {
        self == Default::default()
    }

    fn to_reverse(self) -> bool {
        self.to_reverse
    }
}

impl Add for RevAffineAction {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        RevAffineAction {
            to_reverse: self.to_reverse ^ other.to_reverse,
            mul: self.mul * other.mul,
            add: self.add + self.mul * other.add,
        }
    }
}

impl Default for RevAffineAction {
    fn default() -> Self {
        RevAffineAction {
            to_reverse: false,
            mul: 1,
            add: 0,
        }
    }
}

impl Acts<I> for RevAffineAction {
    fn act_inplace(&self, val: &mut I) {
        *val *= self.mul;
        *val += self.add;
    }
}

impl Acts<NumSummary> for RevAffineAction {
    fn act_inplace(&self, summary: &mut NumSummary) {
        if self.mul < 0 {
            std::mem::swap(&mut summary.min, &mut summary.max);
        }
        summary.max = summary.max.map(|max: I| max * self.mul);
        summary.min = summary.min.map(|max: I| max * self.mul);
        summary.sum *= self.mul;

        summary.max = summary.max.map(|max: I| max + self.add);
        summary.min = summary.min.map(|max: I| max + self.add);
        summary.sum += self.add * summary.size;
    }
}

/// A Data marker for a standard set of summaries and actions used for numbers. Specifically,
/// one can reverse or add a constant to a whole segment at once, and one can query
/// the maximum, minimum, size and sum of a whole segment at once.
pub struct StdNum {}

impl Data for StdNum {
    type Value = I;
    type Summary = NumSummary;
    type Action = RevAffineAction;
}


/// A summary type that can sum up applications of polynomials.
/// That is, for a segment that has value a_i, is can compute
/// (for example) \sum_i a_i*P(i) for any polynomial P of degree at most D-1.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct PolyNum<const D: usize> {
    // Contains the amount of elements in this segment
    size: usize,
    // Contains the moments of the segment.
    // The k'th moment is the sum of i^k*a_i for i in 0..size.
    // In other words, it's the same as `self.apply_poly(x^k)`.
    moments: [I; D],
}

/// Shifts a polynomial by some shift.
/// That is, we have `ResultPoly(x) = InputPoly(x+shift)` for all x.
fn shift_poly<const D: usize>(shift: I, poly: &[I; D]) -> [I; D] {
    // Represents powers of (x+shift)
    let mut powers = [0; D];
    powers[0] = 1;

    let mut result = [0; D];

    for deg in 0..D {
        // add poly[deg] * (x+shift)^deg to the result
        for i in 0..D {
            result[i] += poly[deg] * powers[i];
        }

        if deg >= D-1 {
            break // skip multiplying the polynomial by (x+shift) one too many times
        }

        // multiply `powers` by (x+shift)
        for i in (0..=deg).rev() {
            powers[i+1] += powers[i];
            powers[i] *= shift;
        }
    }

    result
}

impl<const D: usize> PolyNum<D> {
    /// Sums up the polynomial on these values, starting with index 0.
    /// That is, if this represents a segment with values `a_0, ..., a_k`,
    /// this returns `P(0)*a_0 + P(1)*a_1 + ... P(k)a_k`.
    pub fn apply_poly(&self, poly: &[I; D]) -> I {
        let mut result = 0;
        for i in 0..D {
            result += poly[i] * self.moments[i];
        }
        result
    }

    /// Sums up the polynomial on these values, starting with index `index`.
    /// That is, if this represents a segment with values a_0, ..., a_k,
    /// this returns `P(index)*a_0 + P(index+1)*a_1 + ... P(k)a_k`.
    pub fn apply_poly_index(&self, index: I, poly: &[I; D]) -> I {
        let shifted_poly = shift_poly(index, poly);
        self.apply_poly(&shifted_poly)
    }
}

impl<const D: usize> Default for PolyNum<D> {
    fn default() -> Self {
        PolyNum {
            size: 0,
            moments: [0; D]
        }
    }
}

impl<const D: usize> Add for PolyNum<D> {
    type Output = PolyNum<D>;

    fn add(self, rhs: Self) -> Self::Output {
        let mut moments: [I; D] = [0; D];
        let mut power = [0; D];
        for i in 0..D {
            power[i] = 1;
            moments[i] = self.moments[i] + rhs.apply_poly_index(self.size as I, &power);
            power[i] = 0;
        }
        
        PolyNum {
            size: self.size + rhs.size,
            moments
        }
    }
}

impl<const D: usize> FromSingletonValue<I> for PolyNum<D> {
    fn to_summary(value: &I) -> Self {
        let mut moments = [0; D];
        if D > 0 {
            moments[0] = *value;
        }
        PolyNum {
            size: 1,
            moments,
        }
    }
}

// TODO: consider retiring this and just requiring Value: Ord instead.

/// A trait for values that are keyed.
/// For example, when storing integers in sorted order, use the `Ordered`
/// struct, and now you can use binary search to find specific elements /
/// specify the edges of the segments you want to act upon.
///
/// Smaller values go on the left.
pub trait Keyed {
    /// The key type that elements are ordered by.
    type Key: std::cmp::Ord;

    // TODO: is it possible to switch to `impl Borrow<Self::Key> + '_` or something similar?
    /// Gets the key associated with a value
    fn get_key(&self) -> &Self::Key;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
/// A wrapper that wraps around values that should be stored in sorted order.
/// Implements `Keyed` with the key being the value itself.
pub struct Ordered<T>(pub T);

impl<T: Ord> Keyed for Ordered<T> {
    type Key = T;
    fn get_key(&self) -> &Self::Key {
        &self.0
    }
}
