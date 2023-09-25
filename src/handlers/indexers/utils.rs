use crate::constants::s3::INDEXER_SERVICE_SCRIPTS_FOLDER;
use uuid::Uuid;

pub fn get_s3_script_key(id: Uuid) -> String {
    format!("{}/{}.js", INDEXER_SERVICE_SCRIPTS_FOLDER, id)
}

pub fn get_script_tmp_directory(id: Uuid) -> String {
    format!("{}/{}/{}.js", env!("CARGO_MANIFEST_DIR"), "tmp", id)
}
