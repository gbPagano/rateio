use std::collections::{HashMap, HashSet};

use petgraph::dot::{Config, Dot};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;

use crate::person::Person;

#[derive(Debug)]
pub struct Payment {
    pub from: Person,
    pub to: Person,
    pub value: f64,
}

impl Payment {
    pub fn new(from: &Person, to: &Person, value: f64) -> Self {
        Self {
            from: from.clone(),
            to: to.clone(),
            value,
        }
    }
}

pub trait ToGraph {
    fn to_graph(self) -> DiGraph<String, f64>;
}

impl ToGraph for &[Payment] {
    fn to_graph(self) -> DiGraph<String, f64> {
        let mut graph = DiGraph::<String, f64>::new();

        let persons: HashSet<_> = self.iter().flat_map(|p| [&p.from, &p.to]).collect();

        let node_map: HashMap<_, _> = persons
            .into_iter()
            .map(|p| {
                let idx = graph.add_node(p.identifier());
                (p, idx)
            })
            .collect();

        for payment in self {
            let from = node_map[&payment.from];
            let to = node_map[&payment.to];

            let val = ((payment.value * 100.) as i32) as f64 / 100.;
            graph.add_edge(from, to, val);
        }

        graph
    }
}
