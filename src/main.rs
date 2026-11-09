use chrono::Timelike;
use chrono::{Duration, Local};
use rand::{Rng, seq::SliceRandom};
use std::collections::HashMap;
use std::fs;

mod fonctions;
mod interface;
mod menu;
mod schedule;

use interface::{MonApp, Page};
use menu::{ListIngredient, Recette, Repas};
use schedule::{Day, Week};

use crate::fonctions::{
    Config, apply_nb_mangeur, charger_recettes, creer_liste_courses, generateMenu, loadConfig,
    loadWeeks, saveWeeks, showWeek,
};
use crate::fonctions::{getRepasByMaxDuration, sauvegarder_recettes};
use crate::schedule::getNextMonday;
use eframe::egui;
use std::sync::Mutex;

// 1. Tes données d'état

// 2. L'implémentation de la boucle d'affichage
impl eframe::App for MonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Local::now();
        let mut semaine_act: Week = Week {
            days: vec![],
            tMax: 200,
        };
        if let Some(semaine_active) = self.semaines.first() {
            semaine_act = semaine_active.clone();
        } else {
            semaine_act.generate_this_week();
        }
        let d: Day = Day {
            menu: vec![],
            date: Local::now(),
        };
        let (aujd, recette) = if let Some(first_week) = self.semaines.first() {
            if let Some(jour) = first_week.getDayByDate(now) {
                let rec_txt = if now.hour() > 16 {
                    match &jour.menu[3].recette {
                        Some(r) => r.display(),
                        None => "Aucune recette".to_string(),
                    }
                } else if now.hour() > 10 {
                    match &jour.menu[1].recette {
                        Some(r) => r.display(),
                        None => "Aucune recette".to_string(),
                    }
                } else {
                    match &jour.menu[0].recette {
                        Some(r) => r.display(),
                        None => "Aucune recette".to_string(),
                    }
                };

                (jour.clone(), rec_txt)
            } else {
                (d.clone(), "Aucune recette".to_string())
            }
        } else {
            (d.clone(), "Aucune recette".to_string())
        };
        // C'est ici que toute l'interface est construite à chaque frame
        egui::CentralPanel::default().show(ctx, |ui| match self.page {
            Page::Acceuil => {
                // AFFICHAGE DU MENU DU JOUR

                ui.separator();
                ui.label("\n");

                ui.label(aujd.date.format("%a %d %b %Y").to_string());
                for i in 0..aujd.menu.len() {
                    ui.label(aujd.menu[i].display());
                    if let Some(r) = &aujd.menu[i].recette {
                        if ui.button("Afficher la recette").clicked() {
                            self.recette_courante = aujd.menu[i].recette.clone();
                            self.page_precedente = Page::Acceuil;
                            self.page = Page::Recette;
                        }
                    };
                }
            }
            Page::Recette => {
                ui.separator();
                ui.label("\n");
                if let Some(r) = &self.recette_courante {
                    ui.label(r.affichage_complet());
                }
            }
            Page::Week => {
                ui.label("\n");
                ui.label("Planning de la Semaine");
                ui.separator();

                if let Some(semaine_active) = self.semaines.get(self.id_week).cloned() {
                    self.afficher_grille_semaine(ui, &semaine_active);
                } else {
                    self.afficher_grille_semaine(ui, &semaine_act);
                }
            }
            Page::Config => {
                ui.label("");
                if let Some(semaine_active) = self.semaines.get(self.id_week).cloned() {
                    self.afficher_grille_semaine(ui, &semaine_active);
                } else {
                    self.afficher_grille_semaine(ui, &semaine_act);
                }
                ui.label("Durée maximal de temps de cuisine durant les prochaines semaines :");
                ui.text_edit_singleline(&mut self.duree_max_config);
                if ui.button("regenerer la prochaine semaine").clicked() {
                    generateMenu(
                        &mut self.semaines[1],
                        &self.recettes,
                        &self.config.list_nb_mangeur,
                        self.duree_max_config
                            .parse()
                            .unwrap_or(self.config.duree_max), // A CHANGER -----------------------------------------------------------
                        self.config.is_recette_filtre,
                    );
                }
                let mut case: String = "         ".to_string();
                ui.horizontal(|ui| {
                    ui.label("Envoie du menu dans la liste auto : ");
                    if ui
                        .add(match self.config.auto_send_sl {
                            true => egui::Button::new(case.clone())
                                .fill(egui::Color32::from_rgb(0, 255, 0)),
                            false => egui::Button::new(case.clone())
                                .fill(egui::Color32::from_rgb(255, 0, 0)),
                        })
                        .clicked()
                    {
                        self.config.auto_send_sl = !self.config.auto_send_sl;
                    }
                });

                if self.config.auto_send_sl {
                    ui.horizontal(|ui| {
                        if self.config.c_liste.len() == 6 {
                            ui.label(format!("code : {}", self.config.c_liste.clone()));
                            case = "REGENERER UN CODE".to_string();
                        } else {
                            case = "GENERER UN CODE".to_string();
                        }
                        if ui.button(case.clone()).clicked() {
                            self.config.ensure_room_code();
                        }
                    });
                }

                ui.horizontal(|ui| {
                    ui.label("Désactivation automatique des recettes hors saison : ");
                    if ui
                        .add(match self.config.is_recette_filtre {
                            true => egui::Button::new("        ")
                                .fill(egui::Color32::from_rgb(0, 255, 0)),
                            false => egui::Button::new("        ")
                                .fill(egui::Color32::from_rgb(255, 0, 0)),
                        })
                        .clicked()
                    {
                        self.config.is_recette_filtre = !self.config.is_recette_filtre;
                    }
                });
            }
            Page::Recettes => {
                ui.label("\n\n");
                for r in 0..self.recettes.len() {
                    ui.horizontal(|ui| {
                        if ui
                            .add(match self.recettes[r].activable {
                                true => egui::Button::new("     \n      ")
                                    .fill(egui::Color32::from_rgb(0, 255, 0)),
                                false => egui::Button::new("     \n      ")
                                    .fill(egui::Color32::from_rgb(255, 0, 0)),
                            })
                            .clicked()
                        {
                            self.recettes[r].activable = !self.recettes[r].activable;
                        }
                        ui.label(self.recettes[r].display().to_string());
                    });
                }
            }
        });
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| match self.page {
            Page::Acceuil => {
                ui.horizontal(|ui| {
                    // BOUTON MENU DE LA SEMAINE
                    if ui.button("MENU DE LA SEMAINE").clicked() {
                        self.id_week = 0;
                        self.page = Page::Week;
                    }
                    if ui.button("CONFIGURATION").clicked() {
                        self.page = Page::Config;
                    }
                    if ui.button("RECETTES").clicked() {
                        self.page = Page::Recettes
                    }
                });
            }
            Page::Recette => {
                ui.horizontal(|ui| {
                    if ui.button("RETOUR").clicked() {
                        self.recette_courante = None;
                        self.page = self.page_precedente.clone();
                    }
                });
            }
            Page::Week => {
                ui.horizontal(|ui| {
                    if ui.button("RETOUR").clicked() {
                        self.page = Page::Acceuil;
                    }
                    if self.id_week + 1 < self.semaines.len() && ui.button("->").clicked() {
                        if self.semaines.len() > self.id_week + 1 {
                            self.id_week += 1;
                        }
                    }
                    if self.id_week > 0 && ui.button("<-").clicked() {
                        if 0 < self.id_week {
                            self.id_week -= 1;
                        }
                    }
                });
            }
            Page::Config => {
                ui.horizontal(|ui| {
                    if ui.button("RETOUR").clicked() {
                        self.page = Page::Acceuil;
                    }
                });
            }
            Page::Recettes => {
                ui.horizontal(|ui| {
                    if ui.button("RETOUR").clicked() {
                        self.page = Page::Acceuil;
                    }
                });
            }
        });
        egui::TopBottomPanel::bottom("mon_footer").show(ctx, |ui| match self.page {
            Page::Acceuil => {}
            Page::Recette => {}
            Page::Week => {}
            Page::Config => {
                if ui.button("SAVE").clicked() {
                    self.config.duree_max = self
                        .duree_max_config
                        .parse()
                        .unwrap_or(self.config.duree_max);
                    self.config.save("config.json");
                }
            }
            Page::Recettes => {
                if ui.button("SAVE").clicked() {
                    self.sauvegarder_recettes();
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let mut config: Config = loadConfig("config.json");
    let mut recetteList: Vec<Recette> = charger_recettes("recettes.json");
    let today = Local::now();
    let tMax: i32 = 200;

    let mut semaines: Vec<Week> = loadWeeks("schedule.json");
    //let mut semaine: Week = semaines[0].clone();
    /*Week {
        days: vec![],
        tMax: 200,
    };
    */

    while !semaines.is_empty() && today > semaines[0].days[6].date {
        print!("Suppression d'une semaine périmée...");
        semaines.remove(0);
        saveWeeks("schedule.json", &semaines);
    }
    /*
    if let Some(premiere) = semaines.first() {
        semaine = premiere.clone();
    }
    */

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
        // ENVOYE LES INGREDIENTS DANS LA LISTE DE COURSES
        //      Si le code de la room n'existe pas, on en demande un au serveur
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

        saveWeeks("schedule.json", &semaines);
    }
    //showWeek(&semaine);
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
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1400.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Planificateur de Menus",
        options,
        Box::new(|_cc| Box::new(app)),
    )
}
