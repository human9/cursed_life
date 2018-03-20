extern crate alws;
extern crate chrono;

use alws::*;

mod view;
mod editor;

use view::*;
use editor::Buffer;
use std::env;
use editor::InputType;


extern crate ncurses;
use ncurses::*;

fn main() {

    env::set_var("ESCDELAY", "25");
    initscr();
    use_default_colors();
    start_color();
    cbreak();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    keypad(stdscr(), true);

    init_pair(1, -1, -1);
    
    let path = default_path();
    let file = open_file(&path);
    let mut lv = view::LogView::new(open_log(&file));

    let mut ch = getch();
    while ch != 81 && ch != 113 /* Upper and lower case Q */ {
        match ch {
            78 | 110 => {
                //prompt for name

                // add new missions
                let mission_index = lv.log.new_mission();
                lv.new_node(mission_index);

            },
            65 | 97 => /* A */ {
                let entry_text = input_box(lv.details_window).to_string();
                let entry = MissionEntry::new(entry_text);
                let index = item_index(current_item(lv.menu)) as usize;
                {
                    let ref mut mission = lv.log.mission_list()[index];
                    mission.add_entry(entry);
                }
                lv.draw_window();
            },
            KEY_RESIZE => {
                lv.resize();
            },
            KEY_UP => {
                lv.up();
            },
            KEY_DOWN => {
                lv.down();
            },
            10 => {/* Enter */
                pos_menu_cursor(lv.menu);
            },
            _ => {}
        }
        ch = getch();
    }
    
    endwin();

    write_to_file(&path, &lv.log);
}
