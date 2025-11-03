mod payment;
mod person;
mod solver;

use clap::Parser;
use itertools::Itertools;

use person::Person;

fn parse_key_val(s: &str) -> Result<(String, f64), String> {
    let (key, value) = s.split_once('=').ok_or_else(|| {
        format!(
            "argumento inválido, esperado formato 'chave=valor': '{}'",
            s
        )
    })?;

    let parsed_value = value.parse::<f64>().map_err(|e| {
        format!(
            "erro ao processar o valor '{}' da chave '{}': {}",
            value, key, e
        )
    })?;

    Ok((key.to_string(), parsed_value))
}

/// Define os argumentos da linha de comando
#[derive(Parser, Debug)]
#[command(
    version,
    author,
    about = "Exemplo de CLI que aceita --num e pares chave=valor"
)]
struct Args {
    /// O número a ser fornecido
    #[arg(short, long)]
    num: usize,

    /// Lista de pares chave=valor posicionais
    #[arg(
        required = true,
        value_parser = parse_key_val
    )]
    pairs: Vec<(String, f64)>, // <-- Mude de volta para Vec<(String, String)>
}

fn main() {
    let args = Args::parse();

    if args.pairs.len() > args.num {
        eprintln!(
            "Erro: a conta nao fecha! (Número de pares: {}, --num: {})",
            args.pairs.len(),
            args.num
        );
        std::process::exit(1);
    }

    let mut persons: Vec<_> = args
        .pairs
        .iter()
        .map(|p| Person::named(&p.0, p.1))
        .collect();
    persons.push(Person::unnamed(args.num - args.pairs.len()));

    let payments = solver::calc_payments(&persons);
    let payments = solver::optimize_payments(&payments);

    for person in &persons {
        let debts: Vec<_> = payments.iter().filter(|p| p.from == *person).collect();

        let mut total_debt: f64 = debts.iter().map(|p| p.value).sum();
        if let Person::Unnamed { size } = person {
            total_debt /= *size as f64;
        }

        let total_to_receive: f64 = payments
            .iter()
            .filter_map(|p| if p.to == *person { Some(p.value) } else { None })
            .sum();

        println!("\n{}:", person.identifier());
        println!("    total a pagar: {total_debt:.2}");
        println!("    total a receber: {total_to_receive:.2}");
        if !debts.is_empty() {
            println!()
        }

        for p in debts {
            if let Person::Unnamed { size } = person {
                println!(
                    "    pagar: {:.2} -> {}",
                    p.value / *size as f64,
                    p.to.identifier()
                );
            } else {
                println!("    pagar: {:.2} -> {}", p.value, p.to.identifier());
            }
        }
    }
}
