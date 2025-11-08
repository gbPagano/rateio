mod payment;
mod person;

use clap::Parser;
use rust_decimal::Decimal;

use payment::Payments;
use person::Person;

/// Uma CLI para dividir contas de forma justa
///
/// Calcula quanto cada pessoa deve pagar ou receber após uma série de gastos compartilhados
#[derive(Parser, Debug)]
#[command(name = "rachaconta", version)]
struct Args {
    /// Define o número total de pessoas que devem dividir os gastos.
    ///
    /// Por padrão, a conta é dividida igualmente apenas entre quem realizou
    /// algum pagamento. Use esta opção quando houver pessoas que não pagaram
    /// nada mas devem participar da divisão.
    ///
    /// Exemplo: Se 5 pessoas jantaram mas apenas 2 pagaram, use -p 5
    #[arg(short = 'p', long = "pessoas", value_name = "NÚMERO")]
    total_persons: Option<usize>,

    /// Lista dos gastos individuais no formato NOME=VALOR
    ///
    /// Cada pagamento deve ser informado como nome da pessoa seguido de
    /// igual e o valor pago.
    ///
    /// Exemplos:
    ///   Rafael=50.00 Maria=30.50 "Ana Clara"=100
    #[arg(
        required = true,
        value_parser = parse_key_val,
        value_name = "NOME=VALOR"
    )]
    initial_payments: Vec<(String, Decimal)>,

    /// Exporta o resultado no formato Graphviz DOT.
    ///
    /// Quando esta opção é ativada, em vez de exibir o resultado numérico
    /// das transações, o programa imprime na saída padrão a representação
    /// do grafo no formato **DOT**, compatível com o Graphviz.
    #[arg(short, long)]
    graphviz: bool,
}

fn main() {
    let args = Args::parse();

    let initial_payments = args.initial_payments;
    let total_persons = args.total_persons.unwrap_or(initial_payments.len());

    if initial_payments.len() > total_persons {
        eprintln!(
            "Erro: a conta não fecha! {} pessoa(s) pagaram, mas você informou apenas {} pessoa(s) no total.",
            initial_payments.len(),
            total_persons
        );
        eprintln!(
            "Dica: aumente -p para pelo menos {} ou remova a opção -p para dividir apenas entre quem pagou.",
            initial_payments.len()
        );
        std::process::exit(1);
    }

    let mut persons: Vec<_> = initial_payments
        .iter()
        .map(|p| Person::named(&p.0, p.1))
        .collect();

    let remaining = total_persons - initial_payments.len();
    if remaining > 0 {
        persons.push(Person::unnamed(remaining as u32));
    }

    let mut payments_graph: Payments = persons.into_iter().collect();
    payments_graph.optimize();

    if args.graphviz {
        payments_graph.print_dot();
        std::process::exit(0);
    }

    let payments = payments_graph.to_vec();
    for person in payments_graph.get_persons() {
        let debts: Vec<_> = payments.iter().filter(|p| p.from == person).collect();

        let mut total_debt: Decimal = debts.iter().map(|p| p.value).sum();
        if let Person::Unnamed { size } = person {
            total_debt /= Decimal::from(size);
        }

        let total_to_receive: Decimal = payments
            .iter()
            .filter_map(|p| if p.to == person { Some(p.value) } else { None })
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
                    p.value / Decimal::from(size),
                    p.to.identifier()
                );
            } else {
                println!("    pagar: {:.2} -> {}", p.value, p.to.identifier());
            }
        }
    }
}

/// Parser customizado para `clap` que transforma uma string "NOME=VALOR"
/// em uma tupla `(String, Decimal)`.
fn parse_key_val(s: &str) -> Result<(String, Decimal), String> {
    let (k, v) = s.split_once('=').ok_or("use o formato NOME=VALOR")?;
    Ok((
        k.into(),
        v.parse().map_err(|_| format!("número inválido: {v}"))?,
    ))
}
