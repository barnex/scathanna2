use super::internal::*;

/// The history keeps a timeline of changes
/// and a `head` pointer to where the current edits are recorded.
///
///         change 3: add block_cde, block_efg
/// HEAD ->	change 2: add block_abc
///         change 1: remove block_xyz
///         change 0: add block_xyz, remove block_hij
///
/// A change consists of one or more blocks added/removed
/// (the `Cursor ` allows to add multiple blocks in a single change).
///
/// It's the `Editor`'s responsibility to call `commit_change` after recording
/// a group of changes that should be undone as one (E.g. instantiating a complex cursor).
#[derive(Default)]
pub struct History {
	changes: Vec<Change>,
	head: usize,
}

#[derive(Default, Clone)]
pub struct Change {
	pub add: Set<Block>,
	pub remove: Set<Block>,
	is_redo: bool, // after redo, a change needs to be cleared before recording new changes
}

impl Change {
	fn clear(&mut self) {
		self.is_redo = false;
		self.add.clear();
		self.remove.clear();
	}

	fn record_add(&mut self, blk: Block) {
		self.clear_if_needed();
		self.add.insert(blk);
		self.remove.remove(&blk);
	}

	fn record_remove(&mut self, blk: Block) {
		self.clear_if_needed();
		self.remove.insert(blk);
		self.add.remove(&blk);
	}

	fn clear_if_needed(&mut self) {
		if self.is_redo {
			self.clear();
		}
	}

	fn is_empty(&self) -> bool {
		self.add.is_empty() && self.remove.is_empty()
	}
}

impl History {
	/// Mark the start of a new undo-able change (move HEAD forward).
	/// All recorded edits in the same change will be undone/redone as one.
	pub fn commit_change(&mut self) {
		if !self.head().is_empty() {
			self.head += 1;
			self.ensure_head();

			// overwrite previous change, if any (e.g. edit after undo)
			self.head().clear();
		}
	}

	/// Record that a block was added in the current change.
	pub fn record_add(&mut self, blk: Block) {
		self.head().record_add(blk);
	}

	/// Record that a block was removed in the current change.
	pub fn record_remove(&mut self, blk: &Block) {
		self.head().record_remove(blk.clone());
	}

	/// Move the HEAD back and return the changes that need to be applied
	/// to the editor state in order to undo.
	/// An empty change is returned in case we can't undo further.
	pub fn undo(&mut self) -> Change {
		if self.head > 0 {
			self.head -= 1
		}
		// after undo, we can either redo and go back to this change,
		// or we can edit in which case we need to clear the change first.
		self.head().is_redo = true;
		invert(self.head().clone())
	}

	/// Move the HEAD forward and return the changes that need to be applied
	/// to the editor state in order to redo.
	/// An empty change is returned in case we can't redo further.
	pub fn redo(&mut self) -> Change {
		if self.head < self.changes.len() - 1 {
			let change = self.changes[self.head].clone();
			self.head += 1;
			change
		} else {
			default()
		}
	}

	/// The current HEAD change, guaranteed to be valid.
	fn head(&mut self) -> &mut Change {
		self.ensure_head();
		&mut self.changes[self.head]
	}

	fn ensure_head(&mut self) {
		while self.changes.len() <= self.head {
			self.changes.push(default())
		}
	}
}

fn invert(c: Change) -> Change {
	Change { add: c.remove, remove: c.add, ..c }
}
