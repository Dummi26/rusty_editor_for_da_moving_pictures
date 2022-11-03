mod egui;
mod speedy2d;
mod quick_commands;

pub fn main(args: crate::cli::CustomArgs) -> crate::cli::CustomArgs {
    speedy2d::main(args)
}