//! Integration tests for protoc-gen-seaorm
//!
//! These tests exercise the full code generation pipeline.

use prost_types::uninterpreted_option::NamePart;
use prost_types::{
    compiler::CodeGeneratorRequest, field_descriptor_proto::Type, DescriptorProto,
    EnumDescriptorProto, EnumOptions, EnumValueDescriptorProto, FieldDescriptorProto,
    FileDescriptorProto, MessageOptions, MethodDescriptorProto, OneofDescriptorProto, OneofOptions,
    ServiceDescriptorProto, ServiceOptions, UninterpretedOption,
};

/// Create a test CodeGeneratorRequest with a simple User message
fn create_test_request() -> CodeGeneratorRequest {
    // Create the seaorm.model option
    let message_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.model".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("table_name: \"users\"".to_string()),
        ..Default::default()
    };

    // Create field options for primary key
    let pk_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.column".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("primary_key: true, auto_increment: true".to_string()),
        ..Default::default()
    };

    // Create field options for unique
    let unique_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.column".to_string(),
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
    assert!(
        content.contains("DeriveEntityModel"),
        "should have DeriveEntityModel derive"
    );
    assert!(
        content.contains("table_name = \"users\""),
        "should have table_name attribute"
    );
    assert!(content.contains("pub id: i64"), "should have id field");
    assert!(
        content.contains("pub email: String"),
        "should have email field"
    );
    assert!(
        content.contains("pub name: String"),
        "should have name field"
    );
    assert!(
        content.contains("primary_key"),
        "should have primary_key attribute"
    );
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
    assert_eq!(
        response.file.len(),
        0,
        "should generate no files for messages without seaorm options"
    );
}

#[test]
fn test_skip_explicitly_skipped_message() {
    // Create the skip option
    let skip_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.model".to_string(),
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
    assert_eq!(
        response.file.len(),
        0,
        "should generate no files for explicitly skipped messages"
    );
}

/// Create a test CodeGeneratorRequest with a Status enum
fn create_enum_test_request() -> CodeGeneratorRequest {
    // Create the seaorm.enum_opt option
    let enum_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.enum_opt".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("db_type: \"string\"".to_string()),
        ..Default::default()
    };

    // Create the Status enum
    let status_enum = EnumDescriptorProto {
        name: Some("Status".to_string()),
        value: vec![
            EnumValueDescriptorProto {
                name: Some("STATUS_UNKNOWN".to_string()),
                number: Some(0),
                ..Default::default()
            },
            EnumValueDescriptorProto {
                name: Some("STATUS_ACTIVE".to_string()),
                number: Some(1),
                ..Default::default()
            },
            EnumValueDescriptorProto {
                name: Some("STATUS_INACTIVE".to_string()),
                number: Some(2),
                ..Default::default()
            },
        ],
        options: Some(EnumOptions {
            uninterpreted_option: vec![enum_option],
            ..Default::default()
        }),
        ..Default::default()
    };

    // Create the file descriptor
    let file_descriptor = FileDescriptorProto {
        name: Some("test/status.proto".to_string()),
        package: Some("test".to_string()),
        enum_type: vec![status_enum],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    CodeGeneratorRequest {
        file_to_generate: vec!["test/status.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    }
}

#[test]
fn test_generate_enum() {
    let request = create_enum_test_request();
    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    // Should have no error
    assert!(response.error.is_none(), "should have no error");

    // Should generate one file
    assert_eq!(response.file.len(), 1, "should generate one file");

    let file = &response.file[0];
    assert!(file.name.as_ref().unwrap().ends_with("status.rs"));

    let content = file.content.as_ref().unwrap();

    // Check for expected content
    assert!(
        content.contains("DeriveActiveEnum"),
        "should have DeriveActiveEnum derive"
    );
    assert!(
        content.contains("rs_type = \"String\""),
        "should have rs_type String"
    );
    assert!(
        content.contains("string_value"),
        "should have string_value attributes"
    );
    assert!(
        content.contains("StatusUnknown") || content.contains("Unknown"),
        "should have variant"
    );
    assert!(
        content.contains("StatusActive") || content.contains("Active"),
        "should have variant"
    );
}

#[test]
fn test_generate_integer_enum() {
    // Create the seaorm.enum_opt option with integer type
    let enum_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.enum_opt".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("db_type: \"integer\"".to_string()),
        ..Default::default()
    };

    let priority_enum = EnumDescriptorProto {
        name: Some("Priority".to_string()),
        value: vec![
            EnumValueDescriptorProto {
                name: Some("PRIORITY_LOW".to_string()),
                number: Some(0),
                ..Default::default()
            },
            EnumValueDescriptorProto {
                name: Some("PRIORITY_MEDIUM".to_string()),
                number: Some(1),
                ..Default::default()
            },
            EnumValueDescriptorProto {
                name: Some("PRIORITY_HIGH".to_string()),
                number: Some(2),
                ..Default::default()
            },
        ],
        options: Some(EnumOptions {
            uninterpreted_option: vec![enum_option],
            ..Default::default()
        }),
        ..Default::default()
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/priority.proto".to_string()),
        package: Some("test".to_string()),
        enum_type: vec![priority_enum],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/priority.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none());
    assert_eq!(response.file.len(), 1);

    let content = response.file[0].content.as_ref().unwrap();
    assert!(
        content.contains("rs_type = \"i32\""),
        "should have rs_type i32"
    );
    assert!(
        content.contains("num_value"),
        "should have num_value attributes"
    );
}

