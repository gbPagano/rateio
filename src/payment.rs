use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use petgraph::dot::Dot;
use petgraph::prelude::StableDiGraph;
use petgraph::visit::IntoEdgeReferences;
use petgraph::visit::{EdgeRef, IntoNodeReferences, NodeRef};

use crate::money::Money;
use crate::person::Person;

/// Representa uma transação única de pagamento entre duas pessoas.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Payment {
    pub from: Person,
    pub to: Person,
    pub value: Money,
}

impl Payment {
    pub fn new(from: &Person, to: &Person, value: Money) -> Self {
        Self {
            from: from.clone(),
            to: to.clone(),
            value,
        }
    }
}

/// Representa o grafo de pagamentos.
pub struct Payments(StableDiGraph<Person, Money>);

impl Payments {
    pub fn new(payments: &[Payment]) -> Self {
        let mut graph = StableDiGraph::<Person, Money>::new();

        let persons: HashSet<_> = payments.iter().flat_map(|p| [&p.from, &p.to]).collect();

        let node_map: HashMap<_, _> = persons
            .into_iter()
            .map(|p| {
                let idx = graph.add_node(p.clone());
                (p, idx)
            })
            .collect();

        for p in payments {
            let from = node_map[&p.from];
            let to = node_map[&p.to];

            graph.add_edge(from, to, p.value);
        }

        Self(graph)
    }

    /// Retorna todas as pessoas presentes no grafo.
    pub fn get_persons(&self) -> Vec<Person> {
        self.0
            .node_references()
            .map(|n| n.weight().clone())
            .collect()
    }

    /// Otimiza o grafo de pagamentos para reduzir o número de transações.
    pub fn optimize(&mut self) {
        self.simplify_bidirectional_edges();
        self.simplify_transitive_edges();

        assert!(self.validate());
    }

    /// Simplifica dívidas mútuas entre duas pessoas.
    ///
    /// Mantém apenas o saldo líquido: por exemplo, se A deve 10 para B, e B deve 7 para A,
    /// o resultado será A deve 3 para B. Se forem iguais, ambas são removidas.
    fn simplify_bidirectional_edges(&mut self) {
        let indexes = self.0.edge_indices().collect::<Vec<_>>();
        for edge in indexes {
            if let Some((source, target)) = self.0.edge_endpoints(edge) {
                if let Some(e2) = self.0.find_edge(target, source)
                    && let Some(e1) = self.0.find_edge(source, target)
                {
                    let w1 = self.0.edge_weight(e1).unwrap();
                    let w2 = self.0.edge_weight(e2).unwrap();

                    match w1.cmp(w2) {
                        Ordering::Less => {
                            // Aresta A -> B é removida
                            // Aresta B -> A é atualizada com a diferença
                            self.0.update_edge(target, source, *w2 - *w1);
                            self.0.remove_edge(e1);
                        }
                        Ordering::Greater => {
                            // Aresta A -> B é atualizada com a diferença
                            // Aresta B -> A é removida
                            self.0.update_edge(source, target, *w1 - *w2);
                            self.0.remove_edge(e2);
                        }
                        Ordering::Equal => {
                            // Dívidas se anulam
                            self.0.remove_edge(e1);
                            self.0.remove_edge(e2);
                        }
                    }
                }
            }
        }
    }

    /// Simplifica dívidas transitivas (A -> B -> C) criando um "atalho" (A -> C).
    ///
    /// Esta função itera sobre o grafo para encontrar padrões de dívida onde
    /// `A` deve para `B`, e `B` deve para `C`. Quando esse padrão é encontrado,
    /// a dívida é reestruturada para que `A` pague `C` diretamente,
    /// reduzindo o número de transações intermediárias.
    fn simplify_transitive_edges(&mut self) {
        let edge_indexes = self.0.edge_indices().collect::<Vec<_>>();
        for edge_bc in edge_indexes {
            let (node_b, node_c) = match self.0.edge_endpoints(edge_bc) {
                Some(nodes) => nodes,
                None => continue,
            };

            let incoming: Vec<_> = self
                .0
                .edges_directed(node_b, petgraph::Incoming)
                .map(|e| (e.source(), e.id()))
                .collect();
            for (node_a, edge_ab) in incoming {
                // Não simplifica um ciclo de volta para o pagador (A -> B -> A)
                if node_a == node_c {
                    continue;
                }

                if let Some(&w_ab) = self.0.edge_weight(edge_ab)
                    && let Some(&w_bc) = self.0.edge_weight(edge_bc)
                {
                    // Valor que pode ser "transferido" diretamente de B para C
                    let w_transfer = w_ab.min(w_bc);
                    if w_transfer == Money::from(0) {
                        continue;
                    }

                    // 1. Adiciona/atualiza a aresta A -> C
                    if let Some(edge_ac) = self.0.find_edge(node_a, node_c) {
                        *self.0.edge_weight_mut(edge_ac).unwrap() += w_transfer;
                    } else {
                        self.0.update_edge(node_a, node_c, w_transfer);
                    }

                    // 2. Atualiza ou remove a aresta A -> B
                    let w_ab_new = w_ab - w_transfer;
                    if w_ab_new == Money::from(0) {
                        self.0.remove_edge(edge_ab);
                    } else {
                        *self.0.edge_weight_mut(edge_ab).unwrap() = w_ab_new;
                    }

                    // 3. Atualiza ou remove a aresta B -> C
                    let w_bc_new = w_bc - w_transfer;
                    if w_bc_new == Money::from(0) {
                        self.0.remove_edge(edge_bc);
                    } else {
                        *self.0.edge_weight_mut(edge_bc).unwrap() = w_bc_new;
                    }
                }
            }
        }
    }

