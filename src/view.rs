
extern crate ncurses;
use ncurses::*;
use alws::*;

use editor::InputType;

use editor::Buffer;

use chrono::prelude::*;

pub struct LogView {
    pub menu: MENU,
    pub items: Vec<ITEM>,
    pub menu_window: WINDOW,
    pub details: WINDOW,
    pub details_window: WINDOW,
    pub info: WINDOW,
    pub info_window: WINDOW,
    pub log: Log,
}

impl LogView {
    pub fn new(log: Log) -> Self {
        
        let mut lv = LogView {
            menu: new_menu(&mut Vec::new()),
            items: Vec::new(),
            menu_window: newwin(1, 1, 0, 0),
            details: newwin(2, 2, 0, 0),
            details_window: newwin(2, 2, 0, 0),
            info: newwin(2, 2, 0, 0),
            info_window: newwin(2, 2, 0, 0),
            log,
        };

        lv.free_menu();
        lv.build_menu(0);
        lv.draw_lower_text();
        lv
    }

    pub fn resize(&mut self) {
        self.draw_lower_text();
        let index = item_index(current_item(self.menu)) as usize;
        unpost_menu(self.menu);
        self.free_menu();
        self.build_menu(index);
    }


    pub fn new_node(&mut self, index: usize) {
        unpost_menu(self.menu);
        self.free_menu();
        self.build_menu(index);

        wmove(self.details_window, 0, 0);
        werase(self.details_window);
        
        let name = input_box(self.details_window);

        let desc = input_box(self.details_window);

        {
                    let ref mut mission = self.log.mission_list()[index];
                    mission.title = name.to_string();
                    mission.description = desc.to_string();

        }
    }

    pub fn up(&mut self) {
        menu_driver(self.menu, REQ_UP_ITEM);
        wrefresh(self.menu_window);
        self.draw_window();
    }

    pub fn down(&mut self) {
        menu_driver(self.menu, REQ_DOWN_ITEM);
        wrefresh(self.menu_window);
        self.draw_window();
    }

    pub fn free_menu(&mut self) {
        for &item in self.items.iter() {
            free_item(item);
        }
        self.items.clear();
    }

    pub fn build_menu(&mut self, index: usize) {

        for mission in &self.log.mission_list() {
            self.items.push(new_item(mission.title.clone(), mission.description.clone()));
        }
        let my_menu = new_menu(&mut self.items);
        menu_opts_off(my_menu, O_SHOWDESC);

        if self.log.mission_list().len() > 0 {
            set_current_item(my_menu, self.items[index]);
        }

        set_menu_mark(my_menu, "");

        let (mut rows, mut cols) = (0, 0);
        scale_menu(my_menu, &mut rows, &mut cols);
        rows = LINES() - 2;
        if cols > COLS() / 3 { cols = cols / 3};
        if cols < 12 { cols = 12};
        cols += 4;

        let my_menu_win = newwin(rows, cols, 0, 0);
        set_menu_win(my_menu, my_menu_win);
        let subwindow = derwin(my_menu_win, rows-2, cols-2, 2, 2);
        set_menu_sub(my_menu, subwindow);
        keypad(my_menu_win, true);

        box_(my_menu_win, 0, 0);
        mvwprintw(my_menu_win, 0, 2, " MISSION LIST ");
        refresh();
        
        post_menu(my_menu);
        wrefresh(my_menu_win);
        
        self.menu = my_menu;
        self.menu_window = my_menu_win;

        // TODO: No magic
        // And encapsulate windows? Perhaps.

        let d_height = 14;
        wresize(self.details, d_height, COLS()-cols);
        mvwin(self.details, 0, cols);
        wresize(self.details_window, d_height - 2, (COLS()-cols)-3);
        mvwin(self.details_window, 1, cols+2);

        wresize(self.info, LINES()-d_height-2, COLS()-cols);
        mvwin(self.info, d_height, cols);
        box_(self.info, 0, 0);
        wrefresh(self.info);
        wresize(self.info_window, LINES()-d_height-4, COLS()-cols-3);
        mvwin(self.info_window, d_height+1, cols+2);
        //box_(self.info_window, 0, 0);
        wrefresh(self.info_window);

        self.draw_window();
    }


