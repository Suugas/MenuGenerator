use crate::menu::Repas;
use chrono::{
    DateTime, Datelike, Duration, Local,
    Weekday::{self, Mon},
};
use serde::{Deserialize, Serialize};

pub fn getNextMonday() -> DateTime<Local> {
    let today = Local::now();
    for i in 1..7 {
        if (today + Duration::days(i)).weekday() == Weekday::Mon {
            return (today + Duration::days(i));
        }
    }
    unreachable!()
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
    pub fn generate(&mut self) {
        let mut lun = Day {
            menu: vec![
                Repas {
                    name: "petit-dej".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "dej".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "gouter".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "diner".to_string(),
                    nbPersonne: 1,
                    isActive: true,
                    recette: None,
                },
            ],
            date: getNextMonday(),
        };
        let mut mar = Day {
            menu: vec![
                Repas {
                    name: "petit-dej".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "dej".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "gouter".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "diner".to_string(),
                    nbPersonne: 1,
                    isActive: true,
                    recette: None,
                },
            ],
            date: getNextMonday() + Duration::days(1),
        };
        let mut mer = Day {
            menu: vec![
                Repas {
                    name: "petit-dej".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "dej".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "gouter".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "diner".to_string(),
                    nbPersonne: 1,
                    isActive: true,
                    recette: None,
                },
            ],
            date: getNextMonday() + Duration::days(2),
        };
        let mut jeu = Day {
            menu: vec![
                Repas {
                    name: "petit-dej".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "dej".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "gouter".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "diner".to_string(),
                    nbPersonne: 1,
                    isActive: true,
                    recette: None,
                },
            ],
            date: getNextMonday() + Duration::days(3),
        };
        let mut ven = Day {
            menu: vec![
                Repas {
                    name: "petit-dej".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "dej".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "gouter".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "diner".to_string(),
                    nbPersonne: 1,
                    isActive: true,
                    recette: None,
                },
            ],
            date: getNextMonday() + Duration::days(4),
        };
        let mut sam = Day {
            menu: vec![
                Repas {
                    name: "petit-dej".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "dej".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "gouter".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "diner".to_string(),
                    nbPersonne: 1,
                    isActive: true,
                    recette: None,
                },
            ],
            date: getNextMonday() + Duration::days(5),
        };
        let mut dim = Day {
            menu: vec![
                Repas {
                    name: "petit-dej".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "dej".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "gouter".to_string(),
                    nbPersonne: 1,
                    isActive: false,
                    recette: None,
                },
                Repas {
                    name: "diner".to_string(),
                    nbPersonne: 1,
                    isActive: true,
                    recette: None,
                },
            ],
            date: getNextMonday() + Duration::days(6),
        };
        self.days = vec![lun, mar, mer, jeu, ven, sam, dim];
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
