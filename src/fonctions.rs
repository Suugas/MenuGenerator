use crate::menu::{ListIngredient, Recette, Repas, Saison, Tag};
use crate::schedule::{Day, Week};
use chrono::{DateTime, Datelike, Local, TimeZone};
use eframe::emath::Numeric;
use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
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
    let mut config: Config =
        serde_json::from_str(&contenu_json).expect("Erreur : le format json ne correspond pas");
    config.normaliser();
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
/// Temps de cuisine d'un repas (plat + entrée + dessert). Un reste ne se recuisine pas.
fn duree_repas(repas: &Repas) -> i32 {
    let mut som = 0;
    for rec in [repas.recette.as_ref(), repas.entree.as_ref(), repas.dessert.as_ref()] {
        if let Some(r) = rec {
            if !r.est_reste {
                som += r.duree;
            }
        }
    }
    som
}
pub fn getSumDuration(week: &Week) -> i32 {
    let mut som: i32 = 0;
    for d in &week.days {
        for r in &d.menu {
            som += duree_repas(r);
        }
    }
    som
}

/// Partie d'un repas que l'on veut regénérer
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Partie {
    Plat,
    Entree,
    Dessert,
}

/// Un plat principal n'est ni une entrée ni un dessert
fn est_plat(r: &Recette) -> bool {
    r.nbPersonne > 0 && !r.tags.contains(&Tag::Dessert) && !r.tags.contains(&Tag::Entree)
}

fn a_tag(r: &Recette, tag: Tag) -> bool {
    r.nbPersonne > 0 && r.tags.contains(&tag)
}

fn recettes_actives(recettes: &Vec<Recette>, filtre_saison: bool) -> Vec<Recette> {
    recettes
        .iter()
        .filter(|r| r.activable && (!filtre_saison || r.is_valable()))
        .cloned()
        .collect()
}

