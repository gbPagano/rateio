use std::hash::Hash;

#[derive(Debug, Clone, PartialEq)]
pub enum Person {
    Named { name: String, money_spent: f64 },
    Unnamed { size: usize },
}

impl Person {
    pub fn named(name: &str, money_spent: f64) -> Self {
        Person::Named {
            name: name.into(),
            money_spent,
        }
    }

    pub fn unnamed(size: usize) -> Self {
        Person::Unnamed { size }
    }

    pub fn identifier(&self) -> String {
        match self {
            Person::Named {
                name,
                money_spent: _,
            } => name.clone(),
            Person::Unnamed { size } => format!("Outras {size} pessoas"),
        }
    }

    pub fn money_spent(&self) -> f64 {
        match self {
            Person::Named { money_spent, .. } => *money_spent,
            Person::Unnamed { .. } => 0.,
        }
    }
}

impl Eq for Person {}

impl Hash for Person {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.identifier().hash(state);
    }
}
