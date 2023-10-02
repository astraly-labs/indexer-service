pub fn get_test_db_url() -> String {
    std::env::var("TEST_DB_URL").expect("`TEST_DB_URL` environment variable not set")
}
