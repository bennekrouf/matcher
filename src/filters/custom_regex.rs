
use regex::Regex;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref EMAIL_REGEX: Regex = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
}
