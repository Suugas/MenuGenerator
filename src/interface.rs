use chrono::{DateTime, Local, Timelike};
use eframe::egui;
use eframe::egui::{Color32, RichText, Stroke};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, TryRecvError};

use crate::fonctions::{
    Config, add_value, creer_liste_courses, generateMenu, getSumDuration, sauvegarder_recettes,
};
use crate::menu::{ListIngredient, Recette};
use crate::schedule::{Day, Week};

// =====================================================================
//  PALETTE
// =====================================================================
pub const BG: Color32 = Color32::from_rgb(21, 23, 28);
pub const PANEL: Color32 = Color32::from_rgb(29, 32, 39);
pub const CARD: Color32 = Color32::from_rgb(39, 43, 53);
pub const CARD_HOVER: Color32 = Color32::from_rgb(54, 59, 72);
pub const ACCENT: Color32 = Color32::from_rgb(255, 170, 64);
pub const SUR_ACCENT: Color32 = Color32::from_rgb(40, 24, 4); // texte sur fond orange
pub const OK: Color32 = Color32::from_rgb(88, 196, 130);
pub const SUR_OK: Color32 = Color32::from_rgb(10, 40, 25); // texte sur fond vert
pub const KO: Color32 = Color32::from_rgb(226, 98, 98);
pub const TEXTE: Color32 = Color32::from_rgb(232, 235, 240);
pub const TEXTE_DISCRET: Color32 = Color32::from_rgb(172, 179, 192);

const REPAS_NOMS: [&str; 4] = ["Petit-déj", "Déjeuner", "Goûter", "Dîner"];
const REPAS_COULEURS: [Color32; 4] = [
    Color32::from_rgb(255, 196, 92),  // petit-déj
    Color32::from_rgb(120, 200, 140), // déjeuner
    Color32::from_rgb(240, 140, 170), // goûter
    Color32::from_rgb(130, 160, 245), // dîner
];

/// A appeler une seule fois au démarrage : applique les couleurs, arrondis et tailles de texte.
pub fn appliquer_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    let mut v = egui::Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = PANEL;
    v.extreme_bg_color = Color32::from_rgb(15, 17, 21);
    v.faint_bg_color = Color32::from_rgb(34, 37, 45);
    v.window_rounding = egui::Rounding::same(12.0);

    let arrondi = egui::Rounding::same(8.0);

    v.widgets.noninteractive.bg_fill = PANEL;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(52, 56, 68));
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXTE);
    v.widgets.noninteractive.rounding = arrondi;

    v.widgets.inactive.bg_fill = CARD;
    v.widgets.inactive.weak_bg_fill = CARD;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(58, 63, 77));
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXTE);
    v.widgets.inactive.rounding = arrondi;

    v.widgets.hovered.bg_fill = CARD_HOVER;
    v.widgets.hovered.weak_bg_fill = CARD_HOVER;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    v.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    v.widgets.hovered.rounding = arrondi;

    // ATTENTION : egui utilise aussi fg_stroke de "active" comme couleur de tous les
    // textes .strong(). Il doit donc rester clair (sinon les titres deviennent sombres).
    v.widgets.active.bg_fill = Color32::from_rgb(190, 120, 35);
    v.widgets.active.weak_bg_fill = Color32::from_rgb(190, 120, 35);
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    v.widgets.active.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    v.widgets.active.rounding = arrondi;

    v.widgets.open.rounding = arrondi;

    v.selection.bg_fill = ACCENT.linear_multiply(0.30);
    v.selection.stroke = Stroke::new(1.0, ACCENT);

    style.visuals = v;

    use egui::{FontFamily, FontId, TextStyle};
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::new(24.0, FontFamily::Proportional));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(15.0, FontFamily::Proportional));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::new(14.5, FontFamily::Proportional));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::new(12.0, FontFamily::Proportional));

    style.spacing.item_spacing = egui::vec2(8.0, 7.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.interact_size.y = 26.0;

    ctx.set_style(style);
}

/// Cadre des barres du haut / du bas
pub fn cadre_barre() -> egui::Frame {
    egui::Frame::none()
        .fill(PANEL)
        .inner_margin(egui::Margin::symmetric(16.0, 9.0))
}

