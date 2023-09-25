use argparse::{ArgumentParser, Store};

fn main() {
    let mut addr = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Transparently broadcasts stdin, stdout via icmp");
        ap.refer(&mut addr)
          .add_option(&["-t", "--target-address"], Store, "Target IP");
        ap.parse_args_or_exit();
    }

    println!("{}", addr)
}
