use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("arg1: {}", args[1]);

}

// have a map contain all the notes
// {
//  1 : Do this
//  2 : Do that
// }
//
// OPTIONS:
//  - ADD (1 arg)
//  - DELETE (1 or more args)
//  - EDIT (1 arg)
//  - CLEAR (no args)
