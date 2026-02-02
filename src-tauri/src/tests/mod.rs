//! Integration tests for Tauri commands
//! 
//! These tests verify the contract between frontend and backend,
//! specifically focusing on parameter serialization which was the source
//! of the project_id bug.

#[cfg(test)]
mod contract_tests;
#[cfg(test)]
mod integration_tests;
