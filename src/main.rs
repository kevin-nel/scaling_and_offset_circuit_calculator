use std::{env, io};

// from 'designing gain and offset in thirty seconds' - Application Report SLOA097 (Texas Instruments)
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        panic!("\n\nPlease type 5 values (as floats) separated by a space in the following order\nv_ref vo_fs vo_zs vi_fs vi_zs.\n\n");
    }
    let mut values = [0.0; 5];
    for (i, arg) in &mut args[1..6].iter().enumerate() {
        values[i] = match arg.parse::<f64>() {
            Ok(value) => value,
            Err(error) => {
                panic!("\n\ncould not parse input as floats\n{error}\n\nPlease type 5 values (as floats) separated by a space in the following order\nv_ref vo_fs vo_zs vi_fs vi_zs.\n\n");
            }
        };
    }
    let vref = values[0];
    let vo_fs = values[1];
    let vo_zs = values[2];
    let vi_fs = values[3];
    let vi_zs = values[4];

    let circuit = AmplifierCircuit::calc(vref, vo_fs, vo_zs, vi_fs, vi_zs);
    println!("\ncomponent values:\n{:?}", circuit);
}

#[derive(Debug)]
enum AmplifierCircuit {
    TopologyA {
        // positive gain and positive offset
        // figure 1
        // section 3
        r_1: f64,
        r_2: f64,
        r_f: f64,
        r_g: f64,
    },
    TopologyB {
        // positive gain and negative offset
        // figure 2
        // section 4
        r_f: f64,
        r_g: f64,
        r_g2: f64,
        r_g1: f64,
        vref_prime: f64,
        r_1: f64,
    },
    TopologyBEnhanced {
        // figure 3
        r_f: f64, // selected
        r_g: f64,
        vref_prime: f64,
        r_1: f64, // selected
        r_2: f64,
    },
    TopologyC {
        // section 5
        // negative gain and positive offset
        r_f: f64, // needs to be selected
        r_g: f64,
        r_2: f64, // same order of magnitude as rf
        r_1: f64, // same order of magnitude as rf
    },

    TopologyD {
        // section 6
        // negative gain and negative offset
        r_f: f64, // needs to be selected
        r_g1: f64,
        r_g2: f64, // same order of magnitude as rf
    },
}

impl AmplifierCircuit {
    fn calc(vref: f64, vo_fs: f64, vo_zs: f64, vi_fs: f64, vi_zs: f64) -> AmplifierCircuit {
        let gain = (vo_fs - vo_zs) / (vi_fs - vi_zs);
        let offset = vo_zs - (gain * vi_zs);

        match (gain.is_sign_negative(), offset.is_sign_negative()) {
            (false, false) => {
                let r_1 = AmplifierCircuit::get_user_input("select r_1:");
                let r_2 = vref * r_1 * gain / offset;
                let r_f = AmplifierCircuit::get_user_input(
                    "select r_f:\n(this may have been suggested by datasheet)",
                );
                let r_g = r_2 * r_f / (gain * (r_1 + r_2) - r_2);
                AmplifierCircuit::TopologyA { r_1, r_2, r_f, r_g }
            }
            (false, true) => {
                let r_f = AmplifierCircuit::get_user_input(
                    "select r_f:\nselect r_f:\n(this may have been suggested by datasheet)",
                );
                let r_g = r_f / (gain - 1.0);
                let r_g2 = r_g / 10.0;
                let r_g1 = r_g - r_g2;
                let vref_prime = offset.abs() * r_g1 / (r_g1 - r_f);
                let r_1 = r_g2 * (vref - vref_prime) / vref_prime;
                AmplifierCircuit::TopologyB {
                    r_f,
                    r_g,
                    r_g2,
                    r_g1,
                    vref_prime,
                    r_1,
                }
            }
            /*
            (false, true) => {
                let r_f = AmplifierCircuit::get_user_input(
                    "select r_f:\nselect r_f:\n(this may have been suggested by datasheet)",
                );
                let r_g = r_f / (gain - 1.0);
                let vref_prime = offset.abs() / gain;
                let r_1 = AmplifierCircuit::get_user_input("select r_1:");
                let r_2 = (vref_prime * r_1) / (vref - vref_prime);

                AmplifierCircuit::TopologyBEnhanced {
                    r_f,
                    r_g,
                    vref_prime,
                    r_1,
                    r_2,
                }
            }
            */
            (true, false) => {
                let r_f = AmplifierCircuit::get_user_input(
                    "select r_f:\nselect r_f:\n(this may have been suggested by datasheet)",
                );
                let r_g = r_f / gain.abs();
                let r_2 = AmplifierCircuit::get_user_input(
                    "select r_2:\n(same order of magnitude as r_f)",
                );
                let r_1 = (offset * r_2 * r_g) / ((vref * (r_f + r_g)) - (offset * r_g));
                AmplifierCircuit::TopologyC { r_f, r_g, r_2, r_1 }
            }
            (true, true) => {
                let r_f = AmplifierCircuit::get_user_input("select r_f:\nselect r_f:\n");
                let r_g1 = r_f / gain.abs();
                let r_g2 = vref * (r_f / offset.abs());
                AmplifierCircuit::TopologyD { r_f, r_g1, r_g2 }
            }
        }
    }
    fn get_user_input(name: &str) -> f64 {
        println!("{name}");
        let mut value = String::new();

        io::stdin()
            .read_line(&mut value)
            .expect("Failed to read line");

        let value: f64 = value.trim().parse().expect("Please type a number!");
        value
    }
}
