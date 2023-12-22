use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};

pub struct StackVec<T, const CAPACITY: usize> {
	values: [MaybeUninit<T>; CAPACITY],
	len: usize,
}

impl<T, const CAPACITY: usize> Deref for StackVec<T, CAPACITY> {
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		self.as_slice()
	}
}

impl<T, const CAPACITY: usize> DerefMut for StackVec<T, CAPACITY> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.as_mut_slice()
	}
}

impl<T, const CAPACITY: usize> FromIterator<T> for StackVec<T, CAPACITY> {
	fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
		let mut this = Self::new();
		for item in iter {
			this.push(item);
		}

		this
	}
}

impl<T, const CAPACITY: usize> StackVec<T, CAPACITY> {
	pub fn new() -> Self {
		Self {
			values: MaybeUninit::uninit_array(),
			len: 0,
		}
	}

	pub fn capcity(&self) -> usize {
		CAPACITY
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	pub fn as_slice(&self) -> &[T] {
		// safety: the first `len` elements are guaranteed to be initialized
		unsafe { MaybeUninit::slice_assume_init_ref(&self.values[..self.len]) }
	}

	pub fn as_mut_slice(&mut self) -> &mut [T] {
		// safety: the first `len` elements are guaranteed to be initialized
		unsafe { MaybeUninit::slice_assume_init_mut(&mut self.values[..self.len]) }
	}

	pub fn as_ptr(&self) -> *const T {
		self.values.as_ptr().cast()
	}

	pub fn as_mut_ptr(&mut self) -> *mut T {
		self.values.as_mut_ptr().cast()
	}

	pub fn try_push(&mut self, value: T) -> Option<()> {
		self.values.get_mut(self.len)?.write(value);
		self.len += 1;
		Some(())
	}

	pub fn push(&mut self, value: T) {
		self.values[self.len].write(value);
		self.len += 1;
	}

	pub fn pop(&mut self) -> Option<T> {
		if self.is_empty() {
			return None;
		}

		// safety: this value will no longer be used, and the value is valid
		//         because it appears in the valid part of the array
		unsafe {
			self.len -= 1;
			Some(std::ptr::read(self.as_ptr().add(self.len())))
		}
	}

	pub fn clear(&mut self) {
		self.len = 0;
	}
}
