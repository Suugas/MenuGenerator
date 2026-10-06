use std::collections::HashMap;
use std::fs;

mod menu;
mod schedule;

use menu::{ListIngredient, Recette, Repas};
use schedule::{Day, Week};

fn charger_recettes(chemin_fichier: &str) -> Result<Vec<Recette>, Box<dyn std::error::Error>> {
    // 1. Lecture du fichier JSON en texte
    let contenu_json = fs::read_to_string(chemin_fichier)?;

    // 2. Conversion du texte JSON vers Vec<Recette>
    let recettes: Vec<Recette> = serde_json::from_str(&contenu_json)?;

    Ok(recettes)
}

fn main() {
    match charger_recettes("recettes.json") {
        Ok(recettes) => {
            println!("{} recette(s) chargée(s) avec succès !", recettes.len());
            for r in recettes {
                println!("- {}", r.name);
            }
        }
        Err(e) => {
            eprintln!("Erreur lors du chargement des recettes : {}", e);
            return;
        }
    }

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
