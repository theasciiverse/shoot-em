//! The Hall of Fame: 128 places with a nickname and a two-line message,
//! seeded the way the original seeded a new table (FUN_0040dcec), kept in
//! the player's state folder.

use std::path::PathBuf;

pub const PLACES: usize = 128;

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub nick: String,
    pub msg1: String,
    pub msg2: String,
    pub score: u32,
}

pub struct Table {
    pub entries: Vec<Entry>,
    path: Option<PathBuf>,
}

impl Table {
    pub fn default_entries() -> Vec<Entry> {
        let mut v = vec![
            Entry {
                nick: "Free Slot".into(),
                msg1: "Play to get your name here!".into(),
                msg2: "And then upload your score.".into(),
                score: 10_000,
            };
            PLACES
        ];
        v[0] = Entry {
            nick: "Tringi".into(),
            msg1: "Grr. No one can beat me !!!".into(),
            msg2: "...not even you...".into(),
            score: 100_000,
        };
        v[1] = Entry {
            nick: "Joe".into(),
            msg1: "This is reference score.".into(),
            msg2: "I am sure you can get more pts.".into(),
            score: 50_000,
        };
        v
    }

    pub fn load(path: Option<PathBuf>) -> Table {
        let mut entries = Vec::new();
        if let Some(p) = &path {
            if let Ok(text) = std::fs::read_to_string(p) {
                for line in text.lines() {
                    let f: Vec<&str> = line.split('\t').collect();
                    if f.len() == 4 {
                        if let Ok(score) = f[0].parse() {
                            entries.push(Entry {
                                score,
                                nick: f[1].into(),
                                msg1: f[2].into(),
                                msg2: f[3].into(),
                            });
                        }
                    }
                }
            }
        }
        if entries.len() != PLACES {
            entries = Table::default_entries();
        }
        Table { entries, path }
    }

    pub fn save(&self) {
        let Some(p) = &self.path else { return };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let clean = |s: &str| s.replace(['\t', '\n', '\r'], " ");
        let text: String = self
            .entries
            .iter()
            .map(|e| {
                format!(
                    "{}\t{}\t{}\t{}\n",
                    e.score,
                    clean(&e.nick),
                    clean(&e.msg1),
                    clean(&e.msg2)
                )
            })
            .collect();
        let _ = std::fs::write(p, text);
    }

    /// Does this score earn a place? (It must beat the last one.)
    pub fn qualifies(&self, score: u32) -> bool {
        self.entries.last().map(|e| score > e.score).unwrap_or(true)
    }

    /// Puts the entry above the first place it beats; the last falls off.
    pub fn insert(&mut self, e: Entry) -> usize {
        let at = self
            .entries
            .iter()
            .position(|x| x.score < e.score)
            .unwrap_or(self.entries.len());
        self.entries.insert(at, e);
        self.entries.truncate(PLACES);
        at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_table_is_the_original_one() {
        let t = Table::load(None);
        assert_eq!(t.entries.len(), 128);
        assert_eq!(t.entries[0].nick, "Tringi");
        assert_eq!(t.entries[127].score, 10_000);
        assert!(!t.qualifies(10_000));
        assert!(t.qualifies(10_001));
    }

    #[test]
    fn a_score_goes_in_its_place() {
        let mut t = Table::load(None);
        let at = t.insert(Entry {
            nick: "A".into(),
            msg1: String::new(),
            msg2: String::new(),
            score: 60_000,
        });
        assert_eq!(at, 1);
        assert_eq!(t.entries.len(), 128);
    }
}
