// https://github.com/haveneer-training/sauve_qui_peut

mod structs;

fn main() {
    let mut s1 = String::from("world");
    let s2 = "world";

    say_hello_str(s2);
    say_hello_str(s2);
    say_hello_ref_string(&mut s1);
    s1.push_str("Z");
    say_hello_string(s1.clone());
    say_hello_string(s1); // ownership error

    // let v: Vec<i32> = Vec::new();
    let mut v = vec![1, 2, 3];
    v.push(47878_u16); // u16 contraint is propagated towards declaration (=> Vec<u16>)
    // without this, 'v' is a Vec<i32>
}

fn say_hello_str(name: &str) {
    println!("Hello {name}");
}

fn say_hello_string(mut name: String) {
    // let mut name = name; // equivalent 'mut name' in the signature

    // let s = name.as_str();

    // name.push_str("Z");
    println!("Hello {name}");
}

fn say_hello_ref_string(name: &String) {
    println!("Hello {name}");
}