use crate::menu::Repas;

pub struct Day {
    pub menu: Vec<Repas>,
    pub date: String,
}
impl Day {
    pub fn display(&self) -> String {
        let mut menu = String::from("");
        for repas in &self.menu {
            menu.push_str(&repas.display());
            menu.push_str("\n");
        }
        return format!(" - {} : \n{}", self.date, menu);
    }
}

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
                    recette: None,
                },
                Repas {
                    name: "dej".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "gouter".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "diner".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
            ],
            date: "lun".to_string(),
        };
        let mut mar = Day {
            menu: vec![
                Repas {
                    name: "petit-dej".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "dej".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "gouter".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "diner".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
            ],
            date: "mardi".to_string(),
        };
        let mut mer = Day {
            menu: vec![
                Repas {
                    name: "petit-dej".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "dej".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "gouter".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "diner".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
            ],
            date: "mercredi".to_string(),
        };
        let mut jeu = Day {
            menu: vec![
                Repas {
                    name: "petit-dej".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "dej".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "gouter".to_string(),
                    nbPersonne: 1,

                    recette: None,
                },
                Repas {
                    name: "diner".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
            ],
            date: "jeudi".to_string(),
        };
        let mut ven = Day {
            menu: vec![
                Repas {
                    name: "petit-dej".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "dej".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "gouter".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "diner".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
            ],
            date: "vendredi".to_string(),
        };
        let mut sam = Day {
            menu: vec![
                Repas {
                    name: "petit-dej".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "dej".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "gouter".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "diner".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
            ],
            date: "samedi".to_string(),
        };
        let mut dim = Day {
            menu: vec![
                Repas {
                    name: "petit-dej".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "dej".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "gouter".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
                Repas {
                    name: "diner".to_string(),
                    nbPersonne: 1,
                    recette: None,
                },
            ],
            date: "dimanche".to_string(),
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
    pub fn getDayByDate(&self, date: String) -> Option<&Day> {
        for d in &self.days {
            if d.date.eq(&date) {
                return Some(d);
            }
        }
        return None;
    }
}
