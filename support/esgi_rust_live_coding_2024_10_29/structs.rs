// All the examples from  the slides:
//  https://www.github.com/haveneer/rust-quicklook-training.git

// Many ways to configure an enum
enum Color {
    Red,
    Yellow,
    Blue,
    RGB(u8, u8, u8),
    RGBA(RGBA),
    CYMK {
        cyan: u8,
        magenta: u8,
        yellow: u8,
        black: u8,
    },
}

#[derive(Debug, Clone)] // Add Clone and Debug 'features' (Debug and Clone trait implementations)
struct RGBA {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

struct Unit; // Unit struct (single value struct)

impl Unit {
    // A UnitStruct may also have implementation based on its value (!= static function)
    fn f(&self) {
        // todo!()
    }
}

struct Port(u16); // Tuple Struct

impl Port {
    // Port::new is a constructor (just a special case of
    fn new(port: u16) -> Self {
        Self(port)
    }

    // several ways to call a method on an object (depending on the ownership/borrowing you need)

    // obj.is_privileged(...)
    fn is_privileged1(&self) -> bool {
        self.0 <= 1024
    }

    fn is_privileged2(self) -> bool {
        self.0 <= 1024
    }

    fn is_privileged3(&mut self) -> bool {
        self.0 <= 1024
    }

    // An associated function (like a "static function" is C/C++/other)
    fn class_name() -> String {
        String::from("Port")
    }
}

#[test]
fn test() {
    let port1: u16 = 8080;
    let mut port2 = Port(8080);

    let x = port1 * 2; // possible but dangerous for a 'port'

    // let x = port2 * 2; // Port type protects from using irrelevant actions

    println!("{}", port2.0);
    println!("is privileged: {}", port2.is_privileged1());
    Port::is_privileged1(&port2); // equivalent to the previous one

    // println!("is privileged: {}", port2.is_privileged2());
    println!("is privileged: {}", port2.is_privileged3());
    println!("{}", Port::class_name());

    Unit.f();
    // Unit::f(&Unit); // also equivalent to this, but less convenient

    let a = 1;
    let b = 2;
    // Choose you style (same execution)
    let c = a.min(b);
    let c = i32::min(a, b);

    let color = Color::RGB(255, 0, 255);

    // A 'match' should always manage all cases.
    match &color {
        Color::Red => {}
        Color::Yellow => {}
        Color::Blue => {}
        Color::RGB(r, g, b) => {}
        Color::RGBA(ref rgba) => paint_rgba(rgba),
        Color::CYMK { .. } => {}

        // _ => {} // manage remaining cases; not a good practice if you use an alternative way
    }

    // To not take ownership of a part of 'color', we should use a reference
    // either with '&' on the right (for all cases)
    // or with 'ref' on the left (selected cases)
    if let Color::RGBA(/* ref */ rgba) = &color {
        paint_rgba(rgba);
    }

    // Deeper destructuration (don't need a & or ref, since u8 supports 'Copy')
    if let Color::RGBA(RGBA {
        red,
        green,
        blue,
        alpha: 0,
    }) = &color
    {
        let rgba = RGBA {
            red: *red,
            green: *green,
            blue: *blue,
            alpha: 0,
        };
        paint_rgba(&rgba);
    }

    if let Color::RGBA(
        rgba @ RGBA {
            red,
            green,
            blue,
            alpha: 0,
        },
    ) = &color
    {
        paint_rgba(rgba);
    } else {
        // return;
    }

    // You can alias partial match with @
    // Recent 'let else' structure
    let Color::RGBA(
        rgba @ RGBA {
            red,
            green,
            blue,
            alpha: 0,
        },
    ) = &color
    else {
        // else-block should stop the execution flow
        println!("BAD RGBA"); // too soft
        panic!();
        todo!();
        unimplemented!();
        return;
    };

    // now, for sure, we have a valid value of rgba
    paint_rgba(rgba);
}

fn paint_rgba(rgba: &RGBA) {}
