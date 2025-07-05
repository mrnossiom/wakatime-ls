// Module for utility functions

pub fn is_valid_api_key(api_key: String) -> bool {
	api_key.len() == 41 && api_key.starts_with("waka")
}
