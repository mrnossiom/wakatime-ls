// Module for utility functions

pub fn is_valid_api_key(api_key: String) -> bool {
	// api-key format is waka_<uuidv4>
	api_key.len() == 41 && api_key.starts_with("waka_")
}
