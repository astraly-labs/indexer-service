pub fn get_environment_variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{} is not set", name))
}
