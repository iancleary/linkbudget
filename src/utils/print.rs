#[allow(dead_code)]
pub fn print_row(msg1: &str, msg2: &str, msg3: &str) {
    println!("{0: <20} | {1: <20} | {2: <5}", msg1, msg2, msg3);
}

#[allow(dead_code)]
pub fn print_header() {
    print_row("--------------", "-----", "----");
}

#[allow(dead_code)]
pub fn print_separator() {
    // println!("");
    println!("--------------------------------------------------");
    println!("");
}

#[allow(dead_code)]
pub fn print_title(title: &str) {
    println!("                   {}", title);
    println!("--------------------------------------------------");
    println!("");
}
