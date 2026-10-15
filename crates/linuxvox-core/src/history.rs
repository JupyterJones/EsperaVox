use crate::grid::VoxelGrid;
use crate::voxel::{VoxelCoord, VoxelId};

/// A single voxel change record for undo/redo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoxelDiff {
    pub coord: VoxelCoord,
    pub old_id: VoxelId,
    pub new_id: VoxelId,
}

/// A collection of voxel changes resulting from a single user action (e.g. brush stroke or fill).
#[derive(Debug, Clone)]
pub struct EditCommand {
    pub name: String,
    pub diffs: Vec<VoxelDiff>,
}

impl EditCommand {
    pub fn new(name: impl Into<String>, diffs: Vec<VoxelDiff>) -> Self {
        Self {
            name: name.into(),
            diffs,
        }
    }
}

/// Manages undo/redo stacks for spatial edits.
#[derive(Debug, Default)]
pub struct HistoryManager {
    undo_stack: Vec<EditCommand>,
    redo_stack: Vec<EditCommand>,
    max_history: usize,
}

impl HistoryManager {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    /// Records a new action and clears the redo stack.
    pub fn record(&mut self, cmd: EditCommand) {
        if cmd.diffs.is_empty() {
            return;
        }
        self.redo_stack.clear();
        if self.undo_stack.len() >= self.max_history && self.max_history > 0 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(cmd);
    }

    /// Reverts the most recent action.
    pub fn undo(&mut self, grid: &mut VoxelGrid) -> bool {
        if let Some(cmd) = self.undo_stack.pop() {
            for diff in &cmd.diffs {
                grid.set_coord(diff.coord, diff.old_id);
            }
            self.redo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    /// Re-applies the most recently reverted action.
    pub fn redo(&mut self, grid: &mut VoxelGrid) -> bool {
        if let Some(cmd) = self.redo_stack.pop() {
            for diff in &cmd.diffs {
                grid.set_coord(diff.coord, diff.new_id);
            }
            self.undo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}
