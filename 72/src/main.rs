use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <hex1> <hex2> <OP>", args[0]);
        std::process::exit(1);
    }
    let a = bignum::from_hex(&args[1]);
    let b = bignum::from_hex(&args[2]);
    let result = match args[3].as_str() {
        "ADD" => bignum::add_ix(&a, &b),
        "SUB" => bignum::sub_ix(&a, &b),
        "MUL" => bignum::mul_ix(&a, &b),
        "QUO" => bignum::div_ix(&a, &b),
        "REM" => bignum::rem_ix(&a, &b),
        _ => {
            eprintln!("Unknown operation: {}", args[3]);
            std::process::exit(1);
        }
    };
    println!("{}", bignum::to_hex(&result));
}
