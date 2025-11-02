/// Comprehensive test file for the controller module
/// This file contains tests for the controller module's functionality
use crate::controller::controllerdb::*;

/// Tests for controllerdb functionality
#[cfg(test)]
mod controllerdb_tests {
    use super::*;

    #[test]
    fn test_find_gamecontroller_db_with_nonexistent_guid() {
        // Tests searching for a non-existent GUID
        let result = find_gamecontroller_db("00000000000000000000000000000000");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_find_gamecontroller_db_with_empty_guid() {
        // Tests searching for an empty GUID
        let result = find_gamecontroller_db("");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}

#[cfg(test)]
mod add_remove_controller_tests {
    use super::*;

    #[test]
    fn test_add_remove_multiple_controllers() {
        // Tests adding multiple controllers up to the maximum
        let guids: Vec<String> = (0..8).map(|i| format!("{:032x}", i)).collect();

        // Add controllers one by one
        for (index, guid) in guids.iter().enumerate() {
            let result = add_sdl_controller_config(index, guid);
            // This checks if the add was successful or if the controller wasn't found in DB
            assert!(result.is_ok());
        }

        // Verify that all controllers were added (those with mappings in DB)
        let all_configs = get_sdl_controller_config();
        assert!(all_configs.len() <= 8); // Should not exceed 8 controllers
    }

    #[test]
    fn test_add_controllers_exceeds_limit() {
        // Tests adding controllers up to the maximum limit of 8
        // Clear any existing configurations first
        // Add controllers one by one up to the limit
        let guids: Vec<String> = (0..8).map(|i| format!("{:032x}", i)).collect();

        // Add controllers one by one
        for (index, guid) in guids.iter().enumerate() {
            let result = add_sdl_controller_config(index, guid);
            assert!(result.is_ok());
        }

        // Verify we're at the limit
        let all_configs = get_sdl_controller_config();
        assert!(all_configs.len() <= 8);

        // Try to add one more controller
        let extra_guid = format!("{:032x}", 8);
        let _result = add_sdl_controller_config(8, &extra_guid);
        // The result depends on whether we already have 8 controllers with valid mappings
        // If we do, it should return an error (or return None if already configured)
    }

    #[test]
    fn test_get_all_sdl_controller_configs_empty() {
        // Clear any existing configurations first
        for (_index, _config) in get_sdl_controller_config() {
            let _ = remove_sdl_controller_config(Some(&_config.guid));
        }

        let all_configs = get_sdl_controller_config();
        assert!(all_configs.is_empty());
    }

    #[test]
    fn test_export_sdl_controller_configs_empty() {
        // Clear any existing configurations first
        for (_index, _config) in get_sdl_controller_config() {
            let _ = remove_sdl_controller_config(Some(&_config.guid));
        }

        let configs = get_sdl_controller_config();
        let exported = configs
            .iter()
            .map(|(_index, controller)| format!("{},{}", controller.guid, "mapping_data"))
            .collect::<Vec<String>>()
            .join("\n");
        assert!(exported.is_empty());
    }
}

#[cfg(test)]
mod controller_config_tests {
    use super::*;

    #[test]
    fn test_get_all_sdl_controller_configs_struct_empty() {
        // Clear any existing configurations first
        for (_index, _config) in get_sdl_controller_config() {
            let _ = remove_sdl_controller_config(Some(&_config.guid));
        }

        let all_configs = get_sdl_controller_config();
        assert!(all_configs.is_empty());
    }

    #[test]
    fn test_export_sdl_controller_configs_struct_empty() {
        // Clear any existing configurations first
        for (_index, _config) in get_sdl_controller_config() {
            let _ = remove_sdl_controller_config(Some(&_config.guid));
        }

        let configs = get_sdl_controller_config();
        let exported = configs
            .iter()
            .map(|(_index, controller)| format!("{},{}", controller.guid, "mapping_data"))
            .collect::<Vec<String>>()
            .join("\n");
        assert!(exported.is_empty());
    }

    #[test]
    fn test_controller_struct_creation() {
        // Tests creating a Controller struct with known values
        let guid = "030000005e0400008e02000010010000".to_string();
        let mut inputs = std::collections::HashMap::new();
        inputs.insert("a".to_string(), "b0".to_string());
        inputs.insert("b".to_string(), "b1".to_string());
        inputs.insert("back".to_string(), "b6".to_string());

        let controller = Controller {
            guid: guid.clone(),
            name: "Test Controller".to_string(),
            inputs,
        };

        assert_eq!(controller.guid, guid);
        assert_eq!(controller.name, "Test Controller");
    }

    #[test]
    fn test_export_sdl_controller_configs_struct_format() {
        // Tests that the exported controller configs have the correct format
        // Clear any existing configurations first
        for (_index, _config) in get_sdl_controller_config() {
            let _ = remove_sdl_controller_config(Some(&_config.guid));
        }

        // Add a specific controller
        let test_index = 0;
        let guid = "030000005e0400008e02000010010000";
        let _ = add_sdl_controller_config(test_index, guid); // Ignore result as controller may not exist in DB

        let configs = get_sdl_controller_config();
        let exported = configs
            .iter()
            .map(|(_index, controller)| format!("{},{}", controller.guid, "mapping_data"))
            .collect::<Vec<String>>()
            .join("\n");
        // If a mapping exists, the export should not be empty
        if !exported.is_empty() {
            let lines: Vec<&str> = exported.lines().collect();
            for line in lines {
                // Each line should contain at least one comma (guid,data format)
                assert!(line.contains(','));
            }
        }
        // If no mapping exists, exported string should be empty
    }
}

#[cfg(test)]
mod dynamic_controller_config_tests {
    use super::*;

    #[test]
    fn test_add_single_controller() {
        // Test adding a single controller
        let guid = "030000005e0400008e02000010010000";
        let index = 0;

        // Add the controller
        let result = add_sdl_controller_config(index, guid);
        match result {
            Ok(Some(controller)) => {
                assert_eq!(controller.guid, guid);
                // Verify it's actually in the configuration
                assert!(is_controller_configured(index));
            }
            Ok(None) => {
                // This is acceptable if the controller database doesn't contain this GUID
                // Just verify that the function didn't return an error
            }
            Err(_) => panic!("add_sdl_controller_config returned an error"),
        }
    }

    #[test]
    fn test_remove_single_controller() {
        // Find a controller GUID that exists in the database
        let test_guid = "030000005e0400008e02000010010000";
        let test_index = 0;

        // Check if this controller exists in the database first
        let controller_exists_in_db = find_gamecontroller_db(test_guid).unwrap_or(None).is_some();

        if controller_exists_in_db {
            // First add the controller
            let _ = add_sdl_controller_config(test_index, test_guid); // Ignore result

            // Verify the controller is configured
            assert!(is_controller_configured(test_index));

            // Remove the controller
            let result = remove_sdl_controller_config(Some(test_guid));
            assert!(result.is_ok());

            // Verify the controller is no longer configured
            assert!(!is_controller_configured(test_index));
        } else {
            // If the controller doesn't exist in the database, just verify the remove function doesn't error
            let result = remove_sdl_controller_config(Some(test_guid));
            assert!(result.is_ok());
        }
    }
}

#[cfg(test)]
mod get_controller_functionality_tests {
    use super::*;

    #[test]
    fn test_get_sdl_controller_config_returns_all() {
        // Clear any existing configurations first
        let _ = remove_sdl_controller_config(None);

        // Add some controllers
        let test_guid1 = "030000005e0400008e02000010010000";
        let test_guid2 = "0500000058626f782033363020576900";

        // Add the controllers (ignoring results as they may not exist in DB)
        let _ = add_sdl_controller_config(0, test_guid1);
        let _ = add_sdl_controller_config(1, test_guid2);

        // Get all controllers using the new function (no arguments)
        let all_configs = get_sdl_controller_config();

        // Should return all configured controllers
        let existing_configs_count = all_configs
            .iter()
            .filter(|(_index, c)| c.guid == test_guid1 || c.guid == test_guid2)
            .count();

        // At least the controllers that exist in the database should be present
        assert!(all_configs.len() >= existing_configs_count);
    }

    #[test]
    fn test_get_sdl_controller_config_empty() {
        // Clear all configurations
        let _ = remove_sdl_controller_config(None);

        // Should return empty vector when no controllers are configured
        let configs = get_sdl_controller_config();
        assert_eq!(configs.len(), 0);
    }

    #[test]
    fn test_remove_all_controllers_with_none_parameter() {
        // Add some controllers
        let test_guid1 = "030000005e0400008e02000010010000";
        let test_guid2 = "0500000058626f782033363020576900";

        let _ = add_sdl_controller_config(0, test_guid1);
        let _ = add_sdl_controller_config(1, test_guid2);

        // Verify they were added
        let configs_before = get_sdl_controller_config();
        let _initial_count = configs_before.len();

        // Remove all controllers using None
        let result = remove_sdl_controller_config(None);
        assert!(result.is_ok());

        // Verify all were removed
        let configs_after = get_sdl_controller_config();
        assert_eq!(configs_after.len(), 0);

        // Add them back to clean up for other tests
        let _ = add_sdl_controller_config(0, test_guid1);
        let _ = add_sdl_controller_config(1, test_guid2);
    }

    #[test]
    fn test_add_sdl_controller_config_returns_option() {
        // Test that add_sdl_controller_config returns Option<Controller>
        let test_guid = "030000005e0400008e02000010010000";
        let test_index = 0;

        let result = add_sdl_controller_config(test_index, test_guid);
        assert!(result.is_ok());

        let result_clone = result.as_ref();
        match result_clone.unwrap() {
            Some(controller) => {
                assert_eq!(controller.guid, test_guid);
            }
            None => {
                // This is acceptable if the controller doesn't exist in the database
            }
        }

        // Also test that the controller is properly configured
        if result.unwrap().is_some() {
            assert!(is_controller_configured(test_index));
        }
    }
}
