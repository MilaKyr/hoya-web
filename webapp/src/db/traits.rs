pub trait ExternalText {
    fn cleaned(&self) -> Self;

    fn clean(&self, value: &str) -> String {
        let value = value.trim().to_lowercase();
        value
            .chars()
            .filter(|c| !c.is_ascii_punctuation())
            .collect::<String>()
    }
}