#[test]
fn test_skip_enum_without_options() {
    // Create an enum without seaorm options
    let enum_desc = EnumDescriptorProto {
        name: Some("NoOptions".to_string()),
        value: vec![EnumValueDescriptorProto {
            name: Some("VALUE".to_string()),
            number: Some(0),
            ..Default::default()
        }],
        ..Default::default()
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/no_options_enum.proto".to_string()),
        package: Some("test".to_string()),
        enum_type: vec![enum_desc],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/no_options_enum.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none());
    assert_eq!(
        response.file.len(),
        0,
        "should generate no files for enums without seaorm options"
    );
}

#[test]
fn test_generate_entity_with_oneof_flatten() {
    // Create the seaorm.model option
    let message_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.model".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("table_name: \"payments\"".to_string()),
        ..Default::default()
    };

    // Create field options for primary key
    let pk_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.column".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("primary_key: true, auto_increment: true".to_string()),
        ..Default::default()
    };

    // Create the seaorm.oneof option for flatten strategy (default)
    let oneof_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.oneof".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("strategy: \"flatten\"".to_string()),
        ..Default::default()
    };

    // Create a Payment message with a oneof for payment method
    let payment_message = DescriptorProto {
        name: Some("Payment".to_string()),
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
                name: Some("amount".to_string()),
                number: Some(2),
                r#type: Some(Type::Double.into()),
                ..Default::default()
            },
            // Oneof fields
            FieldDescriptorProto {
                name: Some("credit_card_number".to_string()),
                number: Some(3),
                r#type: Some(Type::String.into()),
                oneof_index: Some(0),
                ..Default::default()
            },
            FieldDescriptorProto {
                name: Some("bank_account".to_string()),
                number: Some(4),
                r#type: Some(Type::String.into()),
                oneof_index: Some(0),
                ..Default::default()
            },
        ],
        oneof_decl: vec![OneofDescriptorProto {
            name: Some("payment_method".to_string()),
            options: Some(OneofOptions {
                uninterpreted_option: vec![oneof_option],
            }),
        }],
        options: Some(MessageOptions {
            uninterpreted_option: vec![message_option],
            ..Default::default()
        }),
        ..Default::default()
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/payment.proto".to_string()),
        package: Some("test".to_string()),
        message_type: vec![payment_message],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/payment.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none(), "should have no error");
    assert_eq!(response.file.len(), 1, "should generate one file");

    let content = response.file[0].content.as_ref().unwrap();

    // Check regular fields
    assert!(content.contains("pub id: i64"), "should have id field");
    assert!(
        content.contains("pub amount: f64"),
        "should have amount field"
    );

    // Check oneof fields are flattened and nullable
    assert!(
        content.contains("credit_card_number") && content.contains("Option<String>"),
        "should have credit_card_number as Option<String>"
    );
    assert!(
        content.contains("bank_account") && content.contains("Option<String>"),
        "should have bank_account as Option<String>"
    );

    // Check that nullable attribute is present
    assert!(
        content.contains("nullable"),
        "should have nullable attribute for oneof fields"
    );
}

