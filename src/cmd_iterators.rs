use rustyline::{Editor, Config, history::MemHistory};

// An Iterator that calls readline for each next(), and 
// returns either the text read or None to terminate.
//
// Supports readline style history.
pub struct ReadlineCommands {
    ed: Editor<(), MemHistory>
}

impl ReadlineCommands {
    pub fn new() -> Self {
        let config = Config::builder().auto_add_history(true);
        let history = MemHistory::new();
        let ed = Editor::with_history(config.build(), history).unwrap();
        Self {
            ed
        }
    }
}

impl Iterator for ReadlineCommands {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(line) = self.ed.readline("> ") {
            //self.ed..add_history_entry(line.as_str())?;
            Some(line)
        } else {
            None
        }
    }
}