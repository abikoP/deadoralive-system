// Test Suite Entry Point
// すべてのテストモジュールをここで宣言します

// ヘルパー関数
mod helpers;

// Unit Tests
mod unit {
    pub mod config_service_tests;
    pub mod url_validation_service_tests;
    pub mod telegraf_controller_tests;
}

// Property-Based Tests
mod property {
    pub mod property_1_atomicity_tests;
    pub mod property_2_sighup_tests;
    pub mod property_5_rollback_tests;
}

// Integration Tests
mod integration {
    pub mod manager_telegraf_tests;
    pub mod manager_docker_socket_tests;
}
