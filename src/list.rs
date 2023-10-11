use std::hint::unreachable_unchecked;
use std::mem::MaybeUninit;
use std::num::NonZeroU8;
use std::ops::Deref;
use std::ptr;
use std::slice;

#[derive(Copy)]
/// A nonempty list that is ordered
pub struct BArrayVec<T, const N: usize> {
	len: NonZeroU8,
	buf: [MaybeUninit<T>; N],
}

impl<T: Default, const N: usize> Default for BArrayVec<T, N> {
	fn default() -> Self {
		BArrayVec::new(Default::default())
	}
}

impl<T: Clone, const N: usize> Clone for BArrayVec<T, N> {
	fn clone(&self) -> Self {
		let mut iter = self.iter().cloned();

		let first = iter.next().unwrap();

		let mut new = Self::new(first);

		for item in iter {
			unsafe { new._push(item).unwrap_unchecked() };
		}

		new
	}
}

impl<T: PartialEq, const N: usize> PartialEq for BArrayVec<T, N> {
	fn eq(&self, other: &Self) -> bool {
		self.len() == other.len() && self.iter().eq(other.iter())
	}
}

impl<T, const N: usize> Deref for BArrayVec<T, N> {
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
	}
}

const _: () = {
	use std::fmt::*;

	impl<T: Debug, const N: usize> Debug for BArrayVec<T, N> {
		fn fmt(&self, f: &mut Formatter) -> Result {
			let mut f = f.debug_list();

			f.entries(self.iter());

			f.finish()
		}
	}
};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum AbsoluteOrdering {
	Less,
	Greater,
}

impl<T, const N: usize> BArrayVec<T, N> {
	pub fn new(item: T) -> Self {
		assert!(N != 0 && N < (u8::MAX as usize));

		let mut this = Self {
			buf: unsafe { MaybeUninit::uninit().assume_init() },
			len: NonZeroU8::new(1).unwrap(),
		};

		let _ = this.buf[0].write(item);

		this
	}
	pub fn as_ptr(&self) -> *const T {
		self.buf.as_ptr().cast()
	}
	/// Unlike `slice::as_mut_ptr` this pointer can be used to initialize uninitialized items
	pub fn as_mut_ptr(&mut self) -> *mut T {
		self.buf.as_mut_ptr().cast()
	}
	pub fn len(&self) -> usize {
		self.len.get().into()
	}
	pub unsafe fn set_len(&mut self, new_len: usize) {
		self.len = NonZeroU8::new_unchecked(new_len as u8);
	}
	/// An infallible alias for `slice::first`
	pub fn min(&self) -> &T {
		unsafe { self.first().unwrap_unchecked() }
	}
	/// An infallible alias for `slice::last`
	pub fn max(&self) -> &T {
		unsafe { self.last().unwrap_unchecked() }
	}
	pub fn is_full(&self) -> bool {
		self.len() == N
	}
	// INTERNAL: push item at the end
	fn _push(&mut self, item: T) -> Result<(), T> {
		if !self.is_full() {
			let len = self.len();

			self.buf[len].write(item);

			unsafe { self.set_len(len + 1) };

			Ok(())
		} else {
			Err(item)
		}
	}
	// INTERNAL: push item at the front
	fn _push_front(&mut self, item: T) -> Result<(), T> {
		if self.is_full() {
			Err(item)
		} else {
			if let Some(_) = self._insert(0, item) {
				unsafe { unreachable_unchecked() }
			} else {
				Ok(())
			}
		}
	}
	// INTERNAL: insert item at index and shift items right possibly popping the last element
	fn _insert(&mut self, index: usize, item: T) -> Option<T> {
		if !(index < self.len()) {
			panic!(
				"insert overflows allocation (index is {} but length is {})",
				index, N
			);
		}

		let mut ret = None;

		unsafe {
			if self.is_full() {
				ret = Some(ptr::read(self.max()))
			} else {
				self.set_len(self.len() + 1)
			}
		}

		let ptr = unsafe { self.as_mut_ptr().add(index) };

		unsafe {
			ptr::copy(ptr, ptr.add(1), self.len() - index - 1);
			ptr::write(ptr, item);
		}
		// a, b, c, d, e
		// _insert(B, 1) => e
		// 0  1
		// a, B, b, c, d,

		ret
	}
}

impl<T, const N: usize> BArrayVec<T, N>
where
	T: Ord,
{
	/// Inserts `item` into list preserving ordering
	pub fn insert(&mut self, item: T) -> Result<Option<T>, (T, AbsoluteOrdering)> {
		if item > *self.max() {
			self._push(item)
				.map(|()| None)
				.map_err(|item| (item, AbsoluteOrdering::Greater))
		} else if item < *self.min() {
			self._push_front(item)
				.map(|()| None)
				.map_err(|item| (item, AbsoluteOrdering::Less))
		} else {
			let (Ok(index) | Err(index)) = self.binary_search(&item);

			Ok(self._insert(index, item))
		}
	}
	pub fn contains(&self, item: &T) -> Result<bool, AbsoluteOrdering> {
		self.find(item).map(|index| index.is_some())
	}
	pub fn find(&self, item: &T) -> Result<Option<usize>, AbsoluteOrdering> {
		if item > self.max() {
			Err(AbsoluteOrdering::Greater)
		} else if item < self.min() {
			Err(AbsoluteOrdering::Less)
		} else {
			if let Ok(index) = self.binary_search(item) {
				Ok(Some(index))
			} else {
				Ok(None)
			}
		}
	}
}

const _ASSERT_NULL_OPTIMIZED: () = {
	use std::mem::size_of;

	type L = BArrayVec<i32, 5>;

	if size_of::<L>() != size_of::<Option<L>>() {
		panic!("`List<T, N>` is not null optimized");
	}
};

#[test]
fn test_push() {
	let mut this = BArrayVec::<i32, 5>::new(100);

	for i in 101..105 {
		this._push(i).unwrap();
	}

	assert_eq!(&*this, &[100, 101, 102, 103, 104]);

	let old = this;

	assert_eq!(this._push(105), Err(105));
	assert_eq!(this, old);
}

#[test]
fn test_push_front() {
	let mut this = BArrayVec::<i32, 5>::new(100);

	for i in 101..105 {
		this._push_front(i).unwrap();
	}

	assert_eq!(&*this, &[104, 103, 102, 101, 100]);

	let old = this;

	assert_eq!(this._push_front(105), Err(105));
	assert_eq!(this, old);
}

#[test]
fn test_insert() {
	let mut this = BArrayVec::<i32, 5>::new(100);

	this.insert(-101).unwrap();
	this.insert(102).unwrap();
	this.insert(-102).unwrap();
	this.insert(103).unwrap();

	assert_eq!(&*this, &[-102, -101, 100, 102, 103]);

	assert_eq!(this.insert(101), Ok(Some(103)));

	assert_eq!(&*this, &[-102, -101, 100, 101, 102]);

	let old = this;

	assert_eq!(this.insert(-200), Err((-200, AbsoluteOrdering::Less)));
	assert_eq!(this.insert(200), Err((200, AbsoluteOrdering::Greater)));

	assert_eq!(this, old);
}
