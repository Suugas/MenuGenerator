use crate::menu::Repas;
use chrono::{DateTime, Datelike, Duration, Local, Weekday};
use serde::{Deserialize, Serialize};

pub fn getNextMonday() -> DateTime<Local> {
    let today = Local::now();
    for i in 1..=7 {
        if (today + Duration::days(i)).weekday() == Weekday::Mon {
            return today + Duration::days(i);
        }
    }
    unreachable!()
}
pub fn getNextMondayFrom(semaines: &Vec<Week>) -> DateTime<Local> {
    let today = match semaines.last() {
        Some(s) => s.days[6].date,
        None => Local::now(),
    };
    for i in 1..7 {
        if (today + Duration::days(i)).weekday() == Weekday::Mon {
            return today + Duration::days(i);
        }
    }
    unreachable!()
}

/// Nombre de personnes par défaut : [petit-déj, déjeuner, goûter, dîner] pour chaque jour (lundi -> dimanche)
const NB_PERSONNES_DEFAUT: [[i32; 4]; 7] = [
    [2, 2, 2, 1], // lundi
    [2, 2, 2, 1], // mardi
    [2, 2, 2, 2], // mercredi
    [2, 2, 2, 2], // jeudi
    [2, 2, 2, 2], // vendredi
    [2, 2, 2, 3], // samedi
    [2, 3, 2, 3], // dimanche
];
const NOMS_REPAS: [&str; 4] = ["petit-dej", "dej", "gouter", "diner"];

/// Construit les 7 jours d'une semaine à partir de la date de son lundi
fn jours_depuis(lundi: DateTime<Local>) -> Vec<Day> {
    (0..7)
        .map(|i| Day {
            menu: (0..4)
                .map(|r| Repas::new(NOMS_REPAS[r], NB_PERSONNES_DEFAUT[i][r]))
                .collect(),
            date: lundi + Duration::days(i as i64),
        })
        .collect()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Day {
    pub menu: Vec<Repas>,
    pub date: DateTime<Local>,
}
impl Day {
    pub fn display(&self) -> String {
        let mut menu = String::from("");
        for repas in &self.menu {
            menu.push_str(&repas.display());
            menu.push_str("\n");
        }
        return format!(" - {} : \n{}", self.date.format("%Y-%m-%d"), menu);
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Week {
    pub days: Vec<Day>,
    pub tMax: i32,
}
impl Week {
    /// Semaine commençant le prochain lundi
    pub fn generate(&mut self) {
        self.days = jours_depuis(getNextMonday());
    }
    /// Semaine qui suit la dernière semaine de `semaines`
    pub fn generate_from_weeks(&mut self, semaines: &Vec<Week>) {
        self.days = jours_depuis(getNextMondayFrom(semaines));
    }
    /// Semaine en cours (lundi de cette semaine)
    pub fn generate_this_week(&mut self) {
        self.days = jours_depuis(getNextMonday() - Duration::days(7));
    }
    /// Numéro de la semaine (norme ISO) d'après le premier jour
    pub fn numero(&self) -> u32 {
        self.days
            .first()
            .map(|d| d.date.iso_week().week())
            .unwrap_or(0)
    }
    pub fn display(&self) -> String {
        let mut res = String::from("");
        for day in &self.days {
            res.push_str(&day.display());
        }
        return res;
    }
    pub fn getDayByDate(&self, date: DateTime<Local>) -> Option<&Day> {
        for d in &self.days {
            if d.date.format("%Y-%m-%d").to_string() == date.format("%Y-%m-%d").to_string() {
                return Some(d);
            }
        }
        return None;
    }
}
