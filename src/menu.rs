use crate::fonctions::get_saison;
use chrono::{DateTime, Datelike, Local, TimeZone};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::format,
};

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
        let now: DateTime<Local> = Local::now();
        let mut is_valable: bool = true;
        for (nom, ing) in self.ingredients.ingredients.clone() {
            if !(ing.ingredient.disponibilité.len() == 0
                || ing.ingredient.disponibilité.contains(&get_saison(&now)))
            {
                is_valable = false;
                break;
            }
        }
        return is_valable;
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Repas {
    pub name: String,
    pub nbPersonne: i32,
    pub recette: Option<Recette>,
}
impl Repas {
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
