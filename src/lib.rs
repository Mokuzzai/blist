mod list;

pub use list::AbsoluteOrdering;
pub use list::List;

pub struct Node<T, const N: usize> {
	items: List<T, N>,

	next: Option<Box<Self>>,
}

const _: () = {
	use std::fmt::*;

	impl<T: Debug, const N: usize> Debug for Node<T, N> {
		fn fmt(&self, f: &mut Formatter) -> Result {
			let mut f = f.debug_struct("Node");

			f.field("items", &self.items);

			if let Some(next) = self.next.as_deref() {
				f.field("next", next);
			} else {
				f.field("next", &[] as &[T; 0]);
			}

			f.finish()
		}
	}

	impl<T: Debug, const N: usize> Debug for LinkedLists<T, N> {
		fn fmt(&self, f: &mut Formatter) -> Result {
			let mut next = self.root.as_ref();

			let mut f = f.debug_list();

			while let Some(node) = next {
				f.entry(&node.items);

				next = node.next.as_deref()
			}

			f.finish()
		}
	}
};

impl<T, const N: usize> Node<T, N> {
	pub fn new(item: T) -> Self {
		Self {
			items: List::new(item),

			next: None,
		}
	}
}

impl<T, const N: usize> Node<T, N>
where
	T: Ord,
{
	pub fn insert(&mut self, item: T) {
		match self.items.insert(item) {
			Err((item, AbsoluteOrdering::Less)) => {
				let mut node = Node::new(item);

				std::mem::swap(self, &mut node);

				self.next = Some(Box::new(node))
			}
			Err((item, AbsoluteOrdering::Greater)) | Ok(Some(item)) => {
				if let Some(next) = self.next.as_deref_mut() {
					next.insert(item);
				} else {
					self.next = Some(Box::new(Node::new(item)))
				}
			}
			Ok(None) => (),
		}
	}
	pub fn find(&self, item: &T) -> Option<usize> {
		match self.items.find(item) {
			Ok(index) => index,
			Err(AbsoluteOrdering::Greater) => self.next.as_ref()?.find(item),
			Err(AbsoluteOrdering::Less) => None,
		}
	}
	pub fn contains(&self, item: &T) -> bool {
		self.find(item).is_some()
	}
}

pub struct LinkedLists<T, const N: usize> {
	root: Option<Node<T, N>>,

	len: usize,
}

impl<T, const N: usize> LinkedLists<T, N> {
	pub const fn new() -> Self {
		Self { root: None, len: 0 }
	}
	pub const fn len(&self) -> usize {
		self.len
	}
}

impl<T, const N: usize> LinkedLists<T, N>
where
	T: Ord,
{
	pub fn insert(&mut self, item: T) {
		if let Some(root) = self.root.as_mut() {
			root.insert(item)
		} else {
			self.root = Some(Node::new(item));
		}

		self.len += 1;
	}
	pub fn find(&self, item: &T) -> Option<usize> {
		self.root.as_ref()?.find(item)
	}
	pub fn contains(&self, item: &T) -> bool {
		self.find(item).is_some()
	}
}