    pub fn to_vec(&self) -> Vec<Payment> {
        let persons = self.get_persons();
        self.0
            .edge_references()
            .map(|edge| {
                let source = persons
                    .iter()
                    .find(|p| p == &self.0.node_weight(edge.source()).unwrap())
                    .unwrap();
                let target = persons
                    .iter()
                    .find(|p| p == &self.0.node_weight(edge.target()).unwrap())
                    .unwrap();
                Payment::new(source, target, *edge.weight())
            })
            .collect()
    }

    /// Imprime a representação do grafo no formato Graphviz DOT na saída padrão.
    pub fn print_dot(&self) {
        let dot = Dot::new(&self.0);
        println!("{dot}");
    }

    /// Verifica se os pagamentos estão consistentes dentro de um limite de tolerância.
    ///
    /// Calcula o valor médio que cada pessoa deveria ter pago e compara com o saldo
    /// final de cada participante (considerando o que gastou, pagou e recebeu).
    ///
    /// Aceita pequenas diferenças de até '0,5 centavo * número de participantes'.
    /// Retorna `true` se todos os saldos estiverem dentro desse limite.
    pub fn validate(&self) -> bool {
        let payments = self.to_vec();
        let persons = self.get_persons();

        let num_persons: u32 = persons.iter().map(|p| p.size()).sum();
        let total_debt: Money = persons.iter().map(|p| p.money_spent()).sum();
        let amount_for_each = total_debt / num_persons;

        for person in persons {
            let to_receive: Money = payments
                .iter()
                .filter(|p| p.to == person)
                .map(|p| p.value)
                .sum();
            let to_pay: Money = payments
                .iter()
                .filter(|p| p.from == person)
                .map(|p| p.value)
                .sum();

            let final_balance = (person.money_spent() + to_pay - to_receive) / person.size();

            // Verifica se a diferença está dentro do limite de tolerância.
            // O limite é definido como 0.5 centavos multiplicado pelo número total de pessoas,
            // permitindo uma margem de erro proporcional ao tamanho do grupo.
            let diff = (amount_for_each.decimal() - final_balance.decimal()).abs();
            if diff >= 0.005 * num_persons as f64 {
                return false;
            }
        }
        true
    }
}

impl FromIterator<Person> for Payments {
    fn from_iter<T: IntoIterator<Item = Person>>(iter: T) -> Self {
        let persons: Vec<Person> = iter.into_iter().collect();
        let mut payments = Vec::new();

        let num_persons: u32 = persons.iter().map(|p| p.size()).sum();

        for creditor in persons.iter() {
            if matches!(creditor, Person::Unnamed { .. })
                || matches!(creditor, Person::Named { money_spent, .. } if money_spent.cents() == 0)
            {
                continue;
            }

            let amount_for_each = creditor.money_spent() / num_persons as f64;
            for debitor in persons.iter().filter(|p| p != &creditor) {
                let amount = amount_for_each * debitor.size();

                payments.push(Payment::new(debitor, creditor, amount));
            }
        }

        Payments::new(&payments)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn simplify_bidirectional_edges() {
        let persons = vec![
            Person::named("A", 10.into()),
            Person::named("B", 20.into()),
            Person::named("C", 10.into()),
            Person::unnamed(1),
        ];

        let mut initial_payments: Payments = persons.clone().into_iter().collect();

        let final_payments = vec![
            Payment::new(&persons[0], &persons[1], 2.5.into()),
            Payment::new(&persons[2], &persons[1], 2.5.into()),
            Payment::new(&persons[3], &persons[0], 2.5.into()),
            Payment::new(&persons[3], &persons[2], 2.5.into()),
            Payment::new(&persons[3], &persons[1], 5.into()),
        ];

        initial_payments.simplify_bidirectional_edges();
        let left: HashSet<Payment> = HashSet::from_iter(initial_payments.to_vec());
        let right: HashSet<Payment> = HashSet::from_iter(final_payments);

        assert_eq!(left, right);
        assert!(initial_payments.validate());
    }

    #[test]
    fn simplify_transitive_edges() {
        let persons = vec![
            Person::named("A", 14.into()),
            Person::named("B", 20.into()),
            Person::named("C", 8.into()),
            Person::unnamed(1),
        ];

        let mut initial_payments: Payments = persons.clone().into_iter().collect();

        let final_payments = vec![
            Payment::new(&persons[2], &persons[1], 2.5.into()),
            Payment::new(&persons[3], &persons[1], 7.into()),
            Payment::new(&persons[3], &persons[0], 3.5.into()),
        ];

        initial_payments.simplify_bidirectional_edges();
        initial_payments.simplify_transitive_edges();
        let left: HashSet<Payment> = HashSet::from_iter(initial_payments.to_vec());
        let right: HashSet<Payment> = HashSet::from_iter(final_payments);

        assert_eq!(left, right);
        assert!(initial_payments.validate());
    }
}
