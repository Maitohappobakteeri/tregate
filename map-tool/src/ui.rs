use std::time::SystemTime;

const PROGRESS_BAR_LENGTH: usize = 20;

struct UI {
    pub last_print_time: SystemTime,
}

impl UI {
    pub fn new() -> UI {
        return UI {
            last_print_time: SystemTime::UNIX_EPOCH,
        };
    }

    pub fn print_progress_bar(&mut self, progress: f64) {
        let new_sys_time = SystemTime::now();
        let difference = new_sys_time.duration_since(self.last_print_time);
        if let Ok(d) = difference {
            if d.as_millis() < 200 {
                return;
            }
        }
        self.last_print_time = new_sys_time;

        // let mut stdout = stdout();
        let progress = progress * 100.0;
        let bar_length = PROGRESS_BAR_LENGTH as f64;
        let primary_step = 100.0 / bar_length;

        let primary_progress = (progress / primary_step).round().clamp(0.0, PROGRESS_BAR_LENGTH as f64) as usize;
        let secondary_progress = ((progress % primary_step) / primary_step) as usize
            * (PROGRESS_BAR_LENGTH - primary_progress);
        let empty = PROGRESS_BAR_LENGTH
            - (primary_progress + secondary_progress).clamp(0, PROGRESS_BAR_LENGTH);
        print!(
            "\r[{}{}{}] {:2.1}%",
            "❤".repeat(primary_progress),
            "♡".repeat(secondary_progress),
            "-".repeat(empty),
            progress
        );
        // stdout.flush().unwrap();
    }

    pub fn print_progress_bar_completed(&self) {
        println!("\r[{}] 100%   ", "❤".repeat(PROGRESS_BAR_LENGTH));
    }
}

pub fn print_progress_bar(progress: f64) {
    UI::new().print_progress_bar(progress);
}

pub fn print_progress_bar_completed() {
    UI::new().print_progress_bar_completed();
}
