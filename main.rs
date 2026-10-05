struct Repas {
    name: String,
    id: i8,
    recette: String,
}
impl Repas {
    fn display(&self) -> String {
        return format!(
            "Name : {} -  - id : {} - recette : {}",
            self.name, self.id, self.recette,
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
        return format!("{} : \n{}", self.date, menu);
    }
}

struct Week {
    jours: Vec<Day>,
}

fn main() {
    let mut menu1 = Repas {
        name: "test1".to_string(),
        id: 1,
        recette: "recette2".to_string(),
    };
    println!("{}", menu1.display());
    let mut menu2 = Repas {
        name: "test2".to_string(),
        id: 1,
        recette: "recette2".to_string(),
    };
    println!("{}", menu2.display());

    let jour = Day {
        date: "Jour".to_string(),
        menu: vec![menu1, menu2],
    };
    println!("{}", &jour.display());
}
