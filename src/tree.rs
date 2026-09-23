use std::collections::HashSet;

#[derive(Clone, Copy)]
struct Node {
    id: &'static str,
    label: &'static str,
    parent: Option<&'static str>,
    depth: i32,
    folder: bool,
    disabled: bool,
}

const NODES: &[Node] = &[
    Node {
        id: "documents",
        label: "Documents",
        parent: None,
        depth: 0,
        folder: true,
        disabled: false,
    },
    Node {
        id: "brief",
        label: "Project brief.md",
        parent: Some("documents"),
        depth: 1,
        folder: false,
        disabled: false,
    },
    Node {
        id: "notes",
        label: "Meeting notes.md",
        parent: Some("documents"),
        depth: 1,
        folder: false,
        disabled: false,
    },
    Node {
        id: "projects",
        label: "Projects",
        parent: None,
        depth: 0,
        folder: true,
        disabled: false,
    },
    Node {
        id: "website",
        label: "Website",
        parent: Some("projects"),
        depth: 1,
        folder: true,
        disabled: false,
    },
    Node {
        id: "homepage",
        label: "Homepage.md",
        parent: Some("website"),
        depth: 2,
        folder: false,
        disabled: false,
    },
    Node {
        id: "assets",
        label: "Assets.md",
        parent: Some("website"),
        depth: 2,
        folder: false,
        disabled: false,
    },
    Node {
        id: "archive",
        label: "Archive (unavailable)",
        parent: None,
        depth: 0,
        folder: false,
        disabled: true,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisibleNode {
    pub id: &'static str,
    pub label: &'static str,
    pub depth: i32,
    pub folder: bool,
    pub expanded: bool,
    pub disabled: bool,
}

pub struct TreeState {
    expanded: HashSet<&'static str>,
    cursor: &'static str,
    selected: Option<&'static str>,
}

impl Default for TreeState {
    fn default() -> Self {
        Self {
            expanded: ["documents", "projects"].into_iter().collect(),
            cursor: "documents",
            selected: None,
        }
    }
}

impl TreeState {
    pub fn visible(&self) -> Vec<VisibleNode> {
        NODES
            .iter()
            .filter(|node| {
                let mut parent = node.parent;
                while let Some(id) = parent {
                    if !self.expanded.contains(id) {
                        return false;
                    }
                    parent = NODES
                        .iter()
                        .find(|candidate| candidate.id == id)
                        .and_then(|n| n.parent);
                }
                true
            })
            .map(|node| VisibleNode {
                id: node.id,
                label: node.label,
                depth: node.depth,
                folder: node.folder,
                expanded: self.expanded.contains(node.id),
                disabled: node.disabled,
            })
            .collect()
    }

    pub fn cursor_index(&self) -> usize {
        self.visible()
            .iter()
            .position(|node| node.id == self.cursor)
            .unwrap_or(0)
    }

    pub fn selected_label(&self) -> Option<&'static str> {
        self.selected.and_then(|id| {
            NODES
                .iter()
                .find(|node| node.id == id)
                .map(|node| node.label)
        })
    }

    pub fn selected_id(&self) -> Option<&'static str> {
        self.selected
    }

    pub fn move_cursor(&mut self, delta: i32) {
        let rows = self.visible();
        let mut index = self.cursor_index() as i32;
        loop {
            let next = index + delta.signum();
            if next < 0 || next >= rows.len() as i32 {
                break;
            }
            index = next;
            if !rows[index as usize].disabled {
                self.cursor = rows[index as usize].id;
                break;
            }
        }
    }

    pub fn activate(&mut self, index: usize) {
        let Some(row) = self.visible().get(index).copied() else {
            return;
        };
        if row.disabled {
            return;
        }
        self.cursor = row.id;
        self.selected = Some(row.id);
        if row.folder {
            if !self.expanded.remove(row.id) {
                self.expanded.insert(row.id);
            }
        }
    }

    pub fn left(&mut self) {
        let Some(node) = NODES.iter().find(|node| node.id == self.cursor) else {
            return;
        };
        if node.folder && self.expanded.remove(node.id) {
            return;
        }
        if let Some(parent) = node.parent {
            self.cursor = parent;
        }
    }

    pub fn right(&mut self) {
        let Some(node) = NODES.iter().find(|node| node.id == self.cursor) else {
            return;
        };
        if node.folder {
            if !self.expanded.insert(node.id) {
                if let Some(child) = NODES
                    .iter()
                    .find(|candidate| candidate.parent == Some(node.id))
                {
                    self.cursor = child.id;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TreeState;

    #[test]
    fn expansion_and_keyboard_navigation_preserve_selection() {
        let mut tree = TreeState::default();
        assert_eq!(tree.visible().len(), 6);
        tree.activate(4);
        assert_eq!(tree.selected_label(), Some("Website"));
        assert_eq!(tree.visible().len(), 8);
        tree.right();
        assert_eq!(tree.cursor_index(), 5);
        tree.left();
        assert_eq!(tree.cursor_index(), 4);
        tree.move_cursor(1);
        assert_eq!(tree.cursor_index(), 5);
        tree.activate(5);
        assert_eq!(tree.selected_label(), Some("Homepage.md"));
    }

    #[test]
    fn disabled_archive_is_skipped() {
        let mut tree = TreeState::default();
        tree.activate(4);
        tree.move_cursor(1);
        tree.move_cursor(1);
        tree.move_cursor(1);
        assert_eq!(tree.cursor_index(), 6);
        tree.move_cursor(1);
        assert_eq!(tree.cursor_index(), 6);
        tree.activate(7);
        assert_eq!(tree.selected_label(), Some("Website"));
    }
}