/// Cadre de la zone centrale
pub fn cadre_central() -> egui::Frame {
    egui::Frame::none()
        .fill(BG)
        .inner_margin(egui::Margin::symmetric(20.0, 10.0))
}

// =====================================================================
//  PETITS COMPOSANTS
// =====================================================================

/// Carte à coins arrondis (fond de couleur + marge intérieure)
fn carte(fond: Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(fond)
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::same(12.0))
}

/// Petite pastille d'information ("4 personnes", "25 min", ...)
fn pastille(ui: &mut egui::Ui, texte: &str, couleur: Color32) {
    egui::Frame::none()
        .fill(couleur.linear_multiply(0.18))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::symmetric(8.0, 2.0))
        .show(ui, |ui| {
            ui.label(RichText::new(texte).size(12.5).color(couleur));
        });
}

/// Interrupteur ON / OFF. Retourne true si la valeur a changé.
fn interrupteur(ui: &mut egui::Ui, actif: &mut bool) -> bool {
    let (texte, fond, couleur_texte) = if *actif {
        ("ON", OK, SUR_OK)
    } else {
        ("OFF", Color32::from_rgb(70, 74, 86), TEXTE_DISCRET)
    };
    let reponse = ui.add_sized(
        [46.0, 24.0],
        egui::Button::new(RichText::new(texte).strong().color(couleur_texte)).fill(fond),
    );
    if reponse.clicked() {
        *actif = !*actif;
        return true;
    }
    false
}

// =====================================================================
//  OUTILS DE FORMATAGE (dates en français, ingrédients, étapes)
// =====================================================================
const JOURS: [&str; 7] = [
    "Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi", "Dimanche",
];
const JOURS_COURTS: [&str; 7] = ["Lun", "Mar", "Mer", "Jeu", "Ven", "Sam", "Dim"];
const MOIS: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];
const MOIS_COURTS: [&str; 12] = [
    "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.",
    "déc.",
];

fn nombre(date: &DateTime<Local>, format: &str) -> usize {
    date.format(format).to_string().parse::<usize>().unwrap_or(1)
}

fn index_jour(date: &DateTime<Local>) -> usize {
    nombre(date, "%u").saturating_sub(1).min(6) // %u : lundi = 1 ... dimanche = 7
}

fn index_mois(date: &DateTime<Local>) -> usize {
    nombre(date, "%m").saturating_sub(1).min(11)
}

fn jour_court(date: &DateTime<Local>) -> &'static str {
    JOURS_COURTS[index_jour(date)]
}

fn date_longue(date: &DateTime<Local>) -> String {
    format!(
        "{} {} {} {}",
        JOURS[index_jour(date)],
        nombre(date, "%d"),
        MOIS[index_mois(date)],
        date.format("%Y")
    )
}

fn date_courte(date: &DateTime<Local>) -> String {
    format!("{} {}", nombre(date, "%d"), MOIS_COURTS[index_mois(date)])
}

fn est_aujourdhui(date: &DateTime<Local>) -> bool {
    date.format("%Y-%m-%d").to_string() == Local::now().format("%Y-%m-%d").to_string()
}

/// "Pâtes (g)" -> ("Pâtes", "g")
fn nom_et_unite(nom: &str) -> (String, String) {
    let nom = nom.trim();
    if nom.ends_with(')') {
        if let Some(pos) = nom.rfind(" (") {
            let base = nom[..pos].trim().to_string();
            let unite = nom[pos + 2..nom.len() - 1].trim().to_string();
            return (base, unite);
        }
    }
    (nom.to_string(), String::new())
}

/// Ingrédients triés par nom : (nom, "200 g")
fn ingredients_tries(liste: &ListIngredient) -> Vec<(String, String)> {
    let mut v: Vec<(String, i32)> = liste
        .ingredients
        .iter()
        .map(|(nom, element)| (nom.clone(), element.quantite))
        .collect();
    v.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    v.into_iter()
        .map(|(nom, quantite)| {
            let (base, unite) = nom_et_unite(&nom);
            let texte = if unite.is_empty() {
                format!("{}", quantite)
            } else {
                format!("{} {}", quantite, unite)
            };
            (base, texte)
        })
        .collect()
}