pub fn generateMenu(week: &mut Week, recettes: &Vec<Recette>, config: &Config, dureeMax: i32) {
    let actives = recettes_actives(recettes, config.is_recette_filtre);

    apply_nb_mangeur(week, &config.list_nb_mangeur);

    let mut rng = rand::thread_rng();
    let mut meilleur: Option<Week> = None;
    let mut meilleure_duree = i32::MAX;
    let aucun_fige: HashSet<(usize, usize)> = HashSet::new();

    // On retire au sort jusqu'à respecter la durée max (100 essais), sinon on garde le plus court
    for _ in 0..100 {
        assigner_recettes(week, &actives, config, &mut rng, &aucun_fige, None, false);
        let duree = getSumDuration(week);
        if duree < meilleure_duree {
            meilleure_duree = duree;
            meilleur = Some(week.clone());
        }
        if duree <= dureeMax || !actives.iter().any(|r| est_plat(r)) {
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

/// Regénère une partie d'un repas (plat, entrée ou dessert) sans toucher aux autres repas.
/// `d` = index du jour, `r` = index du repas (0 petit-déj ... 3 dîner).
pub fn regenerer_repas(
    week: &mut Week,
    d: usize,
    r: usize,
    partie: Partie,
    recettes: &Vec<Recette>,
    config: &Config,
) {
    if d >= week.days.len() || r >= week.days[d].menu.len() {
        return;
    }
    let nb = week.days[d].menu[r].nbPersonne;
    if nb <= 0 {
        return;
    }
    let actives = recettes_actives(recettes, config.is_recette_filtre);
    let mut rng = rand::thread_rng();

    match partie {
        Partie::Entree | Partie::Dessert => {
            let tag = if partie == Partie::Entree {
                Tag::Entree
            } else {
                Tag::Dessert
            };
            let liste: Vec<&Recette> = actives.iter().filter(|x| a_tag(x, tag)).collect();
            let actuel: Option<String> = if partie == Partie::Entree {
                week.days[d].menu[r].entree.as_ref().map(|x| x.name.clone())
            } else {
                week.days[d].menu[r].dessert.as_ref().map(|x| x.name.clone())
            };
            if let Some(nouveau) = choisir_extra(&liste, nb, &mut rng, actuel.as_deref()) {
                if partie == Partie::Entree {
                    week.days[d].menu[r].entree = Some(nouveau);
                } else {
                    week.days[d].menu[r].dessert = Some(nouveau);
                }
            }
        }
        Partie::Plat => {
            let ancien: Option<Recette> = week.days[d].menu[r].recette.clone();

            // Tous les autres repas restent tels quels...
            let mut figes: HashSet<(usize, usize)> = HashSet::new();
            for dd in 0..week.days.len() {
                for rr in 0..week.days[dd].menu.len() {
                    if (dd, rr) != (d, r) {
                        figes.insert((dd, rr));
                    }
                }
            }
            // ... sauf les restes de l'ancien plat, qui n'existeraient plus : on les retire aussi
            if let Some(a) = &ancien {
                if !a.est_reste {
                    for dd in 0..week.days.len() {
                        for rr in 0..week.days[dd].menu.len() {
                            let apres = dd > d || (dd == d && rr > r);
                            let est_reste_de_a = week.days[dd].menu[rr]
                                .recette
                                .as_ref()
                                .map_or(false, |x| x.est_reste && x.name == a.name);
                            if apres && est_reste_de_a {
                                figes.remove(&(dd, rr));
                            }
                        }
                    }
                }
            }

            let nom_ancien: Option<String> = ancien.as_ref().map(|a| a.name.clone());
            let exclu = nom_ancien.as_deref().map(|n| (d, r, n));
            assigner_recettes(week, &actives, config, &mut rng, &figes, exclu, true);
        }
    }
}

/// Tire une entrée / un dessert au hasard (en évitant `exclu`) et l'adapte au nombre de mangeurs
fn choisir_extra(
    liste: &Vec<&Recette>,
    nb: i32,
    rng: &mut rand::rngs::ThreadRng,
    exclu: Option<&str>,
) -> Option<Recette> {
    let autres: Vec<&Recette> = liste
        .iter()
        .copied()
        .filter(|r| exclu.map_or(true, |n| r.name != n))
        .collect();
    let source: &Vec<&Recette> = if autres.is_empty() { liste } else { &autres };
    let base: &Recette = source.choose(rng).copied()?;
    let mut rec: Recette = base.clone();
    let mult = ((nb + rec.nbPersonne - 1) / rec.nbPersonne).max(1);
    rec.multNbRecette(mult);
    Some(rec)
}

/// Ajoute l'entrée / le dessert du repas (si les réglages en prévoient un)
fn remplir_extras(
    week: &mut Week,
    d: usize,
    r: usize,
    nb: i32,
    idx: usize,
    config: &Config,
    entrees: &Vec<&Recette>,
    desserts: &Vec<&Recette>,
    rng: &mut rand::rngs::ThreadRng,
    garder_extras: bool,
) {
    if config.entree_prevue(idx) && !(garder_extras && week.days[d].menu[r].entree.is_some()) {
        week.days[d].menu[r].entree = choisir_extra(entrees, nb, rng, None);
    }
    if config.dessert_prevu(idx) && !(garder_extras && week.days[d].menu[r].dessert.is_some()) {
        week.days[d].menu[r].dessert = choisir_extra(desserts, nb, rng, None);
    }
}

/// Remplit tous les repas de la semaine (un tirage complet).
/// - `figes` : repas à ne pas toucher (ils comptent quand même pour les restes)
/// - `exclu` : (jour, repas, nom) d'un plat à ne pas retirer pour ce repas
/// - `garder_extras` : conserve les entrées / desserts déjà présents
fn assigner_recettes(
    week: &mut Week,
    recettes: &Vec<Recette>,
    config: &Config,
    rng: &mut rand::rngs::ThreadRng,
    figes: &HashSet<(usize, usize)>,
    exclu: Option<(usize, usize, &str)>,
    garder_extras: bool,
) {
    let nb_jours = week.days.len();
    // Les plats principaux ne sont jamais des entrées ni des desserts
    let plats: Vec<&Recette> = recettes.iter().filter(|r| est_plat(r)).collect();
    let entrees: Vec<&Recette> = recettes.iter().filter(|r| a_tag(r, Tag::Entree)).collect();
    let desserts: Vec<&Recette> = recettes.iter().filter(|r| a_tag(r, Tag::Dessert)).collect();

    // Repas à pourvoir, dans l'ordre chronologique (et on vide les anciennes recettes)
    let mut ordre: Vec<(usize, usize)> = Vec::new();
    for d in 0..week.days.len() {
        for r in 0..week.days[d].menu.len() {
            if !figes.contains(&(d, r)) {
                let repas = &mut week.days[d].menu[r];
                repas.recette = None;
                if !garder_extras || repas.nbPersonne <= 0 {
                    repas.entree = None;
                    repas.dessert = None;
                }
            }
            if week.days[d].menu[r].nbPersonne > 0 {
                ordre.push((d, r));
            }
        }
    }

    // Restes disponibles : (recette, portions restantes, jour de cuisson)
    let mut restes: Vec<(Recette, i32, usize)> = Vec::new();

    for pos in 0..ordre.len() {
        let (d, r) = ordre[pos];
        let nb = week.days[d].menu[r].nbPersonne;
        let idx = Config::indice(r, d, nb_jours);
        let tag_force: Option<Tag> = config.tag_force(idx);

        restes.retain(|(_, portions, jour)| *portions > 0 && d - *jour <= MAX_JOURS_RESTES);

        // Repas figé : on garde son contenu, mais on tient les restes à jour
        if figes.contains(&(d, r)) {
            if let Some(rec) = week.days[d].menu[r].recette.clone() {
                if rec.est_reste {
                    if let Some(i) = restes
                        .iter()
                        .position(|(x, portions, _)| x.name == rec.name && *portions >= nb)
                    {
                        restes[i].1 -= nb;
                    }
                } else if rec.nbPersonne > nb {
                    restes.push((rec.clone(), rec.nbPersonne - nb, d));
                }
            }
            continue;
        }

        let tag_ok = |rec: &Recette| tag_force.map_or(true, |t| rec.tags.contains(&t));
        let est_exclu = |rec: &Recette| match exclu {
            Some((ed, er, nom)) => ed == d && er == r && rec.name == nom,
            None => false,
        };

        // 1. Y a-t-il des restes à manger ? (en respectant le tag forcé)
        if let Some(i) = restes
            .iter()
            .position(|(rec, portions, _)| *portions >= nb && tag_ok(rec) && !est_exclu(rec))
        {
            let mut rec = restes[i].0.clone();
            rec.est_reste = true;
            restes[i].1 -= nb;
            week.days[d].menu[r].recette = Some(rec);
            remplir_extras(
                week, d, r, nb, idx, config, &entrees, &desserts, rng, garder_extras,
            );
            continue;
        }

        // 2. Sinon on choisit un plat principal
        let candidats: Vec<&Recette> = plats
            .iter()
            .copied()
            .filter(|rec| {
                let rec: &Recette = *rec;
                if !tag_ok(rec) || est_exclu(rec) {
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

        let mut choix: Option<&Recette> = candidats.choose(rng).copied();
        if choix.is_none() {
            // repli 1 : n'importe quelle taille, mais on respecte le tag forcé
            let v: Vec<&Recette> = plats
                .iter()
                .copied()
                .filter(|x| tag_ok(*x) && !est_exclu(*x))
                .collect();
            choix = v.choose(rng).copied();
        }
        if choix.is_none() {
            // repli 2 : n'importe quel plat
            let v: Vec<&Recette> = plats.iter().copied().filter(|x| !est_exclu(*x)).collect();
            choix = v.choose(rng).copied();
        }
        if choix.is_none() {
            choix = plats.choose(rng).copied();
        }
        let Some(base) = choix else {
            continue; // aucun plat disponible
        };
        let mut rec: Recette = base.clone();

        if rec.nbPersonne <= nb {
            // multiplie la recette (arrondi au supérieur, jamais 0)
            let mult = ((nb + rec.nbPersonne - 1) / rec.nbPersonne).max(1);
            rec.multNbRecette(mult);
        } else {
            // recette cuisinée en entier, les portions en trop deviennent des restes
            restes.push((rec.clone(), rec.nbPersonne - nb, d));
        }
        week.days[d].menu[r].recette = Some(rec);
        remplir_extras(
            week, d, r, nb, idx, config, &entrees, &desserts, rng, garder_extras,
        );
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
    /// Un repas comprend-il une entrée ? (même indexation que `list_nb_mangeur`)
    #[serde(default)]
    pub list_entree: Vec<bool>,
    /// Un repas comprend-il un dessert ?
    #[serde(default)]
    pub list_dessert: Vec<bool>,
    /// Tag imposé au plat d'un repas (None = aucun)
    #[serde(default)]
    pub list_tag_force: Vec<Option<Tag>>,
}
impl Config {
    /// Index d'un repas dans les listes de réglages (repas `r`, jour `d`)
    pub fn indice(r: usize, d: usize, nb_jours: usize) -> usize {
        r * nb_jours + d
    }

    /// Met les listes entrée / dessert / tag à la même taille que `list_nb_mangeur`
    pub fn normaliser(&mut self) {
        let n = self.list_nb_mangeur.len();
        self.list_entree.resize(n, false);
        self.list_dessert.resize(n, false);
        self.list_tag_force.resize(n, None);
    }

    pub fn entree_prevue(&self, idx: usize) -> bool {
        self.list_entree.get(idx).copied().unwrap_or(false)
    }

    pub fn dessert_prevu(&self, idx: usize) -> bool {
        self.list_dessert.get(idx).copied().unwrap_or(false)
    }

    pub fn tag_force(&self, idx: usize) -> Option<Tag> {
        self.list_tag_force.get(idx).copied().flatten()
    }

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
    let nb_jours = week.days.len();
    for day in 0..nb_jours {
        for r in 0..week.days[day].menu.len() {
            let index = Config::indice(r, day, nb_jours);
            week.days[day].menu[r].nbPersonne = list_nb.get(index).copied().unwrap_or(0);
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
            for rec in [repas.recette.as_ref(), repas.entree.as_ref(), repas.dessert.as_ref()] {
                if let Some(r) = rec {
                    // les restes ont déjà été achetés avec le repas d'origine
                    if !r.est_reste {
                        res.add_list(&r.ingredients);
                    }
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
