use std::{cmp, io::Result};

use crossterm::{
  cursor,
  event::{KeyCode, KeyEvent, KeyModifiers},
  terminal,
};

use super::{INDENT_LEN, CrosstermCli};

pub enum CommandResponse {
  /// Do nothing
  NoOp,
  /// Rerender full screen
  RerenderScreen,
  /// Rerender only the cursor
  RerenderCursor,
  /// Quit the program
  Quit,
}

pub type CommandResult = Result<CommandResponse>;

impl CrosstermCli {
  pub fn handle_key(&mut self, key_event: KeyEvent) -> CommandResult {
    const NONE: KeyModifiers = KeyModifiers::empty();
    match (key_event.code, key_event.modifiers) {
      (KeyCode::Enter, NONE) => self.toogle_expand(),
      (KeyCode::Down, NONE) | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
        self.handle_down()
      },
      (KeyCode::Up, NONE) | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
        self.handle_up()
      },
      (KeyCode::Char('q'), NONE)
      | (KeyCode::Char('c'), KeyModifiers::CONTROL)
      | (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
        Ok(CommandResponse::Quit)
      },
      _ => Ok(CommandResponse::NoOp),
    }
  }
  fn toogle_expand(&mut self) -> CommandResult {
    let curr_node = self.curr_node_mut();
    if curr_node.children.is_empty() {
      Ok(CommandResponse::NoOp)
    } else {
      // 回车之后,如果有children, is_expanded = true
      curr_node.is_expanded = !curr_node.is_expanded;
      Ok(CommandResponse::RerenderScreen)
    }
  }

  fn handle_up(&mut self) -> CommandResult {
    let path = self.current_path.clone();
    if let Some(new_path) = self.get_path_above(path) {
      let cursor::MoveTo(_, row) = self.cursor_pos;
      let new_row = if row == 0 { 0 } else { row - 1 };
      let new_col = col_from_path(&new_path);

      self.cursor_pos = cursor::MoveTo(new_col, new_row);
      self.current_path = new_path;

      if row == 0 {
        let top_path = self.top_path.clone();
        if let Some(new_top_path) = self.get_path_above(top_path) {
          self.top_path = new_top_path;
        }
        // We shifted the screen, so rerender everything
        Ok(CommandResponse::RerenderScreen)
      } else {
        // We did not shift the screen, so rerender only the cursor
        Ok(CommandResponse::RerenderCursor)
      }
    } else {
      Ok(CommandResponse::NoOp)
    }
  }

  fn get_path_above(&self, mut path: Vec<usize>) -> Option<Vec<usize>> {
    if path.is_empty() {
      return None;
    }

    let last_idx = path.len() - 1;
    if path[last_idx] == 0 {
      // We're the 0th child --> go to parent
      path.pop();
    } else {
      // Parent node has a prev child --> go to prev child then find last child
      // of last child...
      path[last_idx] -= 1;

      let mut candidate_node = self.node_at_path(&path[..]);
      let mut path_tail: Vec<usize> = vec![];

      while candidate_node.is_expanded && !candidate_node.children.is_empty() {
        let last_child_idx = candidate_node.children.len() - 1;
        path_tail.push(last_child_idx);
        candidate_node = &candidate_node.children[last_child_idx];
      }

      path.extend(path_tail);
    }
    return Some(path);
  }

  fn handle_down(&mut self) -> CommandResult {
    let path = self.current_path.clone();
    if let Some(new_path) = self.get_path_below(path) {
      let new_col = col_from_path(&new_path);
      let cursor::MoveTo(_, row) = self.cursor_pos;
      let (_, screen_rows) = terminal::size()?;
      let new_row = cmp::min(row + 1, screen_rows - 1);

      self.cursor_pos = cursor::MoveTo(new_col, new_row);
      self.current_path = new_path;

      if row == screen_rows - 1 {
        let top_path = self.top_path.clone();
        if let Some(new_top_path) = self.get_path_below(top_path) {
          self.top_path = new_top_path;
        }
        // We shifted the screen, so rerender everything
        Ok(CommandResponse::RerenderScreen)
      } else {
        // We did not shift the screen, so rerender only the cursor
        Ok(CommandResponse::RerenderCursor)
      }
    } else {
      // We did not actually move, so no need to rerender
      Ok(CommandResponse::NoOp)
    }
  }

  fn get_path_below(&self, path: Vec<usize>) -> Option<Vec<usize>> {
    self.get_path_below_helper(path, false)
  }

  fn get_path_below_helper(
    &self,
    mut path: Vec<usize>,
    skip_children: bool,
  ) -> Option<Vec<usize>> {
    let curr_node = self.node_at_path(&path[..]);
    if !skip_children && curr_node.is_expanded && curr_node.children.len() > 0 {
      path.push(0);
      return Some(path);
    }
    if path.len() == 0 {
      return None;
    }
    let last_idx = path.len() - 1;
    let parent_node = self.node_at_path(&path[..last_idx]);

    // Parent node has a next child --> go to next child
    if parent_node.children.len() - 1 > path[last_idx] {
      path[last_idx] += 1;
      return Some(path);
    }

    // Parent node has no next child --> pop up a level and recurse
    path.pop();
    return self.get_path_below_helper(path, true);
  }
}

fn col_from_path(path: &Vec<usize>) -> u16 {
  // this result will fit into a u16 unless you have > 32,000 nested directories
  // in your filesystem, in which case this tool is not meant for your use case
  (path.len() as u16) * INDENT_LEN + 1
}
