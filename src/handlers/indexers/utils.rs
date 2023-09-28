use uuid::Uuid;

use crate::constants::s3::INDEXER_SERVICE_SCRIPTS_FOLDER;

pub fn get_s3_script_key(id: Uuid) -> String {
    format!("{}/{}.js", INDEXER_SERVICE_SCRIPTS_FOLDER, id)
}

pub fn get_script_tmp_directory(id: Uuid) -> String {
    format!("{}/{}/{}.js", env!("CARGO_MANIFEST_DIR"), "tmp", id)
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn test_get_s3_script_key_random_uuid() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let expected_key = format!("{}/550e8400-e29b-41d4-a716-446655440000.js", INDEXER_SERVICE_SCRIPTS_FOLDER);

        let result_key = get_s3_script_key(id);

        assert_eq!(result_key, expected_key);
    }

    #[test]
    fn test_get_s3_script_key_all_zeros_uuid() {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        let expected_key = format!("{}/00000000-0000-0000-0000-000000000000.js", INDEXER_SERVICE_SCRIPTS_FOLDER);

        let result_key = get_s3_script_key(id);

        assert_eq!(result_key, expected_key);
    }

    #[test]
    fn test_get_s3_script_key_all_fs_uuid() {
        let id = Uuid::parse_str("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap();
        let expected_key = format!("{}/ffffffff-ffff-ffff-ffff-ffffffffffff.js", INDEXER_SERVICE_SCRIPTS_FOLDER);

        let result_key = get_s3_script_key(id);

        assert_eq!(result_key, expected_key);
    }

    #[test]
    fn test_get_script_tmp_directory_random_uuid() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let expected_path = format!("{}/tmp/550e8400-e29b-41d4-a716-446655440000.js", env!("CARGO_MANIFEST_DIR"));

        let result_path = get_script_tmp_directory(id);

        assert_eq!(result_path, expected_path);
    }

    #[test]
    fn test_get_script_tmp_directory_all_zeros_uuid() {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        let expected_path = format!("{}/tmp/00000000-0000-0000-0000-000000000000.js", env!("CARGO_MANIFEST_DIR"));

        let result_path = get_script_tmp_directory(id);

        assert_eq!(result_path, expected_path);
    }

    #[test]
    fn test_get_script_tmp_directory_all_fs_uuid() {
        let id = Uuid::parse_str("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap();
        let expected_path = format!("{}/tmp/ffffffff-ffff-ffff-ffff-ffffffffffff.js", env!("CARGO_MANIFEST_DIR"));

        let result_path = get_script_tmp_directory(id);

        assert_eq!(result_path, expected_path);
    }
}
