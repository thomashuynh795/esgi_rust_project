pub enum UserError {
    DBNotFound,
}

pub struct User;

struct DB;

fn get_db() -> Option<DB> {
    // Some(DB)
    None
}

pub fn get_user(name: &str) -> Result<Option<User>, UserError> {
    let db = get_db().ok_or(UserError::DBNotFound)?;
    if name == "John Doe" {
        Ok(Some(User))
    } else {
        Ok(None)
    }
}