#[test]
fn test_generate_entity_with_oneof_json() {
    let message_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.model".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("table_name: \"events\"".to_string()),
        ..Default::default()
    };

    let pk_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.column".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("primary_key: true".to_string()),
        ..Default::default()
    };

    // Create the seaorm.oneof option for JSON strategy
    let oneof_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.oneof".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("strategy: \"json\"".to_string()),
        ..Default::default()
    };

    let event_message = DescriptorProto {
        name: Some("Event".to_string()),
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
            // Oneof fields
            FieldDescriptorProto {
                name: Some("click".to_string()),
                number: Some(2),
                r#type: Some(Type::String.into()),
                oneof_index: Some(0),
                ..Default::default()
            },
            FieldDescriptorProto {
                name: Some("purchase".to_string()),
                number: Some(3),
                r#type: Some(Type::String.into()),
                oneof_index: Some(0),
                ..Default::default()
            },
        ],
        oneof_decl: vec![OneofDescriptorProto {
            name: Some("event_type".to_string()),
            options: Some(OneofOptions {
                uninterpreted_option: vec![oneof_option],
            }),
        }],
        options: Some(MessageOptions {
            uninterpreted_option: vec![message_option],
            ..Default::default()
        }),
        ..Default::default()
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/event.proto".to_string()),
        package: Some("test".to_string()),
        message_type: vec![event_message],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/event.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none());
    assert_eq!(response.file.len(), 1);

    let content = response.file[0].content.as_ref().unwrap();

    // Check that JSON column is created for the oneof
    assert!(
        content.contains("event_type"),
        "should have event_type field for JSON oneof"
    );
    assert!(content.contains("Json"), "should have Json column type");
}

#[test]
fn test_generate_entity_with_message_level_relations() {
    // Create the seaorm.model option with relations
    let message_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.model".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some(
            r#"table_name: "users", relations: [
                {name: "posts", type: RELATION_TYPE_HAS_MANY, related: "post"},
                {name: "profile", type: RELATION_TYPE_HAS_ONE, related: "profile"}
            ]"#
            .to_string(),
        ),
        ..Default::default()
    };

    let pk_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.column".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("primary_key: true".to_string()),
        ..Default::default()
    };

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
                name: Some("name".to_string()),
                number: Some(2),
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

    let file_descriptor = FileDescriptorProto {
        name: Some("test/user_assoc.proto".to_string()),
        package: Some("test".to_string()),
        message_type: vec![user_message],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/user_assoc.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none(), "should have no error");
    assert_eq!(response.file.len(), 1, "should generate one file");

    let content = response.file[0].content.as_ref().unwrap();

    // Check that relations are generated as dense format fields
    assert!(
        content.contains("pub posts: HasMany<"),
        "should have posts relation field"
    );
    assert!(
        content.contains("pub profile: HasOne<"),
        "should have profile relation field"
    );
    assert!(
        content.contains("has_many"),
        "should have has_many attribute"
    );
    assert!(content.contains("has_one"), "should have has_one attribute");
}

