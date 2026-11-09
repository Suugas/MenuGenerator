use eframe::egui;
use eframe::egui::accesskit::Checked::False;
use eframe::egui::accesskit::Role::Switch;
use std::collections::HashMap;

use crate::fonctions::{Config, add_value, sauvegarder_recettes};
use crate::menu::{Recette, Repas};
use crate::schedule::Week;

#[derive(Clone, Debug)]
pub enum Page {
    Acceuil,
    Recette,
    Week,
    Config,
    Recettes,
}

pub struct MonApp {
    pub config: Config,
    pub recettes: Vec<Recette>,
    pub semaines: Vec<Week>,
    pub button_display_recette: bool,
    pub is_button_pressed: HashMap<String, bool>,
    pub page: Page,
    pub page_precedente: Page,
    pub recette_courante: Option<Recette>,
    pub id_week: usize,
    pub duree_max_config: String,
}
impl MonApp {
    /// Affiche une semaine complète sous forme de grille / tableau
    pub fn afficher_grille_semaine(&mut self, ui: &mut egui::Ui, week: &Week) {
        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("grille_semaine")
                .striped(true) // Alterne les couleurs des lignes pour plus de lisibilité
                .spacing([12.0, 8.0]) // Espace entre les colonnes et les lignes
                .show(ui, |ui| {
                    // --- EN-TÊTE DE LA GRILLE ---
                    ui.heading("Repas");
                    for day in &week.days {
                        ui.heading(day.date.format("%a %d").to_string());
                    }
                    ui.end_row(); // Passé à la ligne suivante !

                    // --- CONTENU POUR CHAQUE JOUR DE LA SEMAINE --
                    let mut rendu_tableau =
                        |ui: &mut egui::Ui, i: usize, day: usize, week: &Week| match self.page {
                            Page::Week => {
                                let repas = week.days[day].menu[i].clone();
                                if let Some(ref recette) = repas.recette {
                                    if ui.button(&recette.name).clicked() {
                                        self.recette_courante = Some(recette.clone());
                                        self.page_precedente = self.page.clone();
                                        self.page = Page::Recette;
                                    }
                                } else {
                                    ui.label("---");
                                }
                            }
                            Page::Config => {
                                ui.horizontal(|ui| {
                                    let index = i * week.days.len() + day;
                                    if ui.button("-").clicked() {
                                        add_value(
                                            &mut self.config.list_nb_mangeur
                                                [i * (&week.days.len()) + day],
                                            -1,
                                        );
                                    }
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "   {}   ",
                                                    self.config.list_nb_mangeur[index]
                                                ))
                                                .color(egui::Color32::BLACK), // Texte en noir
                                            )
                                            .fill(
                                                match self.config.list_nb_mangeur
                                                    [i * week.days.len() + day]
                                                    > 0
                                                {
                                                    true => egui::Color32::from_rgb(0, 255, 0),
                                                    false => egui::Color32::from_rgb(255, 0, 0),
                                                },
                                            ),
                                        )
                                        .clicked()
                                    {}
                                    if ui.button("+").clicked() {
                                        add_value(
                                            &mut self.config.list_nb_mangeur
                                                [i * (&week.days.len()) + day],
                                            1,
                                        );
                                    }
                                });
                            }
                            _ => {}
                        };
                    for i in 0..=3 {
                        if i == 0 {
                            ui.label("Petit-dej");
                        } else if i == 1 {
                            ui.label("Dej");
                        } else if i == 2 {
                            ui.label("Gouté");
                        } else {
                            ui.label("Diné");
                        }
                        for day in 0..&week.days.len() + 0 {
                            rendu_tableau(ui, i, day, week);
                        }
                        ui.end_row();
                    }
                });
        });
    }

    pub fn sauvegarder_recettes(&self) {
        if let Err(e) = sauvegarder_recettes("recettes.json", &self.recettes) {
            eprintln!("[ERREUR] Sauvegarde des recettes impossible : {}", e);
        }
    }
}
