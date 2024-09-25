#[allow(unused)]
pub(crate) trait StringUtils {
    fn capitalize(&self) -> String;

    fn to_ascii(&self) -> String;
}

impl StringUtils for String {
    fn capitalize(&self) -> String {
        let mut c = self.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    fn to_ascii(&self) -> String {
        self.chars().filter(|c| c.is_ascii()).collect()
    }
}
