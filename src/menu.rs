use crate::fonctions::get_saison;
use chrono::{DateTime, Datelike, Local, TimeZone};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::format,
};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tag {
    Viande,
    Boeuf,
    Porc,
    Poulet,
    Poisson,
    Vegetarien,
    Pates,
    Riz,
    Patates,
    Legumes,
    Oeufs,
    Fromage,
    Rapide,
    Mijote,
    PetitDej,
    Gouter,
    Dessert,
    Entree,
}
impl Tag {
    /// Tous les tags, dans l'ordre d'affichage (à compléter si tu ajoutes une variante)
    pub const TOUS: [Tag; 18] = [
        Tag::Viande,
        Tag::Boeuf,
        Tag::Porc,
        Tag::Poulet,
        Tag::Poisson,
        Tag::Vegetarien,
        Tag::Pates,
        Tag::Riz,
        Tag::Patates,
        Tag::Legumes,
        Tag::Oeufs,
        Tag::Fromage,
        Tag::Rapide,
        Tag::Mijote,
        Tag::PetitDej,
        Tag::Gouter,
        Tag::Dessert,
        Tag::Entree,
    ];

    /// Nom affiché dans l'interface
    pub fn nom(&self) -> &'static str {
        match self {
            Tag::Viande => "Viande",
            Tag::Boeuf => "Boeuf",
            Tag::Porc => "Porc",
            Tag::Poulet => "Poulet",
            Tag::Poisson => "Poisson",
            Tag::Vegetarien => "Végétarien",
            Tag::Pates => "Pâtes",
            Tag::Riz => "Riz",
            Tag::Patates => "Patates",
            Tag::Legumes => "Légumes",
            Tag::Oeufs => "Œufs",
            Tag::Fromage => "Fromage",
            Tag::Rapide => "Rapide",
            Tag::Mijote => "Mijoté",
            Tag::PetitDej => "Petit-déj",
            Tag::Gouter => "Goûter",
            Tag::Dessert => "Dessert",
            Tag::Entree => "Entrée",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub enum Saison {
    Automne,
    Hiver,
    Printemps,
    Ete,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ingredient {
    pub nom: String,
    pub disponibilité: HashSet<Saison>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ElementIngredient {
    pub ingredient: Ingredient,
    pub quantite: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListIngredient {
    pub ingredients: HashMap<String, ElementIngredient>,
}
impl ListIngredient {
    pub fn new() -> Self {
        ListIngredient {
            ingredients: HashMap::new(),
        }
    }

    pub fn display(&self) -> String {
        let mut res: String = String::from("");
        for (nom, element) in &self.ingredients {
            res.push_str(&format!("{}: {} - ", nom, element.quantite));
        }
        res
    }

    /// Ajoute `nb` unités de `ingredient` (cumule si déjà présent, clé = nom)
    pub fn add(&mut self, ingredient: Ingredient, nb: i32) {
        if let Some(element) = self.ingredients.get_mut(&ingredient.nom) {
            element.quantite += nb;
        } else {
            self.ingredients.insert(
                ingredient.nom.clone(),
                ElementIngredient {
                    ingredient,
                    quantite: nb,
                },
            );
        }
    }

    pub fn add_list(&mut self, liste: &ListIngredient) {
        for element in liste.ingredients.values() {
            self.add(element.ingredient.clone(), element.quantite);
        }
    }

    pub fn reduce(&mut self, name: String, nb: i32) {
        if let Some(element) = self.ingredients.get_mut(&name) {
            element.quantite -= nb;
        }
    }

    pub fn remove(&mut self, name: String) {
        self.ingredients.remove(&name);
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Recette {
    pub name: String,
    pub nbPersonne: i32,
    pub duree: i32,
    pub ingredients: ListIngredient,
    pub recette: String,
    pub activable: bool,
    #[serde(default)]
    pub est_reste: bool,
    #[serde(default)]
    pub tags: Vec<Tag>,
}
impl Recette {
    pub fn display(&self) -> String {
        return format!(
            "{} pour {} personnes ({}):\n{}",
            &self.name,
            &self.nbPersonne,
            &self.duree,
            &self.ingredients.display(),
        );
    }
    pub fn affichage_complet(&self) -> String {
        return format!(
            "{} pour {} personnes ({}):\n{}\n{}",
            &self.name,
            &self.nbPersonne,
            &self.duree,
            &self.ingredients.display(),
            &self.recette
        );
    }
    pub fn multNbRecette(&mut self, nb: i32) {
        for element in self.ingredients.ingredients.values_mut() {
            element.quantite *= nb;
        }
        self.nbPersonne *= nb;
    }

    pub fn is_valable(&self) -> bool {
        // La saison est calculée une seule fois, et la liste d'ingrédients n'est plus clonée
        let saison = get_saison(&Local::now());
        self.ingredients.ingredients.values().all(|ing| {
            ing.ingredient.disponibilité.is_empty()
                || ing.ingredient.disponibilité.contains(&saison)
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Repas {
    pub name: String,
    pub nbPersonne: i32,
    pub recette: Option<Recette>,
    /// Entrée du repas (recette taguée « Entrée »), si le repas en comporte une
    #[serde(default)]
    pub entree: Option<Recette>,
    /// Dessert du repas (recette taguée « Dessert »), si le repas en comporte un
    #[serde(default)]
    pub dessert: Option<Recette>,
}
impl Repas {
    pub fn new(name: &str, nb_personne: i32) -> Self {
        Repas {
            name: name.to_string(),
            nbPersonne: nb_personne,
            recette: None,
            entree: None,
            dessert: None,
        }
    }

    pub fn display(&self) -> String {
        let recette_txt = match &self.recette {
            Some(r) => format!("{} : {}", r.name, r.ingredients.display()),
            None => "Aucune".to_string(),
        };
        return format!(
            "Name: {} - Nb de personne: {} - recette: {}",
            self.name, self.nbPersonne, recette_txt,
        );
    }
}
