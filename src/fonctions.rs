use crate::menu::{ListIngredient, Recette, Repas, Saison, Tag};
use crate::schedule::{Day, Week};
use chrono::{DateTime, Datelike, Local, TimeZone};
use eframe::emath::Numeric;
use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use tungstenite::{Message, connect};

use crate::schedule::getNextMonday;

const MAX_JOURS_RESTES: usize = 2;

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
pub fn getSumDuration(week: &Week) -> i32 {
    let mut som: i32 = 0;
    for d in &week.days {
        for r in &d.menu {
            if let Some(recette) = &r.recette {
                // un reste ne se recuisine pas : il ne compte pas dans le temps de cuisine
                if !recette.est_reste {
                    som += recette.duree;
                }
            }
        }
    }
    som
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

    apply_nb_mangeur(week, list_nb_mangeur);
    if recettes.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();
    let mut meilleur: Option<Week> = None;
    let mut meilleure_duree = i32::MAX;

    // On retire au sort jusqu'à respecter la durée max (100 essais), sinon on garde le plus court
    for _ in 0..100 {
        assigner_recettes(week, &recettes, &mut rng);
        let duree = getSumDuration(week);
        if duree < meilleure_duree {
            meilleure_duree = duree;
            meilleur = Some(week.clone());
        }
        if duree <= dureeMax {
            break;
        }
    }
    if let Some(m) = meilleur {
        *week = m;
    }
}
pub fn showWeek(week: &Week) {
    println!("{}", week.display());
}

/// Remplit tous les repas de la semaine (un tirage complet).
fn assigner_recettes(week: &mut Week, recettes: &Vec<Recette>, rng: &mut rand::rngs::ThreadRng) {
    // Repas à pourvoir, dans l'ordre chronologique (et on vide les anciennes recettes)
    let mut ordre: Vec<(usize, usize)> = Vec::new();
    for d in 0..week.days.len() {
        for r in 0..week.days[d].menu.len() {
            week.days[d].menu[r].recette = None;
            if week.days[d].menu[r].nbPersonne > 0 {
                ordre.push((d, r));
            }
        }
    }

    // Restes disponibles : (recette, portions restantes, jour de cuisson)
    let mut restes: Vec<(Recette, i32, usize)> = Vec::new();

    for (pos, &(d, r)) in ordre.iter().enumerate() {
        let nb = week.days[d].menu[r].nbPersonne;

        // 1. Y a-t-il des restes à manger ?
        restes.retain(|(_, portions, jour)| *portions > 0 && d - *jour <= MAX_JOURS_RESTES);
        if let Some(i) = restes.iter().position(|(_, portions, _)| *portions >= nb) {
            let mut rec = restes[i].0.clone();
            rec.est_reste = true;
            restes[i].1 -= nb;
            week.days[d].menu[r].recette = Some(rec);
            continue;
        }

        // 2. Sinon on choisit une recette
        let candidats: Vec<&Recette> = recettes
            .iter()
            .filter(|rec| {
                if rec.nbPersonne <= 0 {
                    return false;
                }
                if rec.nbPersonne <= nb {
                    // recette plus petite : on la multiplie
                    return nb % rec.nbPersonne == 0;
                }
                // recette plus grande : OK seulement si un repas suivant peut manger les restes
                let restant = rec.nbPersonne - nb;
                rec.nbPersonne % nb == 0
                    && ordre[pos + 1..].iter().any(|&(d2, r2)| {
                        d2 - d <= MAX_JOURS_RESTES && week.days[d2].menu[r2].nbPersonne <= restant
                    })
            })
            .collect();

        let mut rec: Recette = match candidats.choose(rng) {
            Some(rec) => (*rec).clone(),
            None => match recettes.choose(rng) {
                Some(rec) => rec.clone(), // repli : n'importe laquelle
                None => continue,
            },
        };

        if rec.nbPersonne <= nb {
            // multiplie la recette (arrondi au supérieur, jamais 0)
            let mult = ((nb + rec.nbPersonne - 1) / rec.nbPersonne).max(1);
            rec.multNbRecette(mult);
        } else {
            // recette cuisinée en entier, les portions en trop deviennent des restes
            restes.push((rec.clone(), rec.nbPersonne - nb, d));
        }
        week.days[d].menu[r].recette = Some(rec);
    }
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
    let mut res = ListIngredient {
        ingredients: HashMap::new(),
    };
    for day in &week.days {
        for repas in &day.menu {
            if let Some(r) = &repas.recette {
                // les restes ont déjà été achetés avec le repas d'origine
                if !r.est_reste {
                    res.add_list(&r.ingredients);
                }
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