    pub fn draw_window(&mut self) {
        //need a subwindow to prevent destroying border


        werase(self.details);
        werase(self.info);
        box_(self.details, 0, 0);
        box_(self.info, 0, 0);

        mvwprintw(self.details, 0, 2, " MISSION DETAILS ");
        mvwprintw(self.info, 0, 2, " LOG ENTRIES ");
        wrefresh(self.details);
        wrefresh(self.info);

        werase(self.details_window);
        werase(self.info_window);

        wmove(self.details_window, 1, 0);



        let pretty_format = |ref utc: DateTime<Utc>| {
            let local = utc.with_timezone(&Local);
            let fmt = format!("%A, the {}{} of %B at %T", local.day(), day_suffixer(local.day()));
            local.format(&fmt).to_string()
        };
        
        if self.log.mission_list().len() > 0 {
            let index = item_index(current_item(self.menu)) as usize;
            let ref mission = self.log.mission_list()[index];


            let start = mission.timestamp.clone();
            let status = match &mission.completion {
                &None => format!("Ongoing since {}", pretty_format(start)),
                &Some(ref dt) => format!("Completion on {}", pretty_format(dt.timestamp)),
            };
            wprint(self.details_window, &format!("{}\nStatus: {}\n\nMission brief:\n{}\n", mission.title, status, mission.description));
                    
            wmove(self.info_window, 1, 0);

            let basic_format = |ref utc: DateTime<Utc>| {
                let local = utc.with_timezone(&Local);
                let fmt = format!("%F at %T");
                local.format(&fmt).to_string()
            };

            for entry in &mission.entries {
                wprint(self.info_window, &format!("{}\n", basic_format(entry.timestamp)));
                wprint(self.info_window, &format!("{}\n\n", entry.entry_text));
            }

        };

        wrefresh(self.details_window);
        wrefresh(self.info_window);


    }

    pub fn draw_lower_text(&mut self) {
                // Probably needs to be redone anytime the details window is redone
        clrprint(LINES()-2, 0, "ALWS pre-alpha development build");
        clrprint(LINES()-1, 0, "Press <A> to add new entry, Q to quit");

    }

}

/// Creates an editable region within the given window
pub fn input_box(window: WINDOW) -> Buffer {

    werase(window);    
    let mut buf = Buffer::new(InputType::MultiLine); 

    curs_set(CURSOR_VISIBILITY::CURSOR_VERY_VISIBLE);
    
    refresh();
    wmove(window, buf.pos.1 as i32, buf.pos.0 as i32);
    wrefresh(window);

    while buf.take_input() == Ok(()) {
        let (mut row, mut col): (i32, i32) = (0, 0);
        getmaxyx(window, &mut row, &mut col); 

        let mut ax = 0;
        let mut ay = 0;

        let mut extra: i32 = 0;
        for (i, line) in buf.lines.iter().enumerate() {

            let mut gain = 0;
            if col > 0 && line.len() > 0 {
                gain = line.len() as i32 / col;
            }
            if i == buf.pos.1 {
                ax = buf.pos.0;
                ay = i as i32 + extra; // now at start of line

                if gain > 0 {
                    if buf.pos.1 as i32 > col {
                        let minigain = buf.pos.1 as i32 / col;
                        ay += minigain; // now at start of line
                        ax = buf.pos.0 - (minigain * col) as usize;
                    }
                }
            }
            for l in 0..gain+1 {
                clrprintw(window, i as i32 + extra + l + 1, 0, "");
            }
            clrprintw(window, i as i32 + extra, 0, line);
            extra += gain;
        }
        
        refresh();

        wmove(window, ay as i32, ax as i32);
        wrefresh(window);
    }
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    buf
}

fn clrprint(y: i32, x: i32, string: &str) {
    mv(y, x);
    clrtoeol();
    mvprintw(y, x, string);
}

fn clrprintw(window: WINDOW, y: i32, x: i32, string: &str) {
    wmove(window, y, x);
    wclrtoeol(window);
    mvwprintw(window, y, x, string);
}

fn wprint(window: WINDOW, string: &str) {
    wclrtoeol(window);
    wprintw(window, string);
}

fn day_suffixer(day: u32) -> String {
    if day >= 11 && day <= 13 {
        return "th".to_string();
    }
    match day % 10 {
        1 => return "st".to_string(),
        2 => return "nd".to_string(),
        3 => return "rd".to_string(),
        _ => return "th".to_string(),

    }
}
