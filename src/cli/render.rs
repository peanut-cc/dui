use crossterm::{
  cursor, execute, queue, style,
  terminal::{self, ClearType},
};

use crate::crawl::data_string;

use super::{CrosstermCli, TreeNode};
use std::io::{Result, Write};

const LAST_CHILD_INDENT: &str = "  ";
const MIDDLE_CHILD_INDENT: &str = "┃ ";

const UNEXPANDED: char = '⊞';
const EXPANDED: char = '⊟';
const NO_CHILDREN: char = '⊡';
const MIDDLE_BULLET: char = '┣';
const END_BULLET: char = '┗';

impl CrosstermCli {
  pub fn render<W>(&self, w: &mut W) -> Result<()>
  where
    W: Write,
  {
    queue!(w, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0),)?;
    let (_, screen_rows) = terminal::size()?;
    let mut rows_rendered = 0;
    self.tree.render(
      w,
      &String::new(),
      true,
      None,
      &self.top_path[..],
      screen_rows,
      &mut rows_rendered,
    )?;
    queue!(w, self.cursor_pos)?;
    w.flush()?;
    Ok(())
  }
  pub fn render_cursor<W>(&self, w: &mut W) -> Result<()>
  where
    W: Write,
  {
    execute!(w, self.cursor_pos)
  }
}

impl TreeNode {
  fn render<W>(
    &self,
    w: &mut W,
    prefix: &String,
    is_last_child: bool,
    parent_size: Option<f64>,
    top_path: &[usize],
    screen_rows: u16,
    rows_rendered: &mut u16,
  ) -> Result<()>
  where
    W: Write,
  {
    if *rows_rendered >= screen_rows {
      return Ok(());
    }
    let float_size = self.size as f64;
    if top_path.is_empty() {
      let bullet = if is_last_child {
        END_BULLET
      } else {
        MIDDLE_BULLET
      };
      let expand_toggle = if self.children.is_empty() {
        NO_CHILDREN
      } else if self.is_expanded {
        EXPANDED
      } else {
        UNEXPANDED
      };
      let size_str = data_string(self.data_type, self.size);
      let size_percent = if let Some(parent_size) = parent_size {
        100.0 * float_size / parent_size
      } else {
        100.0
      };
      queue!(
        w,
        style::Print(format!(
          "{prefix}{bullet}{expand_toggle} {:?}: {size_str} ({:0.2}%)",
          self.path, size_percent,
        )),
        cursor::MoveToNextLine(1),
      )?;
      *rows_rendered += 1;
    }
    let mut child_prefix = prefix.clone();
    if is_last_child {
      child_prefix.extend(LAST_CHILD_INDENT.chars());
    } else {
      child_prefix.extend(MIDDLE_CHILD_INDENT.chars());
    }
    let (to_skip, child_top_path) = if top_path.is_empty() {
      (0, top_path)
    } else {
      (top_path[0], &top_path[1..])
    };
    if self.is_expanded {
      let mut child_iter = self.children.iter().enumerate().skip(to_skip);
      if let Some((first_i, first_c)) = child_iter.next() {
        first_c.render(
          w,
          &child_prefix,
          first_i == self.children.len() - 1,
          Some(float_size),
          child_top_path,
          screen_rows,
          rows_rendered,
        )?;
        for (i, c) in child_iter {
          c.render(
            w,
            &child_prefix,
            i == self.children.len() - 1,
            Some(float_size),
            &[],
            screen_rows,
            rows_rendered,
          )?;
        }
      }
    }
    Ok(())
  }
}
