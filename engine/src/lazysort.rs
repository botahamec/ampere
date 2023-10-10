#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LazySort<T, F: Fn(&T) -> R, R: Ord> {
	collection: Box<[T]>,
	sorted: usize,
	sort_by: F,
}

pub struct LazySortIter<T, F: Fn(&T) -> R, R: Ord> {
	sorter: LazySort<T, F, R>,
	index: usize,
}

impl<T, F: Fn(&T) -> R, R: Ord> LazySort<T, F, R> {
	pub fn new(collection: Box<[T]>, sort_by: F) -> Self {
		Self {
			collection,
			sort_by,
			sorted: 0,
		}
	}

	fn len(&self) -> usize {
		self.collection.len()
	}

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

impl<T: Copy, F: Fn(&T) -> R, R: Ord> IntoIterator for LazySort<T, F, R> {
	type IntoIter = LazySortIter<T, F, R>;
	type Item = T;

	fn into_iter(self) -> Self::IntoIter {
		LazySortIter {
			sorter: self,
			index: 0,
		}
	}
}

impl<T: Copy, F: Fn(&T) -> R, R: Ord> Iterator for LazySortIter<T, F, R> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		let r = self.sorter.get(self.index);
		self.index += 1;
		r.cloned()
	}
}
