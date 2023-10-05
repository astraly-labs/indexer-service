pub fn get_environment_variable(name: &str) -> String {
    std::env::var(name).expect(format!("{} is not set", name).as_str())
}