/// "1 personne" / "3 personnes"
fn personnes(n: i32) -> String {
    if n > 1 {
        format!("{} personnes", n)
    } else {
        format!("{} personne", n)
    }
}

/// Enlève un éventuel numéro de début d'étape ("2. Faire cuire" -> "Faire cuire")
fn retirer_numero(texte: &str) -> &str {
    let sans_chiffres = texte.trim_start_matches(|c: char| c.is_ascii_digit());
    if sans_chiffres.len() < texte.len() {
        if let Some(reste) = sans_chiffres
            .strip_prefix('.')
            .or_else(|| sans_chiffres.strip_prefix(')'))
        {
            // "2. Faire cuire" oui, mais "2.5 L d'eau" non
            if reste.starts_with(|c: char| c.is_whitespace()) {
                return reste.trim_start();
            }
        }
    }
    texte
}

/// Découpe le texte de la recette en étapes.
/// - Si le texte contient des lignes vides : une étape = un bloc séparé par une ligne vide
///   (une étape peut donc s'écrire sur plusieurs lignes).
/// - Sinon (ancien format) : une étape par ligne.
fn etapes(texte: &str) -> Vec<String> {
    let propre = texte.replace("\r\n", "\n");
    let lignes: Vec<&str> = propre.trim().lines().map(|l| l.trim()).collect();

    let separe_par_lignes_vides = lignes.iter().any(|l| l.is_empty());

    let blocs: Vec<String> = if separe_par_lignes_vides {
        let mut blocs: Vec<String> = Vec::new();
        let mut courant: Vec<&str> = Vec::new();
        for ligne in lignes {
            if ligne.is_empty() {
                if !courant.is_empty() {
                    blocs.push(courant.join("\n"));
                    courant.clear();
                }
            } else {
                courant.push(ligne);
            }
        }
        if !courant.is_empty() {
            blocs.push(courant.join("\n"));
        }
        blocs
    } else {
        lignes.into_iter().map(|l| l.to_string()).collect()
    };

    blocs
        .into_iter()
        .map(|bloc| retirer_numero(&bloc).to_string())
        .collect()
}

fn tronquer(texte: &str, max: usize) -> String {
    if texte.chars().count() <= max {
        texte.to_string()
    } else {
        let mut s: String = texte.chars().take(max.saturating_sub(1)).collect();
        s.push('…');
        s
    }
}

// =====================================================================
//  ETAT DE L'APPLICATION
// =====================================================================
#[derive(Clone, Debug, PartialEq)]
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
    /// Envoi manuel des ingrédients en cours (thread d'arrière-plan)
    pub envoi_en_cours: Option<Receiver<Result<String, String>>>,
    /// Dernier message d'envoi : (texte, succès ?)
    pub message_envoi: Option<(String, bool)>,
}

impl MonApp {
    /// Semaine sélectionnée (ou une semaine vide si aucune n'existe)
    pub fn semaine_affichee(&self) -> Week {
        if let Some(semaine) = self.semaines.get(self.id_week) {
            semaine.clone()
        } else {
            let mut semaine = Week {
                days: vec![],
                tMax: 200,
            };
            semaine.generate_this_week();
            semaine
        }
    }