#[test]
fn test_generate_entity_with_belongs_to_relation() {
    let message_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.model".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some(
            r#"table_name: "posts", relations: [
                {name: "author", type: RELATION_TYPE_BELONGS_TO, related: "user", foreign_key: "author_id"}
            ]"#
            .to_string(),
        ),
        ..Default::default()
    };

    let pk_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.column".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("primary_key: true".to_string()),
        ..Default::default()
    };

    let post_message = DescriptorProto {
        name: Some("Post".to_string()),
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
                name: Some("title".to_string()),
                number: Some(2),
                r#type: Some(Type::String.into()),
                ..Default::default()
            },
            FieldDescriptorProto {
                name: Some("author_id".to_string()),
                number: Some(3),
                r#type: Some(Type::Int64.into()),
                ..Default::default()
            },
        ],
        options: Some(MessageOptions {
            uninterpreted_option: vec![message_option],
            ..Default::default()
        }),
        ..Default::default()
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/post_assoc.proto".to_string()),
        package: Some("test".to_string()),
        message_type: vec![post_message],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/post_assoc.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none());
    assert_eq!(response.file.len(), 1);

    let content = response.file[0].content.as_ref().unwrap();

    // Check for belongs_to relation in dense format (uses HasOne type)
    assert!(
        content.contains("pub author: HasOne<"),
        "should have author relation field"
    );
    assert!(
        content.contains("belongs_to"),
        "should have belongs_to attribute"
    );
    assert!(
        content.contains("author_id"),
        "should reference author_id column"
    );
}

#[test]
fn test_generate_entity_with_many_to_many_relation() {
    let message_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.model".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some(
            r#"table_name: "tags", relations: [
                {name: "posts", type: RELATION_TYPE_MANY_TO_MANY, related: "post", through: "post_tags"}
            ]"#
            .to_string(),
        ),
        ..Default::default()
    };

    let pk_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.column".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("primary_key: true".to_string()),
        ..Default::default()
    };

    let tag_message = DescriptorProto {
        name: Some("Tag".to_string()),
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
                name: Some("name".to_string()),
                number: Some(2),
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

    let file_descriptor = FileDescriptorProto {
        name: Some("test/tag.proto".to_string()),
        package: Some("test".to_string()),
        message_type: vec![tag_message],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/tag.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none());
    assert_eq!(response.file.len(), 1);

    let content = response.file[0].content.as_ref().unwrap();

    // Check for many_to_many relation in dense format (rendered as HasMany with via)
    assert!(
        content.contains("pub posts: HasMany<"),
        "should have posts relation field"
    );
    assert!(
        content.contains("has_many") && content.contains("via"),
        "should have has_many with via attribute"
    );
    assert!(
        content.contains("post_tags"),
        "should reference post_tags junction table"
    );
}

#[test]
fn test_generate_entity_with_embed_field() {
    let message_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.model".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("table_name: \"articles\"".to_string()),
        ..Default::default()
    };

    let pk_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.column".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("primary_key: true".to_string()),
        ..Default::default()
    };

    let embed_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.column".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("embed: true".to_string()),
        ..Default::default()
    };

    let embed_nullable_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.column".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("embed: true".to_string()),
        ..Default::default()
    };

    let article_message = DescriptorProto {
        name: Some("Article".to_string()),
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
                name: Some("title".to_string()),
                number: Some(2),
                r#type: Some(Type::String.into()),
                ..Default::default()
            },
            FieldDescriptorProto {
                name: Some("metadata".to_string()),
                number: Some(3),
                r#type: Some(Type::Message.into()),
                type_name: Some(".test.Metadata".to_string()),
                options: Some(prost_types::FieldOptions {
                    uninterpreted_option: vec![embed_option],
                    ..Default::default()
                }),
                ..Default::default()
            },
            FieldDescriptorProto {
                name: Some("extra".to_string()),
                number: Some(4),
                r#type: Some(Type::Message.into()),
                type_name: Some(".test.Metadata".to_string()),
                proto3_optional: Some(true),
                options: Some(prost_types::FieldOptions {
                    uninterpreted_option: vec![embed_nullable_option],
                    ..Default::default()
                }),
                ..Default::default()
            },
        ],
        options: Some(MessageOptions {
            uninterpreted_option: vec![message_option],
            ..Default::default()
        }),
        ..Default::default()
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/article.proto".to_string()),
        package: Some("test".to_string()),
        message_type: vec![article_message],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/article.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none());
    assert_eq!(response.file.len(), 1);

    let content = response.file[0].content.as_ref().unwrap();

    // Check for embedded fields with direct type (SeaORM 2.0 uses type directly with FromJsonQueryResult)
    assert!(
        content.contains("pub metadata: Metadata"),
        "should have metadata as Metadata type directly"
    );
    assert!(
        content.contains("JsonBinary"),
        "should have JsonBinary column type"
    );
    assert!(
        content.contains("pub extra: Option<Metadata>"),
        "should have extra as Option<Metadata>"
    );
}

