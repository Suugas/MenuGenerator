use chrono::Local;
use std::collections::HashMap;

mod fonctions;
mod interface;
mod menu;
mod schedule;

use crate::fonctions::{
    Config, charger_recettes, creer_liste_courses, generateMenu, loadConfig, loadWeeks, saveWeeks,
};
use eframe::egui;
use interface::{MonApp, Page, appliquer_theme, cadre_barre, cadre_central};
use menu::Recette;
use schedule::{Day, Week};

// L'implémentation de la boucle d'affichage
impl eframe::App for MonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Récupère le résultat d'un éventuel envoi manuel des ingrédients
        self.verifier_envoi(ctx);

        let now = Local::now();

        // Menu du jour (jour vide si la date du jour n'est dans aucune semaine)
        let aujd: Day = match self.semaines.first().and_then(|w| w.getDayByDate(now)) {
            Some(jour) => jour.clone(),
            None => Day {
                menu: vec![],
                date: Local::now(),
            },
        };

        // IMPORTANT : dans egui, les barres (haut / bas) doivent être ajoutées
        // AVANT le CentralPanel, sinon elles recouvrent le contenu.
        egui::TopBottomPanel::top("top_panel")
            .frame(cadre_barre())
            .show(ctx, |ui| {
                self.barre_navigation(ui);
            });

        if matches!(self.page, Page::Config | Page::Recettes) {
            egui::TopBottomPanel::bottom("mon_footer")
                .frame(cadre_barre())
                .show(ctx, |ui| {
                    self.pied_de_page(ui);
                });
        }

        egui::CentralPanel::default()
            .frame(cadre_central())
            .show(ctx, |ui| match self.page {
                Page::Acceuil => self.afficher_accueil(ui, &aujd),
                Page::Recette => self.afficher_recette(ui),
                Page::Week => {
                    let semaine = self.semaine_affichee();
                    self.afficher_semaine(ui, &semaine);
                }
                Page::Config => {
                    let semaine = self.semaine_affichee();
                    self.afficher_config(ui, &semaine);
                }
                Page::Recettes => self.afficher_recettes(ui),
            });
    }
}

fn main() -> eframe::Result<()> {
    let mut config: Config = loadConfig("config.json");
    let recetteList: Vec<Recette> = charger_recettes("recettes.json");
    let today = Local::now();
    let tMax: i32 = 200;

    let mut semaines: Vec<Week> = loadWeeks("schedule.json");

    while !semaines.is_empty() && today > semaines[0].days[6].date {
        print!("Suppression d'une semaine périmée...");
        semaines.remove(0);
        saveWeeks("schedule.json", &semaines);
    }

    while semaines.len() <= 1 {
        if semaines.len() == 0 {
            println!("on genere la semaine actuel vide");
            let mut n_semaine_len_0: Week = Week {
                days: vec![],
                tMax: tMax,
            };
            n_semaine_len_0.generate_this_week();
            semaines.push(n_semaine_len_0);
        }
        let mut n_semaine: Week = Week {
            days: vec![],
            tMax: tMax,
        };
        n_semaine.generate_from_weeks(&semaines);
        generateMenu(
            &mut n_semaine,
            &recetteList,
            &config.list_nb_mangeur,
            tMax,
            config.is_recette_filtre,
        );
        semaines.push(n_semaine.clone());
        // ENVOYE LES INGREDIENTS DANS LA LISTE DE COURSES (seulement si l'envoi auto est activé)
        //      Si le code de la room n'existe pas, on en demande un au serveur
        if config.auto_send_sl {
            if config.c_liste.len() == 0 {
                match config.ensure_room_code() {
                    Ok(code) => println!("[ROOM] Code salon actif : {}", code),
                    Err(e) => eprintln!("[ROOM ERREUR] {}", e),
                }
            }
            match config.send_ingredients_to_room(&creer_liste_courses(&n_semaine)) {
                Ok(()) => println!("[WEBSOCKET] Ingrédients envoyés au salon !"),
                Err(e) => eprintln!("[WEBSOCKET ERREUR] {}", e),
            }
        }

        saveWeeks("schedule.json", &semaines);
    }
    let date_str = today.format("%Y-%m-%d").to_string();

    println!("===== FIN DU PROGRAMME =====");

    let app = MonApp {
        config: config,
        recettes: recetteList,
        semaines: semaines,
        button_display_recette: false,
        is_button_pressed: HashMap::from([("display recette".to_string(), false)]),
        page: Page::Acceuil,
        page_precedente: Page::Acceuil,
        recette_courante: None,
        id_week: 0,
        duree_max_config: "200".to_string(),
        envoi_en_cours: None,
        message_envoi: None,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([760.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Planificateur de Menus",
        options,
        Box::new(move |cc| {
            appliquer_theme(&cc.egui_ctx);
            Box::new(app)
        }),
    )
}
