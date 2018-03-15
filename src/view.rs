
extern crate ncurses;
use ncurses::*;
use alws::*;

use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

use editor::Buffer;

use chrono::prelude::*;

pub struct LogView {
    pub menu: MENU,
    pub items: Vec<ITEM>,
    pub menu_window: WINDOW,
    pub details: WINDOW,
    pub details_window: WINDOW,
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
        
        let mut buf = Buffer::new(); 

        let path = Path::new("log.txt");
        let display = path.display();



		curs_set(CURSOR_VISIBILITY::CURSOR_VERY_VISIBLE);
        while buf.take_input() == Ok(()) {
            let (mut row, mut col): (i32, i32) = (0, 0);
            getmaxyx(self.details_window, &mut row, &mut col); 

            for (i, line) in buf.lines.iter().enumerate() {
                clrprintw(self.details_window, i as i32, 0, line);
                clrprintw(self.details_window, i as i32 + 1, 0, "~");
            }
            clrprintw(self.details_window, 30, 0, &format!("POS - {}:{}", buf.pos.0, buf.pos.1));
            
            refresh();
            wmove(self.details_window, buf.pos.1 as i32, buf.pos.0 as i32);
            wrefresh(self.details_window);
            // Open a file in write-only mode, returns `io::Result<File>`
            let mut file = match File::create(&path) {
                Err(why) => panic!("couldn't create {}: {}",
                                   display,
                                   why.description()),
                Ok(file) => file,
            };
            for (i, line) in buf.lines.iter().enumerate() {
                let mut to_print = String::new();
                to_print.push_str(line);
                if i != buf.lines.len()-1 {
                    to_print.push_str("\n");
                }
                match file.write(to_print.as_bytes()) {
                    Err(_) => panic!("FuCK"),
                    Ok(_) => (),
                }
            }

        }



		curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
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

        set_menu_mark(my_menu, "> ");

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
        mvwprintw(my_menu_win, 0, 2, "MISSION LIST");
        refresh();
        
        post_menu(my_menu);
        wrefresh(my_menu_win);
        
        self.menu = my_menu;
        self.menu_window = my_menu_win;

        wresize(self.details, LINES()-2, COLS()-cols);
        mvwin(self.details, 0, cols);
        wresize(self.details_window, LINES()-4, (COLS()-cols)-3);
        mvwin(self.details_window, 1, cols+2);

        self.draw_window();

    }


    pub fn draw_window(&mut self) {
        //need a subwindow to prevent destroying border


        werase(self.details);
        box_(self.details, 0, 0);
        mvwprintw(self.details, 0, 2, "MISSION DETAILS");
        wrefresh(self.details);

        werase(self.details_window);

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
            
            let basic_format = |ref utc: DateTime<Utc>| {
                let local = utc.with_timezone(&Local);
                let fmt = format!("%F at %T");
                local.format(&fmt).to_string()
            };

            for entry in &mission.entries {
                wprint(self.details_window, &format!("\n\n{}\n", basic_format(entry.timestamp)));
                wprint(self.details_window, &format!("{}", entry.entry_text));
            }

        };

        wrefresh(self.details_window);


    }

    pub fn draw_lower_text(&mut self) {
                // Probably needs to be redone anytime the details window is redone
        clrprint(LINES()-2, 0, "ALWS pre-alpha development build");
        clrprint(LINES()-1, 0, "Press <A> to add new entry, Q to quit");

    }

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