// =============================================================================
// Service / Storage Trait Tests
// =============================================================================

/// Create a test CodeGeneratorRequest with a service
fn create_service_test_request() -> CodeGeneratorRequest {
    // Create the seaorm.service option
    let service_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.service".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("generate_storage: true".to_string()),
        ..Default::default()
    };

    // Create the service
    let user_service = ServiceDescriptorProto {
        name: Some("UserService".to_string()),
        method: vec![
            MethodDescriptorProto {
                name: Some("GetUser".to_string()),
                input_type: Some(".test.GetUserRequest".to_string()),
                output_type: Some(".test.User".to_string()),
                ..Default::default()
            },
            MethodDescriptorProto {
                name: Some("CreateUser".to_string()),
                input_type: Some(".test.CreateUserRequest".to_string()),
                output_type: Some(".test.User".to_string()),
                ..Default::default()
            },
            MethodDescriptorProto {
                name: Some("ListUsers".to_string()),
                input_type: Some(".test.ListUsersRequest".to_string()),
                output_type: Some(".test.ListUsersResponse".to_string()),
                ..Default::default()
            },
        ],
        options: Some(ServiceOptions {
            uninterpreted_option: vec![service_option],
            ..Default::default()
        }),
    };

    // Create the file descriptor
    let file_descriptor = FileDescriptorProto {
        name: Some("test/user_service.proto".to_string()),
        package: Some("test".to_string()),
        service: vec![user_service],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    CodeGeneratorRequest {
        file_to_generate: vec!["test/user_service.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    }
}

#[test]
fn test_generate_storage_trait() {
    let request = create_service_test_request();
    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    // Should have no error
    assert!(response.error.is_none(), "should have no error");

    // Should generate one file
    assert_eq!(response.file.len(), 1, "should generate one file");

    let file = &response.file[0];
    assert!(
        file.name
            .as_ref()
            .unwrap()
            .ends_with("user_service_storage.rs"),
        "file should be named user_service_storage.rs"
    );

    let content = file.content.as_ref().unwrap();

    // Check for trait definition
    assert!(
        content.contains("pub trait UserServiceStorage"),
        "should have UserServiceStorage trait"
    );

    // Check for async_trait
    assert!(
        content.contains("async_trait"),
        "should use async_trait attribute"
    );

    // Check for StorageError
    assert!(
        content.contains("StorageError"),
        "should have StorageError enum"
    );
    assert!(
        content.contains("Database"),
        "should have Database error variant"
    );
    assert!(
        content.contains("NotFound"),
        "should have NotFound error variant"
    );

    // Check for method signatures
    assert!(content.contains("get_user"), "should have get_user method");
    assert!(
        content.contains("create_user"),
        "should have create_user method"
    );
    assert!(
        content.contains("list_users"),
        "should have list_users method"
    );

    // Check for request/response types
    assert!(
        content.contains("GetUserRequest"),
        "should reference GetUserRequest"
    );
    assert!(content.contains("User"), "should reference User type");
    assert!(
        content.contains("ListUsersResponse"),
        "should reference ListUsersResponse"
    );

    // Check for Result return type
    assert!(content.contains("Result<"), "should return Result type");
}

#[test]
fn test_skip_service_without_options() {
    // Create a service without seaorm options
    let service = ServiceDescriptorProto {
        name: Some("InternalService".to_string()),
        method: vec![MethodDescriptorProto {
            name: Some("Ping".to_string()),
            input_type: Some(".test.Request".to_string()),
            output_type: Some(".test.Response".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/internal.proto".to_string()),
        package: Some("test".to_string()),
        service: vec![service],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/internal.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none());
    assert_eq!(
        response.file.len(),
        0,
        "should generate no files for services without seaorm options"
    );
}

#[test]
fn test_generate_storage_with_custom_trait_name() {
    let service_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.service".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("generate_storage: true, trait_name: \"AccountStore\"".to_string()),
        ..Default::default()
    };

    let service = ServiceDescriptorProto {
        name: Some("AccountService".to_string()),
        method: vec![MethodDescriptorProto {
            name: Some("GetAccount".to_string()),
            input_type: Some(".test.GetAccountRequest".to_string()),
            output_type: Some(".test.Account".to_string()),
            ..Default::default()
        }],
        options: Some(ServiceOptions {
            uninterpreted_option: vec![service_option],
            ..Default::default()
        }),
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/account.proto".to_string()),
        package: Some("test".to_string()),
        service: vec![service],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/account.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none());
    assert_eq!(response.file.len(), 1);

    let file = &response.file[0];
    assert!(
        file.name.as_ref().unwrap().ends_with("account_store.rs"),
        "file should be named account_store.rs"
    );

    let content = file.content.as_ref().unwrap();
    assert!(
        content.contains("pub trait AccountStore"),
        "should have custom trait name AccountStore"
    );
}

// =============================================================================
// Domain Type / Input Validation Tests
// =============================================================================

/// Create a test request for domain type generation with input_message options
fn create_domain_type_test_request() -> CodeGeneratorRequest {
    // Create the input_message option for domain type generation
    let input_message_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.input_message".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("domain_type: \"CreateUser\", generate_try_from: true".to_string()),
        ..Default::default()
    };

    // Create input option for email validation
    let email_input_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.input".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("validate: { email: true }".to_string()),
        ..Default::default()
    };

    // Create input option for length validation
    let length_input_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.input".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("validate: { length: { min: 1, max: 100 } }".to_string()),
        ..Default::default()
    };

    // Create the CreateUserRequest message
    let create_user_request = DescriptorProto {
        name: Some("CreateUserRequest".to_string()),
        field: vec![
            FieldDescriptorProto {
                name: Some("email".to_string()),
                number: Some(1),
                r#type: Some(Type::String.into()),
                options: Some(prost_types::FieldOptions {
                    uninterpreted_option: vec![email_input_option],
                    ..Default::default()
                }),
                ..Default::default()
            },
            FieldDescriptorProto {
                name: Some("name".to_string()),
                number: Some(2),
                r#type: Some(Type::String.into()),
                options: Some(prost_types::FieldOptions {
                    uninterpreted_option: vec![length_input_option],
                    ..Default::default()
                }),
                ..Default::default()
            },
        ],
        options: Some(MessageOptions {
            uninterpreted_option: vec![input_message_option],
            ..Default::default()
        }),
        ..Default::default()
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/request.proto".to_string()),
        package: Some("test".to_string()),
        message_type: vec![create_user_request],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    CodeGeneratorRequest {
        file_to_generate: vec!["test/request.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    }
}

#[test]
fn test_generate_domain_type() {
    let request = create_domain_type_test_request();
    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none(), "should have no error");
    assert_eq!(response.file.len(), 1, "should generate one file");

    let file = &response.file[0];
    assert!(
        file.name.as_ref().unwrap().ends_with("create_user.rs"),
        "file should be named create_user.rs"
    );

    let content = file.content.as_ref().unwrap();

    // Check for domain struct
    assert!(
        content.contains("pub struct CreateUser"),
        "should have CreateUser struct"
    );

    // Check for garde derive
    assert!(
        content.contains("garde::Validate"),
        "should have garde::Validate derive"
    );

    // Check for email validation
    assert!(
        content.contains("#[garde(email)]"),
        "should have email validation"
    );

    // Check for length validation
    // Debug: print content for debugging
    if !content.contains("garde(length(min = 1u32, max = 100u32))") {
        eprintln!("Generated content:\n{}", content);
    }
    assert!(
        content.contains("garde(length(min = 1u32, max = 100u32))"),
        "should have length validation with correct u32 type"
    );

    // Check for TryFrom implementation
    assert!(
        content.contains("impl TryFrom<CreateUserRequest>"),
        "should have TryFrom implementation"
    );

    // Check for DomainError
    assert!(
        content.contains("pub enum DomainError"),
        "should have DomainError enum"
    );

    // Check for validate call
    assert!(
        content.contains("domain.validate()"),
        "should call validate()"
    );
}

#[test]
fn test_generate_domain_type_with_range_validation() {
    // Create input_message option
    let input_message_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.input_message".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("domain_type: \"GetUser\", generate_try_from: true".to_string()),
        ..Default::default()
    };

    // Create input option for range validation on i64 field
    let range_i64_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.input".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("validate: { range: { min: 1 } }".to_string()),
        ..Default::default()
    };

    // Create input option for range validation on i32 field
    let range_i32_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.input".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("validate: { range: { min: 0, max: 100 } }".to_string()),
        ..Default::default()
    };

    let message = DescriptorProto {
        name: Some("GetUserRequest".to_string()),
        field: vec![
            FieldDescriptorProto {
                name: Some("id".to_string()),
                number: Some(1),
                r#type: Some(Type::Int64.into()),
                options: Some(prost_types::FieldOptions {
                    uninterpreted_option: vec![range_i64_option],
                    ..Default::default()
                }),
                ..Default::default()
            },
            FieldDescriptorProto {
                name: Some("page".to_string()),
                number: Some(2),
                r#type: Some(Type::Int32.into()),
                options: Some(prost_types::FieldOptions {
                    uninterpreted_option: vec![range_i32_option],
                    ..Default::default()
                }),
                ..Default::default()
            },
        ],
        options: Some(MessageOptions {
            uninterpreted_option: vec![input_message_option],
            ..Default::default()
        }),
        ..Default::default()
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/get_user.proto".to_string()),
        package: Some("test".to_string()),
        message_type: vec![message],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/get_user.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none());
    assert_eq!(response.file.len(), 1);

    let content = response.file[0].content.as_ref().unwrap();

    // Check for correct i64 range type
    assert!(
        content.contains("range(min = 1i64)"),
        "should have i64 range for int64 field"
    );

    // Check for correct i32 range type
    assert!(
        content.contains("range(min = 0i32, max = 100i32)"),
        "should have i32 range for int32 field"
    );
}

