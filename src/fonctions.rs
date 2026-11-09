use crate::menu::{ListIngredient, Recette, Repas, Saison};
use crate::schedule::{Day, Week};
use chrono::{DateTime, Datelike, Local, TimeZone};
use eframe::emath::Numeric;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use tungstenite::{Message, connect};

use crate::schedule::getNextMonday;

pub fn charger_recettes(chemin_fichier: &str) -> Vec<Recette> {
    // 1. Lecture du fichier JSON en texte
    let contenu_json =
        fs::read_to_string(chemin_fichier).expect("Erreur : Impossible de lire le fichier JSON");

    // 2. Conversion du texte JSON vers Vec<Recette>
    let recettes: Vec<Recette> = serde_json::from_str(&contenu_json)
        .expect("Erreur : Le format JSON ne correspond pas à Vec<Recette>");

    recettes
}
pub fn loadWeeks(chemin_fichier: &str) -> Vec<Week> {
    let contenu_json =
        fs::read_to_string(chemin_fichier).expect("Erreur lors de la lecture du schedule");

    let weeks: Vec<Week> =
        serde_json::from_str(&contenu_json).expect("Erreur : le format json ne correspond pas");

    weeks
}
pub fn loadConfig(chemin_fichier: &str) -> Config {
    let contenu_json =
        fs::read_to_string(chemin_fichier).expect("Erreur lors de la lecture du schedule");
    let config: Config =
        serde_json::from_str(&contenu_json).expect("Erreur : le format json ne correspond pas");
    config
}
pub fn save_config(chemin_fichier: &str, config: &Config) {
    let config_json = serde_json::to_string_pretty(&config)
        .expect("Erreur: impossible de sérialiser les donnée en JSON");
    fs::write(chemin_fichier, config_json).expect("Erreur : impossible de sauvegarder les données");
}
pub fn saveWeeks(chemin_fichier: &str, weeks: &Vec<Week>) {
    let weeks_json = serde_json::to_string_pretty(&weeks)
        .expect("Erreur: impossible de sérialiser les donnée en JSON");
    fs::write(chemin_fichier, weeks_json).expect("Erreur : impossible de sauvegarder les données");
}
pub fn getRepasByMaxDuration(week: &mut Week) -> &mut Repas {
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
pub fn getSumDuration(week: &Week) -> i32 {
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
pub fn generateMenu(
    week: &mut Week,
    recettes: &Vec<Recette>,
    list_nb_mangeur: &Vec<i32>,
    dureeMax: i32,
    is_recette_filtre: bool,
) {
    let recettes: Vec<Recette> = recettes
        .clone()
        .into_iter()
        .filter(|r| r.activable && (!is_recette_filtre || r.is_valable()))
        .collect();

    let mut rng = rand::thread_rng();
    apply_nb_mangeur(week, list_nb_mangeur);

    for d in 0..(week.days.len() as i32) {
        let days = week.days.clone();
        for r in 0..(week.days[d as usize].menu.len() as i32) {
            let repas = days[d as usize].menu.clone();
            if !recettes.is_empty() && list_nb_mangeur[(r * (days.len() as i32) + d) as usize] > 0 {
                // 1. On filtre les recettes valides pour ce repas
                // Une recette convient si :
                // - r.nbPersonne % rec.nbPersonne == 0 (la recette est un diviseur du nombre de mangeurs)
                // - OU rec.nbPersonne == r.nbPersonne (pile le même nombre)
                let mut recettes_valides: Vec<&Recette> = recettes
                    .iter()
                    .filter(|rec| {
                        rec.nbPersonne > 0
                            && (repas[r as usize].nbPersonne % rec.nbPersonne == 0
                                || rec.nbPersonne == repas[r as usize].nbPersonne)
                    })
                    .collect();

                // 2. On pioche parmi les recettes éligibles (si la liste n'est pas vide)
                if let Some(recette_choisie) = recettes_valides.choose(&mut rng) {
                    week.days[d as usize].menu[r as usize].recette =
                        Some((*recette_choisie).clone());
                } else {
                    // Repli : si aucune ne correspond aux critères de personnes, on pioche n'importe laquelle
                    week.days[d as usize].menu[r as usize].recette =
                        recettes.choose(&mut rng).cloned();
                }
            }
        }
    }

    // Ajustement de la durée max avec sécurité pour éviter les boucles infinies
    let mut tentatives = 0;
    while getSumDuration(week) > dureeMax && tentatives < 100 {
        let repas = getRepasByMaxDuration(week);

        let recettes_valides: Vec<&Recette> = recettes
            .iter()
            .filter(|rec| rec.nbPersonne > 0 && (repas.nbPersonne % rec.nbPersonne == 0))
            .collect();

        if let Some(recette_choisie) = recettes_valides.choose(&mut rng) {
            repas.recette = Some((*recette_choisie).clone());
        } else {
            repas.recette = recettes.choose(&mut rng).cloned();
        }

        tentatives += 1;
    }

    // Ajustement des ingredient par rapport au nombre de mangeur
    for d in &mut week.days {
        for r in &mut d.menu {
            if let Some(rec) = &mut r.recette {
                let mult = r.nbPersonne / rec.nbPersonne;
                rec.multNbRecette(mult);
                println!("{}", mult);
            }
        }
    }
}
pub fn showWeek(week: &Week) {
    println!("{}", week.display());
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub ip: String,
    pub port: i32,
    pub c_liste: String,
    pub list_nb_mangeur: Vec<i32>,
    pub duree_max: i32,
    pub is_recette_filtre: bool,
    pub auto_send_sl: bool,
}
impl Config {
    /// Retourne l'URL du WebSocket (ex: ws://51.77.150.177:8887)
    pub fn get_ws_url(&self) -> String {
        format!("ws://{}:{}", self.ip, self.port)
    }

    pub fn ensure_room_code(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        if !self.c_liste.trim().is_empty() {
            println!(
                "[DEBUG] Code de salon existant réutilisé : '{}'",
                self.c_liste
            );
            return Ok(self.c_liste.clone());
        }

        let url = self.get_ws_url();
        println!(
            "[DEBUG] Connexion au serveur WebSocket pour générer un nouveau code room ({})",
            url
        );
        let (mut socket, _) = connect(&url)?;

        let req = json!({ "action": "generateCode" });
        println!("[DEBUG] Envoi : {}", req);
        socket.send(Message::Text(req.to_string()))?;

        if let Ok(Message::Text(response_str)) = socket.read() {
            println!("[DEBUG] Réponse reçue du serveur : {}", response_str);
            let response: serde_json::Value = serde_json::from_str(&response_str)?;
            if response["type"] == "codeGenerated" {
                if let Some(new_code) = response["code"].as_str() {
                    self.c_liste = new_code.to_string();
                    println!("[DEBUG] Nouveau code obtenu : '{}'", self.c_liste);
                    return Ok(self.c_liste.clone());
                }
            }
        }

        Err("Impossible d'obtenir un nouveau code de salon depuis le serveur.".into())
    }

    pub fn send_ingredients_to_room(
        &mut self,
        ingredients: &ListIngredient,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("[DEBUG] Préparation de l'envoi des ingrédients au serveur...");
        let code = self.ensure_room_code()?;
        let url = self.get_ws_url();

        println!("[DEBUG] Connexion à le serveur WebSocket : {}", url);
        let (mut socket, _) = connect(&url)?;

        let items: Vec<serde_json::Value> = ingredients
            .ingredients
            .iter()
            .map(|(nom, element)| {
                json!({
                    "name": nom,
                    "qty": element.quantite,
                    "checked": false
                })
            })
            .collect();

        let req = json!({
            "action": "append",
            "code": code,
            "items": items
        });

        println!("[DEBUG] Trame JSON envoyée : {}", req);
        socket.send(Message::Text(req.to_string()))?;

        let _ = socket.close(None);

        // On force Rust à attendre que le serveur valide la déconnexion
        // Cela garantit que le message JSON a bien été reçu et traité.
        loop {
            if socket.read().is_err() {
                break; // La connexion est officiellement et proprement fermée
            }
        }

        println!("[OK] Ingrédients envoyés au salon '{}' avec succès !", code);

        Ok(())
    }

    /// Sauvegarde la configuration actuelle dans le fichier JSON spécifié
    pub fn save(&self, chemin_fichier: &str) {
        let config_json = serde_json::to_string_pretty(&self)
            .expect("Erreur : impossible de sérialiser la configuration en JSON");
        fs::write(chemin_fichier, config_json)
            .expect("Erreur : impossible de sauvegarder la configuration dans le fichier");
    }
}

pub fn generate_list_repas_bool() -> Vec<bool> {
    return vec![
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false,
    ];
}

pub fn apply_nb_mangeur(week: &mut Week, list_nb: &Vec<i32>) {
    for day in 0..week.days.len() {
        for r in 0..week.days[day].menu.len() {
            let index = (r * week.days.len() + day) as usize;
            week.days[day].menu[r].nbPersonne = list_nb[index];
        }
    }
}

pub fn add_value(b: &mut i32, v: i32) {
    if *b + v >= 0 {
        *b += v;
    }
}

pub fn creer_liste_courses(week: &Week) -> ListIngredient {
    let mut res: ListIngredient = ListIngredient {
        ingredients: HashMap::new(),
    };

    for day in week.days.clone() {
        for repas in day.menu {
            if let Some(r) = repas.recette {
                res.add_list(&r.ingredients);
            }
        }
    }
    res
}

pub fn sauvegarder_recettes(
    chemin_fichier: &str,
    recettes: &Vec<Recette>,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(recettes)?;
    fs::write(chemin_fichier, json)?;
    Ok(())
}

pub fn get_saison(date: &DateTime<Local>) -> Saison {
    match date.month() {
        3..=5 => Saison::Printemps,
        6..=8 => Saison::Ete,
        9..=11 => Saison::Automne,
        _ => Saison::Hiver, // 12, 1, 2
    }
}
