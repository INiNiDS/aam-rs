use aam_rs::aam::AAM;

fn main() {
    let parser = match AAM::parse(include_str!("standard.aam")) {
        Ok(aaml) => aaml,
        Err(e) => {
            eprint!("{:?}", e);
            return;
        }
    };

    if let vec = parser.find("a") {
        println!("{:?}", vec);
    }

    if let vec = parser.find("c") {
        println!("{:?}", vec);
        if let vec = parser.find(&*d) {
            println!("{:?}", vec);
        }
    }
}
