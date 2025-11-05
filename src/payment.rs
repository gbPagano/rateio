use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use petgraph::dot::{Config, Dot};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::prelude::StableDiGraph;
use petgraph::visit::IntoEdgeReferences;
use petgraph::visit::{EdgeRef, IntoNodeReferences, NodeRef};

use crate::money::Money;
use crate::person::Person;

#[derive(Debug)]
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
            value: value.into(),
        }
    }
}

pub struct Payments(pub StableDiGraph<Person, Money>);

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

    pub fn get_persons(&self) -> Vec<Person> {
        self.0
            .node_references()
            .map(|n| n.weight().clone())
            .collect()
    }

    pub fn optimize(&mut self) {
        self.simplify_bidirectional_edges();
    }

    fn simplify_bidirectional_edges(&mut self) {
        let indexes = self.0.edge_indices().collect::<Vec<_>>();
        for edge in indexes {
            if let Some((source, target)) = self.0.edge_endpoints(edge) {
                if let Some(e2) = self.0.find_edge(target, source)
                    && let Some(e1) = self.0.find_edge(source, target)
                {
                    let w1 = self.0.edge_weight(e1).unwrap();
                    let w2 = self.0.edge_weight(e2).unwrap();

                    match w1.cmp(&w2) {
                        Ordering::Less => {
                            self.0.update_edge(target, source, *w2 - *w1);
                            self.0.remove_edge(e1);
                        }
                        Ordering::Greater => {
                            self.0.update_edge(source, target, *w1 - *w2);
                            self.0.remove_edge(e2);
                        }
                        Ordering::Equal => {
                            self.0.remove_edge(e1);
                            self.0.remove_edge(e2);
                        }
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

    pub fn print_dot(&self) {
        let dot = Dot::with_attr_getters(
            &self.0,
            &[],
            &|_graph, edge_ref| format!("label=\"{}\"", edge_ref.weight()),
            &|_graph, node_ref| format!("label=\"{}\"", node_ref.1.identifier()),
        );
        println!("{dot:?}");
    }
}
