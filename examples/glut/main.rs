fn main() {
    println!("A: {}", u32::from('A'));
    println!("我: {}", u32::from('我'));
    let s1 = "Hello world!";
    for c in s1.chars() {
        print!("{}", c);
    }
    print!("\n");
    println!(
        "Char len: {}",
        std::mem::size_of_val(&s1.chars().nth(0).unwrap())
    );
    let s2 = String::from("我是张小白");
    for c in s2.chars() {
        print!("{}", c);
    }
    print!("\n");
    println!(
        "Char len: {}",
        std::mem::size_of_val(&s2.chars().nth(0).unwrap())
    );
}
