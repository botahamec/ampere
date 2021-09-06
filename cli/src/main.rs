use ai::CheckersBitBoard;
use clap::{App, Arg, SubCommand};

mod eval;
mod perft;

fn main() {
	let matches = App::new("Ampere")
		.version("0.1")
		.author("Botahamec <botahamec@outlook.com>")
		.about("An American Checkers AI")
		.subcommand(
			SubCommand::with_name("perft")
				.about("Calculate the number of possible moves")
				.arg(
					Arg::with_name("depth")
						.required(true)
						.short("d")
						.takes_value(true)
						.help("The depth to go to"),
				),
		)
		.subcommand(
			SubCommand::with_name("eval")
				.about("Calculate the advantage")
				.arg(
					Arg::with_name("depth")
						.required(true)
						.short("d")
						.takes_value(true)
						.help("The depth to go to"),
				),
		)
		.get_matches();

	if let Some(matches) = matches.subcommand_matches("perft") {
		println!(
			"{}",
			perft::positions(
				CheckersBitBoard::starting_position(),
				matches
					.value_of("depth")
					.unwrap()
					.parse::<usize>()
					.expect("Error: not a valid number")
			)
		);
	}

	if let Some(matches) = matches.subcommand_matches("eval") {
		println!(
			"{}",
			eval::eval(
				matches
					.value_of("depth")
					.unwrap()
					.parse::<usize>()
					.expect("Error: not a valid number")
			)
		);
	}
}
