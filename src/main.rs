// This file is part of the OOF project, released under the Creative Commons CC0
// https://creativecommons.org/publicdomain/zero/1.0/

extern crate over;
extern crate semver;

use over::obj::Obj;

mod schemas;

fn main() {
    let obj = Obj::from_file("examples/alpine/system.over.oof").unwrap();
    println!("{:?}", obj);
}
