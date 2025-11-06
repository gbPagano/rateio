use crate::payment::{Payment, Payments};
use crate::person::Person;

pub fn gen_payments(persons: &[Person]) -> Payments {
    let mut payments = Vec::new();

    let num_persons: usize = persons
        .iter()
        .map(|p| match p {
            Person::Named { .. } => 1,
            Person::Unnamed { size } => *size,
        })
        .sum();

    for creditor in persons {
        if matches!(creditor, Person::Unnamed { .. })
            || matches!(creditor, Person::Named { money_spent, .. } if money_spent.cents() == 0)
        {
            continue;
        }

        let amount_for_each = creditor.money_spent() / num_persons as f64;
        for debitor in persons.iter().filter(|p| p != &creditor) {
            let amount = match debitor {
                Person::Named { .. } => amount_for_each,
                Person::Unnamed { size } => amount_for_each * *size as f64,
            };

            payments.push(Payment::new(debitor, creditor, amount));
        }
    }

    Payments::new(&payments)
}
