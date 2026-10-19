pub mod capabilities {
    pub fn supports_truecolor() -> bool {
        std::env::var("NO_COLOR").is_err()
            && std::env::var("TERM")
                .map(|term| term != "dumb")
                .unwrap_or(true)
    }
}
