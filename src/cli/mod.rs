use std::{collections::HashMap, ffi::OsString, io::Result, io::Write, vec};

use crossterm::{
  cursor,
  event::{self, Event, KeyEvent},
  execute, style, terminal,
};

use crate::crawl::{DataType, PathSizeRecord, PathSizeRecorder};

use self::handler::CommandResponse;

mod handler;
mod render;

const INDENT_LEN: u16 = 2;

pub struct CrosstermCli {
  pub tree: TreeNode,
  pub current_path: Vec<usize>,
  pub cursor_pos: cursor::MoveTo,
  pub top_path: Vec<usize>,
}

impl CrosstermCli {
  pub fn from_recorder(recorder: PathSizeRecorder) -> CrosstermCli {
    CrosstermCli {
      tree: TreeNode::from_recorder(recorder),
      current_path: vec![],
      cursor_pos: cursor::MoveTo(1, 0),
      top_path: vec![],
    }
  }

  pub fn run<W>(mut self, w: &mut W) -> Result<()>
  where
    W: Write,
  {
    execute!(w, style::ResetColor, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    self.render(w)?;
    loop {
      match self.handle_key(read_key()?)? {
        CommandResponse::Quit => break,
        CommandResponse::RerenderScreen => self.render(w)?,
        CommandResponse::RerenderCursor => self.render_cursor(w)?,
        CommandResponse::NoOp => {},
      }
    }
    execute!(
      w,
      style::ResetColor,
      cursor::Show,
      terminal::LeaveAlternateScreen
    )?;
    terminal::disable_raw_mode()
  }
  pub fn node_at_path(&self, path: &[usize]) -> &TreeNode {
    let mut curr_node = &self.tree;
    for c in path.as_ref() {
      curr_node = &curr_node.children[*c];
    }
    curr_node
  }

  pub fn curr_node_mut(&mut self) -> &mut TreeNode {
    let mut curr_node = &mut self.tree;
    for c in &self.current_path {
      curr_node = &mut curr_node.children[*c];
    }
    curr_node
  }
}

pub struct TreeNode {
  pub path: OsString,
  pub data_type: DataType,
  pub size: u64,
  pub is_expanded: bool,
  pub children: Vec<TreeNode>,
}

impl TreeNode {
  fn from_recorder(recorder: PathSizeRecorder) -> TreeNode {
    let children = TreeNode::from_recorder_children(recorder.data.children);
    TreeNode {
      path: recorder.root.clone().into_os_string(),
      data_type: recorder.data.data_type,
      size: recorder.data.size,
      is_expanded: false,
      children,
    }
  }

  fn from_recorder_children(
    child_map: HashMap<OsString, PathSizeRecord>,
  ) -> Vec<TreeNode> {
    let mut child_nodes = child_map
      .into_iter()
      .map(|(k, v)| TreeNode {
        path: k,
        data_type: v.data_type,
        size: v.size,
        is_expanded: false,
        children: TreeNode::from_recorder_children(v.children),
      })
      .collect::<Vec<TreeNode>>();
    child_nodes.sort_by(|a, b| b.size.cmp(&a.size));
    child_nodes
  }
}

fn read_key() -> Result<KeyEvent> {
  loop {
    if let Ok(Event::Key(ke)) = event::read() {
      return Ok(ke);
    }
  }
}
