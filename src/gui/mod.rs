mod egui;
mod speedy2d;

pub fn main(args: crate::cli::CustomArgs) -> crate::cli::CustomArgs {
    speedy2d::main(args)
}