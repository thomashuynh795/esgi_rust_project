use crate::my_error::MyError;

pub fn div(a: i32, b: i32) -> Result<i32, MyError> {
    details::debug();
    if b != 0 {
        Ok(a / b)
    } else {
        Err(MyError::DivideByZero)
    }
}

mod details {
    pub fn debug() {
        println!("debug");
    }
}

mod tests {
    use crate::div::div;

    #[test]
    fn test_div() {
        assert_eq!(div(8, 4).unwrap(), 2);
    }

    #[test]
    fn test_div_by_zero() {
        assert!(div(8, 0).is_err());
    }

    #[test]
    #[should_panic]
    fn test_div_by_zero2() {
        div(8, 0).unwrap();
    }
}
