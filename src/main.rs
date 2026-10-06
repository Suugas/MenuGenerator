use std::collections::HashMap;

mod list_ingredient;

use list_ingredient::ListIngredient;

struct Recette {
    name: String,
    nbPersonne: i8,
    duree: i8,
    ingredients: ListIngredient,
    recette: String,
}
impl Recette {
    fn display(&self) -> String {
        return format!(
            "{} pour {} personnes ({}):\n{}\n{}",
            &self.name,
            &self.nbPersonne,
            &self.duree,
            &self.ingredients.display(),
            &self.recette,
        );
    }
}

struct Repas {
    name: String,
    nbPersonne: i8,
    recette: Option<Recette>,
}
impl Repas {
    fn display(&self) -> String {
        let recette_txt = match &self.recette {
            Some(r) => r.display(),
            None => "Aucune".to_string(),
        };
        return format!(
            "Name: {} - Nb de personne: {} - recette: {}",
            self.name, self.nbPersonne, recette_txt,
        );
    }
}

struct Day {
    menu: Vec<Repas>,
    date: String,
}
impl Day {
    fn display(&self) -> String {
        let mut menu = String::from("");
        for repas in &self.menu {
            menu.push_str(&repas.display());
            menu.push_str("\n");
        }
        return format!(" - {} : \n{}", self.date, menu);
    }
}

struct Week {
    days: Vec<Day>,
    tMax: i32,
}
impl Week {
    fn generate(&mut self) {
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
    fn display(&self) -> String {
        let mut res = String::from("");
        for day in &self.days {
            res.push_str(&day.display());
        }
        return res;
    }
    fn getDayByDate(&self, date: String) -> Option<&Day> {
        for d in &self.days {
            if d.date.eq(&date) {
                return Some(d);
            }
        }
        return None;
    }
}

fn main() {
    let mut menu1 = Repas {
        name: "test1".to_string(),
        nbPersonne: 1,
        recette: None,
    };
    println!("{}", menu1.display());
    let mut menu2 = Repas {
        name: "test2".to_string(),
        nbPersonne: 1,
        recette: Some(Recette {
            name: "Pates".to_string(),
            nbPersonne: 2,
            duree: 15,
            ingredients: ListIngredient {
                ingredients: HashMap::from([("Pates".to_string(), 1), ("Sauce".to_string(), 1)]),
            },
            recette: "".to_string(),
        }),
    };
    println!("{}", menu2.display());

    let jour = Day {
        date: "Jour".to_string(),
        menu: vec![menu1, menu2],
    };
    println!("{}\n\n", &jour.display());
    let mut semaine = Week {
        days: vec![],
        tMax: 200,
    };
    semaine.generate();
    println!("{}", semaine.display());
}
