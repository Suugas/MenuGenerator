use chrono::{Duration, Local};
use rand::{Rng, seq::SliceRandom};
use std::collections::HashMap;
use std::fs;

mod menu;
mod schedule;

use menu::{ListIngredient, Recette, Repas};
use schedule::{Day, Week};

use crate::schedule::getNextMonday;

fn charger_recettes(chemin_fichier: &str) -> Vec<Recette> {
    // 1. Lecture du fichier JSON en texte
    let contenu_json =
        fs::read_to_string(chemin_fichier).expect("Erreur : Impossible de lire le fichier JSON");

    // 2. Conversion du texte JSON vers Vec<Recette>
    let recettes: Vec<Recette> = serde_json::from_str(&contenu_json)
        .expect("Erreur : Le format JSON ne correspond pas à Vec<Recette>");

    recettes
}

fn loadWeeks(chemin_fichier: &str) -> Vec<Week> {
    let contenu_json =
        fs::read_to_string(chemin_fichier).expect("Erreur lors de la lecture du schedule");

    let weeks: Vec<Week> =
        serde_json::from_str(&contenu_json).expect("Erreur : le format json ne correspond pas");

    weeks
}
fn saveWeeks(chemin_fichier: &str, weeks: Vec<Week>) {
    let weeks_json = serde_json::to_string_pretty(&weeks)
        .expect("Erreur: impossible de sérialiser les donnée en JSON");
    fs::write(chemin_fichier, weeks_json).expect("Erreur : impossible de sauvegarder les données");
}
fn getRepasByMaxDuration(week: &mut Week) -> &mut Repas {
    let mut max_index: Option<(usize, usize)> = None;
    let mut max_duree: i32 = -1;

    // 1. On cherche les INDEX du jour et du repas le plus long
    for (i, d) in week.days.iter().enumerate() {
        for (j, r) in d.menu.iter().enumerate() {
            if let Some(recette) = &r.recette {
                if recette.duree > max_duree {
                    max_duree = recette.duree;
                    max_index = Some((i, j));
                }
            }
        }
    }

    // 2. On extrait la référence mutable grâce aux index trouvés
    let (day_idx, menu_idx) = max_index.expect("Erreur : Aucune recette trouvée !");
    &mut week.days[day_idx].menu[menu_idx]
}
fn getSumDuration(week: &Week) -> i32 {
    let mut som: i32 = 0;
    for d in &week.days {
        for r in &d.menu {
            if let Some(recette) = &r.recette {
                som += recette.duree;
            }
        }
    }
    return som;
}
fn generateMenu(week: &mut Week, recettes: Vec<Recette>, dureeMax: i32) {
    let mut rng = rand::thread_rng();
    let mut som: i32 = 0;
    for d in &mut week.days {
        for r in &mut d.menu {
            if !recettes.is_empty() && r.isActive {
                r.recette = recettes.choose(&mut rng).cloned();
                if let Some(rec) = &r.recette {
                    som += rec.duree;
                }
            }
        }
    }
    while getSumDuration(week) > dureeMax {
        getRepasByMaxDuration(week).recette = recettes.choose(&mut rng).cloned();
    }
}

fn showWeek(week: &Week) {
    println!("{}", week.display());
}

fn main() {
    let recetteList: Vec<Recette> = charger_recettes("recettes.json");
    let today = Local::now();

    let mut menu1 = Repas {
        name: "test1".to_string(),
        nbPersonne: 1,
        isActive: false,
        recette: None,
    };
    println!("{}", menu1.display());
    let mut menu2 = Repas {
        name: "test2".to_string(),
        nbPersonne: 1,
        isActive: false,
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
        date: getNextMonday(),
        menu: vec![menu1, menu2],
    };
    println!("{}\n\n", &jour.display());
    let mut semaine = Week {
        days: vec![],
        tMax: 200,
    };
    semaine.generate();
    generateMenu(&mut semaine, recetteList, 70);

    showWeek(&semaine);
    let date_str = today.format("%Y-%m-%d").to_string();
    println!("Date du jour : {}", date_str);

    if let Some(jour) = &semaine.getDayByDate(today) {
        println!("{}", jour.display());
    } else {
        println!("AUCUN MENU POUR AUJOURD'HUI");
    }

    println!("{}", getRepasByMaxDuration(&mut semaine).display());
}
