use std::sync::{Arc, RwLock};
// Nous n'aurons pas vu:
// * atomic, mpsc
// * async (await) => framework du type tokio (std_async)

use std::time::Duration;

// fn f() {
//     println!("Thread1");
// }

// Etat des joueurs: tableau t:
// * Joueur 1: 1
// * Joueur 2: 2
// * Joueur 3: 3
// Joueur 1 reçoit un Challenge "SecretSum"
// Serveur envoie une mise à jour de son secret à Joueur 2 (valeur 5)
// Joueur 1 calcule
// Joueur 2 met à jour son secret avec la valeur 5
// Joueur 1 envoie la valeur 6
// Le serveur renvoie InvalidChallengeSolution
// Joueur 1 recalcule la solution et la renvoie

fn main() {
    // let h = std::thread::spawn(f);

    // let f = || println!("Thread1");
    // // f = lambda : print("Thread1") // Python
    // let h = std::thread::spawn(f);

    let word = Arc::new(String::from("Thread"));
    let mut acc = Arc::new(RwLock::new(String::from("acc:")));
    // let s = std::sync::Mutex::new(1);

    let h1 = {
        let word = word.clone();
        let mut acc = acc.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::new(0, 100));
            let mut acc = acc.write().unwrap();
            let id = 1;
            for _ in 0..50 {
                std::thread::sleep(Duration::new(0, 100));
                println!("{}{id}", word);
                acc.push_str("1");
            }
        })
    };

    println!("{word}");

    let h2 = {
        let word = word.clone();
        let mut acc = acc.clone();
        std::thread::spawn(move || {
            let id = 2;
            for _ in 0..5 {
                println!("{}{id}", word);
                acc.write().unwrap().push_str("2");
                std::thread::sleep(Duration::new(0, 10));
            }
        })
    };

    // std::thread::sleep(Duration::new(0, 1000));

    h1.join().unwrap();
    h2.join().unwrap();

    // BUG: interblocage
    // let lock1 = acc.write().unwrap();
    // let lock2 = acc.write().unwrap();

    {
        let x = acc.read().unwrap();
        println!("acc = {x}");
    }
    println!("acc = {}", acc.read().unwrap());
}

// Processus
// int main() {
//      int i;
//      int * p = new int;
//      *p = 0;
//      int * q = new int;
//      *q = 0;
//      mutex m
//      start Thread1
//      start Thread2

//      print *p

// }

// Thread1
// m.lock()
// print *p
// *p = 1
// m.unlock()

// Thread2
// m.lock()
// print *p
// *p = 2
// m.unlock()
