mod div;
mod my_error;

use div::div;
use my_error::MyError; /* as MyError */
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;

// trait X {
//     fn f(&self) -> String;
//     fn g(&self) -> String { String::new() } // implementation par défaut
// }
//
// impl X for i32 {
//     fn f(&self) -> String {
//         todo!()
//     }
//     // utilisation de l'implémentation par défaut
// }

#[derive(Debug, Serialize, Deserialize)]
struct Point {
    x: i32,
    y: i32,
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point({}, {})", self.x, self.y)
    }
}

fn f() -> Result<(), MyError> {
    let s = r#"{"x":1,"y":2}"#;
    let p2 = serde_json::from_str::<Point>(s).map_err(|err| MyError::SerdeError)?;
    // Boite(ok).map(f) => Boite(f(ok))
    // Boite(err).map_err(f) => Boite(f(err))

    // en Python: f = lambda x : 2*x
    // en Rust: f = |x| 2*x

    println!("p2: {p2}");

    let user = shared::get_user("toto").map_err(|err| MyError::DBError)?;

    let c = div(5, 0)?;
    let d = 2 * c;
    // println!("c={}", c);

    Ok(())
}

fn main() {
    let p = Point { x: 1, y: 2 };
    println!("{p:?}");
    println!("{p}");

    let s: String = p.to_string(); // disponible grâce à Display
    let s = format!("{p}");
    println!("{s}");

    let s2 = serde_json::to_string(&p).unwrap();
    println!("JSON: {s2}");
    // let p2 : Point = Point::from_str(s);

    let p2: Point = serde_json::from_str(&s2).unwrap();
    println!("{p2}");

    let s = "Point(1, 2)".to_string();
    let p2_result = serde_json::from_str::<Point>(&s2);
    // let p2 = match p2_result {
    //     Ok(ok) => ok,
    //     Err(err) => {
    //         return Err(Box::new(err));
    //     }
    // };
    // let p2 = p2_result?;

    match f() {
        Ok(_) => {
            println!("Success")
        }
        Err(MyError::DivideByZero) => {
            println!("Failed DivideByZero")
        }
        Err(MyError::SerdeError) => {
            println!("Failed SerdeError")
        }
        Err(MyError::DBError) => {
            println!("Failed DBError")
        }
    }

    // def f():
    //     raise RuntimeError()
    //
    // try:
    //     f()
    //     g()
    // except Error:
    //     pass

    // Result::<(), Box<dyn std::error::Error>>::Ok(())
    // Ok(())
}
