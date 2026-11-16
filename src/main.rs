use chrono::Local;
use std::collections::HashMap;

mod fonctions;
mod interface;
mod menu;
mod schedule;

use crate::fonctions::{
    Config, charger_recettes, creer_liste_courses, loadConfig, loadWeeks, mettre_a_jour_semaines,
    saveWeeks,
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

        // Détecte le changement de semaine quand l'appli reste ouverte (génère la suivante)
        self.verifier_changement_semaine(ctx);

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
                Page::Edition => self.afficher_edition(ui),
            });
    }
}

fn charger_icone() -> Option<egui::IconData> {
    let image = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .ok()?
        .into_rgba8();
    let (largeur, hauteur) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width: largeur,
        height: hauteur,
    })
}

fn main() -> eframe::Result<()> {
    let mut config: Config = loadConfig("config.json");
    let recetteList: Vec<Recette> = charger_recettes("recettes.json");
    let today = Local::now();
    let tMax: i32 = config.duree_max;

    let mut semaines: Vec<Week> = loadWeeks("schedule.json");

    // Retire les semaines terminées et complète le planning (semaine en cours + suivante).
    // La même vérification est refaite pendant que l'appli tourne
    // (voir MonApp::verifier_changement_semaine).
    let (retirees, nouvelles) = mettre_a_jour_semaines(&mut semaines, &recetteList, &config, tMax);

    let code_avant = config.c_liste.clone();
    for nouvelle in &nouvelles {
        // ENVOYE LES INGREDIENTS DANS LA LISTE DE COURSES (seulement si l'envoi auto est activé)
        //      Si le code de la room n'existe pas, on en demande un au serveur
        if config.auto_send_sl {
            if config.c_liste.len() == 0 {
                match config.ensure_room_code() {
                    Ok(code) => println!("[ROOM] Code salon actif : {}", code),
                    Err(e) => eprintln!("[ROOM ERREUR] {}", e),
                }
            }
            match config.send_ingredients_to_room(&creer_liste_courses(nouvelle)) {
                Ok(()) => println!("[WEBSOCKET] Ingrédients envoyés au salon !"),
                Err(e) => eprintln!("[WEBSOCKET ERREUR] {}", e),
            }
        }
    }
    // Un nouveau code de salon vient peut-être d'être généré : on le garde
    if config.c_liste != code_avant {
        config.save("config.json");
    }
    if retirees > 0 || !nouvelles.is_empty() {
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
        filtres_tags: std::collections::HashSet::new(),
        edition: None,
        filtre_duree_actif: false,
        filtre_duree_max: 30,
        hauteurs_recettes: HashMap::new(),
        cache_recettes: HashMap::new(),
        options_repas: None,
        apercu: None,
        message_apercu: None,
        recherche: String::new(),
        suppression_en_attente: None,
    };
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1100.0, 720.0])
        .with_min_inner_size([760.0, 480.0]);
    if let Some(icone) = charger_icone() {
        viewport = viewport.with_icon(icone);
    }
    let options = eframe::NativeOptions {
        viewport,
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