#[test]
fn test_skip_domain_type_without_input_options() {
    // Create a message without input_message options
    let message = DescriptorProto {
        name: Some("PlainRequest".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("field".to_string()),
            number: Some(1),
            r#type: Some(Type::String.into()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/plain.proto".to_string()),
        package: Some("test".to_string()),
        message_type: vec![message],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/plain.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none());
    // No files should be generated - no seaorm.model or seaorm.input_message
    assert_eq!(
        response.file.len(),
        0,
        "should not generate domain type for messages without input options"
    );
}

#[test]
fn test_generate_domain_type_with_multiple_validations() {
    let input_message_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.input_message".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("domain_type: \"RegisterUser\", generate_try_from: true".to_string()),
        ..Default::default()
    };

    // URL validation
    let url_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.input".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("validate: { url: true }".to_string()),
        ..Default::default()
    };

    // ASCII validation
    let ascii_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.input".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("validate: { ascii: true }".to_string()),
        ..Default::default()
    };

    // Pattern validation
    let pattern_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.input".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some(r#"validate: { pattern: "^[a-z]+$" }"#.to_string()),
        ..Default::default()
    };

    let message = DescriptorProto {
        name: Some("RegisterUserRequest".to_string()),
        field: vec![
            FieldDescriptorProto {
                name: Some("website".to_string()),
                number: Some(1),
                r#type: Some(Type::String.into()),
                options: Some(prost_types::FieldOptions {
                    uninterpreted_option: vec![url_option],
                    ..Default::default()
                }),
                ..Default::default()
            },
            FieldDescriptorProto {
                name: Some("username".to_string()),
                number: Some(2),
                r#type: Some(Type::String.into()),
                options: Some(prost_types::FieldOptions {
                    uninterpreted_option: vec![ascii_option],
                    ..Default::default()
                }),
                ..Default::default()
            },
            FieldDescriptorProto {
                name: Some("slug".to_string()),
                number: Some(3),
                r#type: Some(Type::String.into()),
                options: Some(prost_types::FieldOptions {
                    uninterpreted_option: vec![pattern_option],
                    ..Default::default()
                }),
                ..Default::default()
            },
        ],
        options: Some(MessageOptions {
            uninterpreted_option: vec![input_message_option],
            ..Default::default()
        }),
        ..Default::default()
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/register.proto".to_string()),
        package: Some("test".to_string()),
        message_type: vec![message],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/register.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none());
    assert_eq!(response.file.len(), 1);

    let content = response.file[0].content.as_ref().unwrap();

    // Check for URL validation
    assert!(
        content.contains("#[garde(url)]"),
        "should have url validation"
    );

    // Check for ASCII validation
    assert!(
        content.contains("#[garde(ascii)]"),
        "should have ascii validation"
    );

    // Check for pattern validation
    assert!(
        content.contains("garde(pattern("),
        "should have pattern validation"
    );
}

