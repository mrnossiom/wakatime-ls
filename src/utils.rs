// Module for utility functions

/// Validates a wakatime api-key
pub fn is_valid_api_key(api_key: String) -> bool {
	// api-key format is waka_<uuidv4>
	api_key.len() == 41 && api_key.starts_with("waka_")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn empty_api_key() {
		assert!(!is_valid_api_key("".to_string()));
	}

	#[test]
	fn invalid_format_1() {
		assert!(!is_valid_api_key("waka_abcd".to_string()));
	}
}
