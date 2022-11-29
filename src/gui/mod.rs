mod egui;
mod speedy2d;
mod quick_commands;
mod cli;

pub fn main(args: crate::cli::CustomArgs) -> crate::cli::CustomArgs {
    speedy2d::main(args)
}
pub fn main_cli(args: crate::cli::CustomArgs) -> crate::cli::CustomArgs {
    cli::interactive::main(args)
}