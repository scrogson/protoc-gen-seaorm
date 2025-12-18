//! Integration tests for protoc-gen-seaorm
//!
//! These tests exercise the full code generation pipeline.

use prost_types::{
    compiler::CodeGeneratorRequest, field_descriptor_proto::Type, DescriptorProto,
    FieldDescriptorProto, FileDescriptorProto, MessageOptions, UninterpretedOption,
};
use prost_types::uninterpreted_option::NamePart;

/// Create a test CodeGeneratorRequest with a simple User message
fn create_test_request() -> CodeGeneratorRequest {
    // Create the seaorm.message option
    let message_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.message".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("table_name: \"users\"".to_string()),
        ..Default::default()
    };

    // Create field options for primary key
    let pk_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.field".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("primary_key: true, auto_increment: true".to_string()),
        ..Default::default()
    };

    // Create field options for unique
    let unique_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.field".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("unique: true".to_string()),
        ..Default::default()
    };

    // Create the User message
    let user_message = DescriptorProto {
        name: Some("User".to_string()),
        field: vec![
            FieldDescriptorProto {
                name: Some("id".to_string()),
                number: Some(1),
                r#type: Some(Type::Int64.into()),
                options: Some(prost_types::FieldOptions {
                    uninterpreted_option: vec![pk_option],
                    ..Default::default()
                }),
                ..Default::default()
            },
            FieldDescriptorProto {
                name: Some("email".to_string()),
                number: Some(2),
                r#type: Some(Type::String.into()),
                options: Some(prost_types::FieldOptions {
                    uninterpreted_option: vec![unique_option],
                    ..Default::default()
                }),
                ..Default::default()
            },
            FieldDescriptorProto {
                name: Some("name".to_string()),
                number: Some(3),
                r#type: Some(Type::String.into()),
                ..Default::default()
            },
        ],
        options: Some(MessageOptions {
            uninterpreted_option: vec![message_option],
            ..Default::default()
        }),
        ..Default::default()
    };

    // Create the file descriptor
    let file_descriptor = FileDescriptorProto {
        name: Some("test/user.proto".to_string()),
        package: Some("test".to_string()),
        message_type: vec![user_message],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    CodeGeneratorRequest {
        file_to_generate: vec!["test/user.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    }
}

#[test]
fn test_generate_entity() {
    let request = create_test_request();
    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    // Should have no error
    assert!(response.error.is_none(), "should have no error");

    // Should generate one file
    assert_eq!(response.file.len(), 1, "should generate one file");

    let file = &response.file[0];
    assert!(file.name.as_ref().unwrap().ends_with("user.rs"));

    let content = file.content.as_ref().unwrap();

    // Check for expected content
    assert!(content.contains("DeriveEntityModel"), "should have DeriveEntityModel derive");
    assert!(content.contains("table_name = \"users\""), "should have table_name attribute");
    assert!(content.contains("pub id: i64"), "should have id field");
    assert!(content.contains("pub email: String"), "should have email field");
    assert!(content.contains("pub name: String"), "should have name field");
    assert!(content.contains("primary_key"), "should have primary_key attribute");
    assert!(content.contains("unique"), "should have unique attribute");
}

#[test]
fn test_skip_message_without_options() {
    // Create a message without seaorm options
    let message = DescriptorProto {
        name: Some("NoOptions".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("field".to_string()),
            number: Some(1),
            r#type: Some(Type::String.into()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/no_options.proto".to_string()),
        package: Some("test".to_string()),
        message_type: vec![message],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/no_options.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    // Should have no error
    assert!(response.error.is_none());

    // Should generate no files (message was skipped)
    assert_eq!(response.file.len(), 0, "should generate no files for messages without seaorm options");
}

#[test]
fn test_skip_explicitly_skipped_message() {
    // Create the skip option
    let skip_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.message".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("skip: true".to_string()),
        ..Default::default()
    };

    let message = DescriptorProto {
        name: Some("Skipped".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("field".to_string()),
            number: Some(1),
            r#type: Some(Type::String.into()),
            ..Default::default()
        }],
        options: Some(MessageOptions {
            uninterpreted_option: vec![skip_option],
            ..Default::default()
        }),
        ..Default::default()
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/skipped.proto".to_string()),
        package: Some("test".to_string()),
        message_type: vec![message],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/skipped.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    // Should have no error
    assert!(response.error.is_none());

    // Should generate no files (message was explicitly skipped)
    assert_eq!(response.file.len(), 0, "should generate no files for explicitly skipped messages");
}
