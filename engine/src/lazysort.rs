use arrayvec::ArrayVec;

pub struct LazySort<T: Clone, F: Fn(&T) -> R, R: Ord, const CAPACITY: usize> {
	collection: ArrayVec<T, CAPACITY>,
	sorted: usize,
	sort_by: F,
}

pub struct LazySortIter<T: Clone, F: Fn(&T) -> R, R: Ord, const CAPACITY: usize> {
	sorter: LazySort<T, F, R, CAPACITY>,
	index: usize,
}

impl<T: Clone, F: Fn(&T) -> R, R: Ord, const CAPACITY: usize> LazySort<T, F, R, CAPACITY> {
	pub fn new(collection: impl IntoIterator<Item = T>, sort_by: F) -> Self {
		Self {
			collection: collection.into_iter().collect(),
			sort_by,
			sorted: 0,
		}
	}

	pub fn is_empty(&self) -> bool {
		self.collection.is_empty()
	}
}

impl<T: Clone, F: Fn(&T) -> R, R: Ord, const CAPACITY: usize> LazySort<T, F, R, CAPACITY> {
	fn sort(&mut self, index: usize) {
		let mut min: Option<R> = None;
		let mut min_index = None;
		for i in index..self.collection.len() {
			if let Some(min) = &mut min {
				let res = (self.sort_by)(&self.collection[i]);
				if res < *min {
					*min = res;
					min_index = Some(i);
				}
			}
		}

		if let Some(min_index) = min_index {
			self.collection.swap(index, min_index);
		}
	}

	fn sort_between(&mut self, start: usize, end: usize) {
		for i in start..=end {
			self.sort(i);
		}
	}

	pub fn get(&mut self, index: usize) -> Option<&T> {
		if index >= self.sorted {
			self.sort_between(self.sorted, index)
		}

		self.collection.get(index)
	}
}

impl<T: Copy, F: Fn(&T) -> R, R: Ord, const CAPACITY: usize> IntoIterator
	for LazySort<T, F, R, CAPACITY>
{
	type IntoIter = LazySortIter<T, F, R, CAPACITY>;
	type Item = T;

	fn into_iter(self) -> Self::IntoIter {
		LazySortIter {
			sorter: self,
			index: 0,
		}
	}
}

impl<T: Copy, F: Fn(&T) -> R, R: Ord, const CAPACITY: usize> Iterator
	for LazySortIter<T, F, R, CAPACITY>
{
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		let r = self.sorter.get(self.index);
		self.index += 1;
		r.cloned()
	}
}