    // -----------------------------------------------------------------
    //  ENVOI MANUEL DES INGREDIENTS (liste de courses)
    // -----------------------------------------------------------------
    /// Lance l'envoi des ingrédients de `semaine` dans un thread (l'interface ne gèle pas)
    pub fn envoyer_ingredients(&mut self, semaine: &Week) {
        let mut config = self.config.clone();
        let liste = creer_liste_courses(semaine);
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let resultat = match config.send_ingredients_to_room(&liste) {
                Ok(()) => Ok(config.c_liste.clone()),
                Err(e) => Err(e.to_string()),
            };
            let _ = tx.send(resultat);
        });

        self.envoi_en_cours = Some(rx);
        self.message_envoi = None;
    }

    /// A appeler à chaque frame : récupère le résultat de l'envoi quand il est terminé
    pub fn verifier_envoi(&mut self, ctx: &egui::Context) {
        let Some(resultat) = self.envoi_en_cours.as_ref().map(|rx| rx.try_recv()) else {
            return;
        };

        match resultat {
            Ok(Ok(code)) => {
                // Si un nouveau code de salon a été généré, on le garde
                if code != self.config.c_liste {
                    self.config.c_liste = code;
                    self.config.save("config.json");
                }
                self.message_envoi = Some((
                    "Ingrédients envoyés dans la liste de courses !".to_string(),
                    true,
                ));
                self.envoi_en_cours = None;
            }
            Ok(Err(erreur)) => {
                self.message_envoi = Some((format!("Échec de l'envoi : {}", erreur), false));
                self.envoi_en_cours = None;
            }
            Err(TryRecvError::Empty) => {
                // Toujours en cours : on redemande une frame bientôt
                ctx.request_repaint_after(std::time::Duration::from_millis(200));
            }
            Err(TryRecvError::Disconnected) => {
                self.message_envoi = Some(("L'envoi a été interrompu.".to_string(), false));
                self.envoi_en_cours = None;
            }
        }
    }

    // -----------------------------------------------------------------
    //  BARRES
    // -----------------------------------------------------------------
    pub fn barre_navigation(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Planificateur de Menus")
                    .size(18.0)
                    .strong()
                    .color(ACCENT),
            );
            ui.add_space(18.0);

            if matches!(self.page, Page::Recette) {
                if ui.button("< Retour").clicked() {
                    self.recette_courante = None;
                    self.page = self.page_precedente.clone();
                }
            } else {
                let onglets = [
                    ("Aujourd'hui", Page::Acceuil),
                    ("Semaine", Page::Week),
                    ("Recettes", Page::Recettes),
                    ("Réglages", Page::Config),
                ];
                for (nom, page) in onglets {
                    let actif = self.page == page;
                    if ui
                        .selectable_label(actif, RichText::new(nom).size(15.0))
                        .clicked()
                        && !actif
                    {
                        if page == Page::Week {
                            self.id_week = 0;
                        }
                        self.page = page;
                    }
                }
            }
        });
    }

    pub fn pied_de_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let bouton = egui::Button::new(
                RichText::new("Enregistrer").strong().color(SUR_ACCENT),
            )
            .fill(ACCENT);
            if ui.add(bouton).clicked() {
                match self.page {
                    Page::Config => {
                        self.config.duree_max = self
                            .duree_max_config
                            .parse()
                            .unwrap_or(self.config.duree_max);
                        self.config.save("config.json");
                    }
                    Page::Recettes => self.sauvegarder_recettes(),
                    _ => {}
                }
            }
        });
    }

    // -----------------------------------------------------------------
    //  PAGE : AUJOURD'HUI
    // -----------------------------------------------------------------
    pub fn afficher_accueil(&mut self, ui: &mut egui::Ui, aujd: &Day) {
        let heure = Local::now().hour();
        let repas_courant = if heure > 16 {
            3
        } else if heure > 10 {
            1
        } else {
            0
        };
        let mut a_ouvrir: Option<Recette> = None;

        ui.add_space(4.0);
        ui.label(RichText::new("Menu du jour").size(13.0).color(TEXTE_DISCRET));
        ui.label(RichText::new(date_longue(&aujd.date)).size(28.0).strong());
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            if aujd.menu.is_empty() {
                ui.label(RichText::new("Aucun menu prévu aujourd'hui.").color(TEXTE_DISCRET));
            }

            for (i, repas) in aujd.menu.iter().enumerate() {
                let k = i.min(3);
                let couleur = REPAS_COULEURS[k];
                let courant = i == repas_courant;
                let bordure = if courant {
                    Stroke::new(1.5, couleur)
                } else {
                    Stroke::new(0.0, Color32::TRANSPARENT)
                };

                carte(CARD).stroke(bordure).show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(REPAS_NOMS[k]).strong().color(couleur));
                        if courant {
                            pastille(ui, "maintenant", couleur);
                        }
                    });

                    match &repas.recette {
                        Some(r) => {
                            ui.label(RichText::new(&r.name).size(20.0).strong());
                            ui.horizontal(|ui| {
                                pastille(
                                    ui,
                                    &personnes(repas.nbPersonne),
                                    TEXTE_DISCRET,
                                );
                                pastille(ui, &format!("{} min", r.duree), TEXTE_DISCRET);
                            });
                            ui.add_space(2.0);
                            let bouton = egui::Button::new(
                                RichText::new("Voir la recette").strong().color(SUR_ACCENT),
                            )
                            .fill(couleur);
                            if ui.add(bouton).clicked() {
                                a_ouvrir = Some(r.clone());
                            }
                        }
                        None => {
                            ui.label(
                                RichText::new("Pas de repas prévu").color(TEXTE_DISCRET),
                            );
                        }
                    }
                });
                ui.add_space(8.0);
            }
        });

        if let Some(r) = a_ouvrir {
            self.recette_courante = Some(r);
            self.page_precedente = Page::Acceuil;
            self.page = Page::Recette;
        }
    }

    // -----------------------------------------------------------------
    //  PAGE : FICHE RECETTE (ingrédients à gauche, étapes à droite)
    // -----------------------------------------------------------------
    pub fn afficher_recette(&mut self, ui: &mut egui::Ui) {
        let r = match &self.recette_courante {
            Some(r) => r.clone(),
            None => {
                ui.label("Aucune recette sélectionnée.");
                return;
            }
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(4.0);
            ui.label(RichText::new(&r.name).size(30.0).strong());
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                pastille(ui, &personnes(r.nbPersonne), ACCENT);
                pastille(ui, &format!("{} min", r.duree), ACCENT);
            });
            ui.add_space(14.0);

            let largeur_gauche = (ui.available_width() * 0.36).clamp(240.0, 380.0);

            ui.horizontal_top(|ui| {
                // ---- Colonne de gauche : ingrédients ----
                ui.vertical(|ui| {
                    ui.set_width(largeur_gauche);
                    carte(CARD).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(
                            RichText::new("Ingrédients")
                                .size(18.0)
                                .strong()
                                .color(ACCENT),
                        );
                        ui.add_space(4.0);
                        egui::Grid::new("grille_ingredients")
                            .num_columns(2)
                            .spacing([16.0, 6.0])
                            .show(ui, |ui| {
                                for (nom, quantite) in ingredients_tries(&r.ingredients) {
                                    ui.label(nom);
                                    ui.label(RichText::new(quantite).strong());
                                    ui.end_row();
                                }
                            });
                    });
                });

                ui.add_space(10.0);

                // ---- Colonne de droite : préparation ----
                ui.vertical(|ui| {
                    let reste = ui.available_width();
                    ui.set_width(reste);

                    ui.label(
                        RichText::new("Préparation")
                            .size(18.0)
                            .strong()
                            .color(ACCENT),
                    );
                    ui.add_space(4.0);

                    let liste = etapes(&r.recette);
                    if liste.is_empty() {
                        ui.label(RichText::new("Pas d'instructions.").color(TEXTE_DISCRET));
                    }
                    for (n, texte) in liste.iter().enumerate() {
                        carte(CARD).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal_top(|ui| {
                                ui.label(
                                    RichText::new(format!("{}", n + 1))
                                        .size(22.0)
                                        .strong()
                                        .color(ACCENT),
                                );
                                ui.add_space(4.0);
                                let largeur_texte = ui.available_width();
                                ui.vertical(|ui| {
                                    ui.set_width(largeur_texte);
                                    ui.label(RichText::new(texte).size(16.5));
                                });
                            });
                        });
                        ui.add_space(6.0);
                    }
                });
            });
        });
    }

    // -----------------------------------------------------------------
    //  PAGE : SEMAINE (compacte)
    // -----------------------------------------------------------------
    pub fn afficher_semaine(&mut self, ui: &mut egui::Ui, semaine: &Week) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(self.id_week > 0, egui::Button::new("<"))
                .clicked()
            {
                self.id_week -= 1;
            }

            let titre = match (semaine.days.first(), semaine.days.last()) {
                (Some(debut), Some(fin)) => format!(
                    "Semaine du {} au {}",
                    date_courte(&debut.date),
                    date_courte(&fin.date)
                ),
                _ => "Semaine".to_string(),
            };
            ui.label(RichText::new(titre).size(22.0).strong());

            if ui
                .add_enabled(
                    self.id_week + 1 < self.semaines.len(),
                    egui::Button::new(">"),
                )
                .clicked()
            {
                self.id_week += 1;
            }

            if self.id_week == 0 {
                pastille(ui, "en cours", OK);
            }
            pastille(
                ui,
                &format!("Cuisine : {} min", getSumDuration(semaine)),
                ACCENT,
            );
        });
        ui.add_space(8.0);

        carte(CARD).show(ui, |ui| {
            ui.set_width(ui.available_width());
            self.afficher_grille_semaine(ui, semaine);
        });

        // Envoi manuel : uniquement quand l'envoi automatique est désactivé
        if !self.config.auto_send_sl {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let en_cours = self.envoi_en_cours.is_some();
                let texte = if en_cours {
                    "Envoi en cours..."
                } else {
                    "Envoyer les ingrédients de cette semaine"
                };
                let bouton =
                    egui::Button::new(RichText::new(texte).strong().color(SUR_ACCENT)).fill(ACCENT);
                if ui
                    .add_enabled(!en_cours, bouton)
                    .on_hover_text(
                        "Les quantités s'ajoutent à la liste de courses existante : n'envoie qu'une fois par semaine.",
                    )
                    .clicked()
                {
                    self.envoyer_ingredients(semaine);
                }

                if let Some((message, succes)) = &self.message_envoi {
                    let couleur = if *succes { OK } else { KO };
                    ui.label(RichText::new(message).color(couleur));
                }
            });
        }

        ui.add_space(6.0);
        ui.label(
            RichText::new("Clique sur une recette pour ouvrir sa fiche (le survol affiche le nom complet).")
                .size(12.5)
                .color(TEXTE_DISCRET),
        );
    }

    /// Grille compacte d'une semaine.
    /// - page Semaine : une pastille cliquable par repas
    /// - page Réglages : boutons -/+ du nombre de mangeurs
    pub fn afficher_grille_semaine(&mut self, ui: &mut egui::Ui, week: &Week) {
        let nb_jours = week.days.len().max(1);
        let espacement = 6.0;
        let largeur_libelle = 78.0;
        let largeur_col = ((ui.available_width()
            - largeur_libelle
            - espacement * (nb_jours as f32 + 1.0))
            / nb_jours as f32)
            .clamp(96.0, 200.0);
        let max_caracteres = ((largeur_col - 18.0) / 6.6).max(6.0) as usize;
        let mode_config = matches!(self.page, Page::Config);

        let mut recette_cliquee: Option<Recette> = None;

        egui::ScrollArea::horizontal().show(ui, |ui| {
            egui::Grid::new("grille_semaine")
                .spacing([espacement, espacement])
                .show(ui, |ui| {
                    // --- EN-TÊTE : jours de la semaine ---
                    ui.label("");
                    for day in &week.days {
                        let couleur = if est_aujourdhui(&day.date) {
                            ACCENT
                        } else {
                            TEXTE_DISCRET
                        };
                        ui.vertical_centered(|ui| {
                            ui.set_width(largeur_col);
                            ui.label(
                                RichText::new(format!(
                                    "{} {}",
                                    jour_court(&day.date),
                                    nombre(&day.date, "%d")
                                ))
                                .strong()
                                .color(couleur),
                            );
                        });
                    }
                    ui.end_row();

                    // --- LIGNES : un repas par ligne ---
                    for i in 0..4 {
                        ui.label(
                            RichText::new(REPAS_NOMS[i])
                                .size(13.0)
                                .strong()
                                .color(REPAS_COULEURS[i]),
                        );

                        for (d, day) in week.days.iter().enumerate() {
                            if mode_config {
                                let index = i * week.days.len() + d;
                                if index < self.config.list_nb_mangeur.len() {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 3.0;
                                        // Centre le bloc [-] [n] [+] (82 px de large) dans la colonne
                                        ui.add_space(((largeur_col - 82.0) / 2.0).max(0.0));
                                        if ui
                                            .add_sized([22.0, 24.0], egui::Button::new("-"))
                                            .clicked()
                                        {
                                            add_value(&mut self.config.list_nb_mangeur[index], -1);
                                        }
                                        let n = self.config.list_nb_mangeur[index];
                                        let (fond, texte) =
                                            if n > 0 { (OK, SUR_OK) } else { (KO, Color32::WHITE) };
                                        ui.add_sized(
                                            [32.0, 24.0],
                                            egui::Button::new(
                                                RichText::new(n.to_string()).strong().color(texte),
                                            )
                                            .fill(fond),
                                        );
                                        if ui
                                            .add_sized([22.0, 24.0], egui::Button::new("+"))
                                            .clicked()
                                        {
                                            add_value(&mut self.config.list_nb_mangeur[index], 1);
                                        }
                                    });
                                } else {
                                    ui.label("?");
                                }
                            } else {
                                match day.menu.get(i).and_then(|repas| repas.recette.as_ref()) {
                                    Some(recette) => {
                                        let couleur = REPAS_COULEURS[i];
                                        let bouton = egui::Button::new(
                                            RichText::new(tronquer(&recette.name, max_caracteres))
                                                .size(12.5),
                                        )
                                        .fill(couleur.linear_multiply(0.16))
                                        .stroke(Stroke::new(1.0, couleur.linear_multiply(0.5)));

                                        let reponse = ui
                                            .add_sized([largeur_col, 28.0], bouton)
                                            .on_hover_text(format!(
                                                "{} - {} min",
                                                recette.name, recette.duree
                                            ));
                                        if reponse.clicked() {
                                            recette_cliquee = Some(recette.clone());
                                        }
                                    }
                                    None => {
                                        ui.add_sized(
                                            [largeur_col, 28.0],
                                            egui::Label::new(
                                                RichText::new("—").color(TEXTE_DISCRET),
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                        ui.end_row();
                    }
                });
        });

        if let Some(r) = recette_cliquee {
            self.recette_courante = Some(r);
            self.page_precedente = self.page.clone();
            self.page = Page::Recette;
        }
    }

    // -----------------------------------------------------------------
    //  PAGE : REGLAGES
    // -----------------------------------------------------------------
    pub fn afficher_config(&mut self, ui: &mut egui::Ui, semaine: &Week) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(4.0);
            ui.label(RichText::new("Réglages").size(26.0).strong());
            ui.add_space(8.0);

            // ---- Nombre de mangeurs ----
            carte(CARD).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new("Nombre de mangeurs par repas")
                        .size(17.0)
                        .strong()
                        .color(ACCENT),
                );
                ui.label(
                    RichText::new("0 (rouge) = pas de repas prévu")
                        .size(12.5)
                        .color(TEXTE_DISCRET),
                );
                ui.add_space(6.0);
                self.afficher_grille_semaine(ui, semaine);
            });
            ui.add_space(10.0);

            // ---- Génération des menus ----
            carte(CARD).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new("Génération des menus")
                        .size(17.0)
                        .strong()
                        .color(ACCENT),
                );
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label("Durée maximale de cuisine par semaine :");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.duree_max_config).desired_width(60.0),
                    );
                    ui.label("min");
                });

                ui.horizontal(|ui| {
                    interrupteur(ui, &mut self.config.is_recette_filtre);
                    ui.label("Exclure automatiquement les recettes hors saison");
                });

                ui.add_space(6.0);
                if ui.button("Régénérer la semaine prochaine").clicked() && self.semaines.len() > 1
                {
                    generateMenu(
                        &mut self.semaines[1],
                        &self.recettes,
                        &self.config.list_nb_mangeur,
                        self.duree_max_config
                            .parse()
                            .unwrap_or(self.config.duree_max),
                        self.config.is_recette_filtre,
                    );
                }
            });
            ui.add_space(10.0);

            // ---- Liste de courses ----
            carte(CARD).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new("Liste de courses partagée")
                        .size(17.0)
                        .strong()
                        .color(ACCENT),
                );
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    interrupteur(ui, &mut self.config.auto_send_sl);
                    ui.label("Envoyer automatiquement le menu dans la liste de courses");
                });

                if self.config.auto_send_sl {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let a_un_code = self.config.c_liste.len() == 6;
                        if a_un_code {
                            ui.label("Code :");
                            ui.label(
                                RichText::new(&self.config.c_liste)
                                    .size(20.0)
                                    .strong()
                                    .color(ACCENT),
                            );
                        }
                        let texte = if a_un_code {
                            "Régénérer un code"
                        } else {
                            "Générer un code"
                        };
                        if ui.button(texte).clicked() {
                            if let Err(e) = self.config.ensure_room_code() {
                                eprintln!("[ROOM ERREUR] {}", e);
                            }
                        }
                    });
                }
            });
            ui.add_space(10.0);
        });
    }

    // -----------------------------------------------------------------
    //  PAGE : RECETTES (activer / désactiver)
    // -----------------------------------------------------------------
    pub fn afficher_recettes(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Mes recettes").size(26.0).strong());
            let actives = self.recettes.iter().filter(|r| r.activable).count();
            pastille(
                ui,
                &format!("{} / {} actives", actives, self.recettes.len()),
                OK,
            );
        });
        ui.label(
            RichText::new(
                "Seules les recettes actives servent à générer les menus. Pense à enregistrer.",
            )
            .size(12.5)
            .color(TEXTE_DISCRET),
        );
        ui.add_space(8.0);

        let mut a_ouvrir: Option<Recette> = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for r in 0..self.recettes.len() {
                let hors_saison = !self.recettes[r].is_valable();
                let resume = ingredients_tries(&self.recettes[r].ingredients)
                    .iter()
                    .map(|(nom, quantite)| format!("{} ({})", nom, quantite))
                    .collect::<Vec<String>>()
                    .join("  ·  ");
                let liste_etapes = etapes(&self.recettes[r].recette);

                ui.push_id(r, |ui| {
                    carte(CARD).show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        ui.horizontal(|ui| {
                            interrupteur(ui, &mut self.recettes[r].activable);

                            let couleur_nom = if self.recettes[r].activable {
                                TEXTE
                            } else {
                                TEXTE_DISCRET
                            };
                            ui.label(
                                RichText::new(&self.recettes[r].name)
                                    .size(18.0)
                                    .strong()
                                    .color(couleur_nom),
                            );
                            pastille(
                                ui,
                                &format!("{} pers.", self.recettes[r].nbPersonne),
                                TEXTE_DISCRET,
                            );
                            pastille(
                                ui,
                                &format!("{} min", self.recettes[r].duree),
                                TEXTE_DISCRET,
                            );
                            if hors_saison {
                                pastille(ui, "hors saison", ACCENT);
                            }
                            if ui.small_button("Voir la fiche").clicked() {
                                a_ouvrir = Some(self.recettes[r].clone());
                            }
                        });

                        ui.add_space(2.0);
                        ui.label(RichText::new(resume).size(13.0).color(TEXTE_DISCRET));

                        egui::CollapsingHeader::new("Voir la préparation").show(ui, |ui| {
                            for (n, texte) in liste_etapes.iter().enumerate() {
                                ui.label(format!("{}. {}", n + 1, texte));
                            }
                        });
                    });
                });
                ui.add_space(6.0);
            }
        });

        if let Some(r) = a_ouvrir {
            self.recette_courante = Some(r);
            self.page_precedente = Page::Recettes;
            self.page = Page::Recette;
        }
    }

    pub fn sauvegarder_recettes(&self) {
        if let Err(e) = sauvegarder_recettes("recettes.json", &self.recettes) {
            eprintln!("[ERREUR] Sauvegarde des recettes impossible : {}", e);
        }
    }
}
