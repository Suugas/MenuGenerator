use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListIngredient {
    pub ingredients: HashMap<String, i32>,
}
impl ListIngredient {
    pub fn display(&self) -> String {
        let mut res: String = String::from("");
        for (cle, val) in &self.ingredients {
            res.push_str(&cle.to_string());
            res.push_str(": ");
            res.push_str(&val.to_string());
            res.push_str(" - ");
        }
        return res;
    }
    pub fn add(&mut self, name: String, nb: i32) {
        if let Some(quantite) = self.ingredients.get_mut(&name) {
            *quantite += nb;
        } else {
            self.ingredients.insert(name, nb);
        }
    }
    pub fn reduce(&mut self, name: String, nb: i32) {
        if let Some(quantite) = self.ingredients.get_mut(&name) {
            *quantite -= nb;
        }
    }
    pub fn remove(&mut self, name: String) {
        if self.ingredients.contains_key(&name) {
            self.ingredients.remove(&name);
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Recette {
    pub name: String,
    pub nbPersonne: i32,
    pub duree: i32,
    pub ingredients: ListIngredient,
    pub recette: String,
}
impl Recette {
    pub fn display(&self) -> String {
        return format!(
            "{} pour {} personnes ({}):\n{}\n{}",
            &self.name,
            &self.nbPersonne,
            &self.duree,
            &self.ingredients.display(),
            &self.recette,
        );
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Repas {
    pub name: String,
    pub nbPersonne: i8,
    pub isActive: bool,
    pub recette: Option<Recette>,
}
impl Repas {
    pub fn display(&self) -> String {
        let recette_txt = match &self.recette {
            Some(r) => r.name.to_string(),
            None => "Aucune".to_string(),
        };
        return format!(
            "Name: {} - Nb de personne: {} - recette: {}",
            self.name, self.nbPersonne, recette_txt,
        );
    }
}