#[test]
fn test_generate_domain_type_without_try_from() {
    // Create input_message option without generate_try_from
    let input_message_option = UninterpretedOption {
        name: vec![NamePart {
            name_part: "seaorm.input_message".to_string(),
            is_extension: true,
        }],
        aggregate_value: Some("domain_type: \"QueryParams\"".to_string()),
        ..Default::default()
    };

    let message = DescriptorProto {
        name: Some("QueryRequest".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("query".to_string()),
            number: Some(1),
            r#type: Some(Type::String.into()),
            ..Default::default()
        }],
        options: Some(MessageOptions {
            uninterpreted_option: vec![input_message_option],
            ..Default::default()
        }),
        ..Default::default()
    };

    let file_descriptor = FileDescriptorProto {
        name: Some("test/query.proto".to_string()),
        package: Some("test".to_string()),
        message_type: vec![message],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    let request = CodeGeneratorRequest {
        file_to_generate: vec!["test/query.proto".to_string()],
        proto_file: vec![file_descriptor],
        ..Default::default()
    };

    let response = protoc_gen_seaorm::generate(request).expect("generation should succeed");

    assert!(response.error.is_none());
    assert_eq!(response.file.len(), 1);

    let content = response.file[0].content.as_ref().unwrap();

    // Should have struct
    assert!(
        content.contains("pub struct QueryParams"),
        "should have QueryParams struct"
    );

    // Should NOT have TryFrom (generate_try_from defaults to false)
    assert!(
        !content.contains("impl TryFrom"),
        "should not have TryFrom when generate_try_from is false"
    );

    // Should NOT have DomainError
    assert!(
        !content.contains("DomainError"),
        "should not have DomainError when generate_try_from is false"
    );
}
