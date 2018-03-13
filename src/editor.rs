extern crate ncurses;
use ncurses::*;

use view::LogView;
/// Simple text buffer
pub struct Buffer {
	lines: Vec<String>,
}

impl Buffer {

	pub fn new() -> Self {
		Buffer {
			lines: Vec::new(),
		}
	}

	pub fn capture_input(&self, lv: &mut LogView) {

		curs_set(CURSOR_VISIBILITY::CURSOR_VERY_VISIBLE);
		lv.draw_lower_text();
		let index = item_index(current_item(lv.menu)) as usize;
		let ei = {
        	let ref mut mission = lv.log.mission_list()[index];
        	mission.entries.len()-1
        };
		let mut ch = getch();
		while ch != 10 {
			match ch {
				27 => {
					// ESC to cancel
	              	lv.log.mission_list()[index].entries.pop();
					lv.draw_window();
					break;
				},
				KEY_BACKSPACE | KEY_DC | KEY_DL => {
					{
	                   lv.log.mission_list()[index].entries[ei].entry_text.pop();
	                }
					lv.draw_window();
				},
				_ => {
	                {
	                   lv.log.mission_list()[index].entries[ei].entry_text.push(ch as u8 as char);
	                }
					lv.draw_window();
				},
			}
			ch = getch();
		}
		curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);


	}

}