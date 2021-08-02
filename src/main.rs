// This file is part of the OOF project, released under the Creative Commons CC0
// https://creativecommons.org/publicdomain/zero/1.0/

extern crate console;
extern crate over;
extern crate semver;

use console::style;
use over::obj::Obj;

mod schemas;

fn main() {
    let obj = Obj::from_file("examples/alpine/system.over.oof").unwrap();

    match schemas::system::from_over_obj(&obj) {
        Ok(cfg) => {
            println!("{:?}", cfg)
        }
        Err(err) => {
            eprintln!("{}: {:?}", style("error parsing <file>").red(), err)
        }
    }
}
