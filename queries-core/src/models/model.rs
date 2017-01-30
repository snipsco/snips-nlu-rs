// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(PartialEq,Clone,Default)]
pub struct Model {
    // message fields
    pub field_type: Model_Type,
    pub classifier_type: ::std::string::String,
    arguments: ::protobuf::RepeatedField<Argument>,
    features: ::protobuf::RepeatedField<Feature>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Model {}

impl Model {
    pub fn new() -> Model {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Model {
        static mut instance: ::protobuf::lazy::Lazy<Model> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Model,
        };
        unsafe {
            instance.get(Model::new)
        }
    }

    // .Model.Type type = 1;

    pub fn clear_field_type(&mut self) {
        self.field_type = Model_Type::INTENT_CLASSIFIER;
    }

    // Param is passed by value, moved
    pub fn set_field_type(&mut self, v: Model_Type) {
        self.field_type = v;
    }

    pub fn get_field_type(&self) -> Model_Type {
        self.field_type
    }

    fn get_field_type_for_reflect(&self) -> &Model_Type {
        &self.field_type
    }

    fn mut_field_type_for_reflect(&mut self) -> &mut Model_Type {
        &mut self.field_type
    }

    // string classifier_type = 2;

    pub fn clear_classifier_type(&mut self) {
        self.classifier_type.clear();
    }

    // Param is passed by value, moved
    pub fn set_classifier_type(&mut self, v: ::std::string::String) {
        self.classifier_type = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_classifier_type(&mut self) -> &mut ::std::string::String {
        &mut self.classifier_type
    }

    // Take field
    pub fn take_classifier_type(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.classifier_type, ::std::string::String::new())
    }

    pub fn get_classifier_type(&self) -> &str {
        &self.classifier_type
    }

    fn get_classifier_type_for_reflect(&self) -> &::std::string::String {
        &self.classifier_type
    }

    fn mut_classifier_type_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.classifier_type
    }

    // repeated .Argument arguments = 3;

    pub fn clear_arguments(&mut self) {
        self.arguments.clear();
    }

    // Param is passed by value, moved
    pub fn set_arguments(&mut self, v: ::protobuf::RepeatedField<Argument>) {
        self.arguments = v;
    }

    // Mutable pointer to the field.
    pub fn mut_arguments(&mut self) -> &mut ::protobuf::RepeatedField<Argument> {
        &mut self.arguments
    }

    // Take field
    pub fn take_arguments(&mut self) -> ::protobuf::RepeatedField<Argument> {
        ::std::mem::replace(&mut self.arguments, ::protobuf::RepeatedField::new())
    }

    pub fn get_arguments(&self) -> &[Argument] {
        &self.arguments
    }

    fn get_arguments_for_reflect(&self) -> &::protobuf::RepeatedField<Argument> {
        &self.arguments
    }

    fn mut_arguments_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<Argument> {
        &mut self.arguments
    }

    // repeated .Feature features = 4;

    pub fn clear_features(&mut self) {
        self.features.clear();
    }

    // Param is passed by value, moved
    pub fn set_features(&mut self, v: ::protobuf::RepeatedField<Feature>) {
        self.features = v;
    }

    // Mutable pointer to the field.
    pub fn mut_features(&mut self) -> &mut ::protobuf::RepeatedField<Feature> {
        &mut self.features
    }

    // Take field
    pub fn take_features(&mut self) -> ::protobuf::RepeatedField<Feature> {
        ::std::mem::replace(&mut self.features, ::protobuf::RepeatedField::new())
    }

    pub fn get_features(&self) -> &[Feature] {
        &self.features
    }

    fn get_features_for_reflect(&self) -> &::protobuf::RepeatedField<Feature> {
        &self.features
    }

    fn mut_features_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<Feature> {
        &mut self.features
    }
}

impl ::protobuf::Message for Model {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    };
                    let tmp = is.read_enum()?;
                    self.field_type = tmp;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.classifier_type)?;
                },
                3 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.arguments)?;
                },
                4 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.features)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.field_type != Model_Type::INTENT_CLASSIFIER {
            my_size += ::protobuf::rt::enum_size(1, self.field_type);
        };
        if !self.classifier_type.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.classifier_type);
        };
        for value in &self.arguments {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        for value in &self.features {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.field_type != Model_Type::INTENT_CLASSIFIER {
            os.write_enum(1, self.field_type.value())?;
        };
        if !self.classifier_type.is_empty() {
            os.write_string(2, &self.classifier_type)?;
        };
        for v in &self.arguments {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        for v in &self.features {
            os.write_tag(4, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Model {
    fn new() -> Model {
        Model::new()
    }

    fn descriptor_static(_: ::std::option::Option<Model>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeEnum<Model_Type>>(
                    "type",
                    Model::get_field_type_for_reflect,
                    Model::mut_field_type_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "classifier_type",
                    Model::get_classifier_type_for_reflect,
                    Model::mut_classifier_type_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Argument>>(
                    "arguments",
                    Model::get_arguments_for_reflect,
                    Model::mut_arguments_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Feature>>(
                    "features",
                    Model::get_features_for_reflect,
                    Model::mut_features_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Model>(
                    "Model",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Model {
    fn clear(&mut self) {
        self.clear_field_type();
        self.clear_classifier_type();
        self.clear_arguments();
        self.clear_features();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Model {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Model {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum Model_Type {
    INTENT_CLASSIFIER = 0,
    TOKENS_CLASSIFIER = 1,
}

impl ::protobuf::ProtobufEnum for Model_Type {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<Model_Type> {
        match value {
            0 => ::std::option::Option::Some(Model_Type::INTENT_CLASSIFIER),
            1 => ::std::option::Option::Some(Model_Type::TOKENS_CLASSIFIER),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [Model_Type] = &[
            Model_Type::INTENT_CLASSIFIER,
            Model_Type::TOKENS_CLASSIFIER,
        ];
        values
    }

    fn enum_descriptor_static(_: Option<Model_Type>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("Model_Type", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for Model_Type {
}

impl ::std::default::Default for Model_Type {
    fn default() -> Self {
        Model_Type::INTENT_CLASSIFIER
    }
}

impl ::protobuf::reflect::ProtobufValue for Model_Type {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Feature {
    // message fields
    pub function_name: ::std::string::String,
    pub domain_name: ::std::string::String,
    arguments: ::protobuf::RepeatedField<Argument>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Feature {}

impl Feature {
    pub fn new() -> Feature {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Feature {
        static mut instance: ::protobuf::lazy::Lazy<Feature> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Feature,
        };
        unsafe {
            instance.get(Feature::new)
        }
    }

    // string function_name = 1;

    pub fn clear_function_name(&mut self) {
        self.function_name.clear();
    }

    // Param is passed by value, moved
    pub fn set_function_name(&mut self, v: ::std::string::String) {
        self.function_name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_function_name(&mut self) -> &mut ::std::string::String {
        &mut self.function_name
    }

    // Take field
    pub fn take_function_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.function_name, ::std::string::String::new())
    }

    pub fn get_function_name(&self) -> &str {
        &self.function_name
    }

    fn get_function_name_for_reflect(&self) -> &::std::string::String {
        &self.function_name
    }

    fn mut_function_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.function_name
    }

    // string domain_name = 2;

    pub fn clear_domain_name(&mut self) {
        self.domain_name.clear();
    }

    // Param is passed by value, moved
    pub fn set_domain_name(&mut self, v: ::std::string::String) {
        self.domain_name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_domain_name(&mut self) -> &mut ::std::string::String {
        &mut self.domain_name
    }

    // Take field
    pub fn take_domain_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.domain_name, ::std::string::String::new())
    }

    pub fn get_domain_name(&self) -> &str {
        &self.domain_name
    }

    fn get_domain_name_for_reflect(&self) -> &::std::string::String {
        &self.domain_name
    }

    fn mut_domain_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.domain_name
    }

    // repeated .Argument arguments = 3;

    pub fn clear_arguments(&mut self) {
        self.arguments.clear();
    }

    // Param is passed by value, moved
    pub fn set_arguments(&mut self, v: ::protobuf::RepeatedField<Argument>) {
        self.arguments = v;
    }

    // Mutable pointer to the field.
    pub fn mut_arguments(&mut self) -> &mut ::protobuf::RepeatedField<Argument> {
        &mut self.arguments
    }

    // Take field
    pub fn take_arguments(&mut self) -> ::protobuf::RepeatedField<Argument> {
        ::std::mem::replace(&mut self.arguments, ::protobuf::RepeatedField::new())
    }

    pub fn get_arguments(&self) -> &[Argument] {
        &self.arguments
    }

    fn get_arguments_for_reflect(&self) -> &::protobuf::RepeatedField<Argument> {
        &self.arguments
    }

    fn mut_arguments_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<Argument> {
        &mut self.arguments
    }
}

impl ::protobuf::Message for Feature {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.function_name)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.domain_name)?;
                },
                3 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.arguments)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.function_name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.function_name);
        };
        if !self.domain_name.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.domain_name);
        };
        for value in &self.arguments {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.function_name.is_empty() {
            os.write_string(1, &self.function_name)?;
        };
        if !self.domain_name.is_empty() {
            os.write_string(2, &self.domain_name)?;
        };
        for v in &self.arguments {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Feature {
    fn new() -> Feature {
        Feature::new()
    }

    fn descriptor_static(_: ::std::option::Option<Feature>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "function_name",
                    Feature::get_function_name_for_reflect,
                    Feature::mut_function_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "domain_name",
                    Feature::get_domain_name_for_reflect,
                    Feature::mut_domain_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Argument>>(
                    "arguments",
                    Feature::get_arguments_for_reflect,
                    Feature::mut_arguments_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Feature>(
                    "Feature",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Feature {
    fn clear(&mut self) {
        self.clear_function_name();
        self.clear_domain_name();
        self.clear_arguments();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Feature {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Feature {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Argument {
    // message oneof groups
    value: ::std::option::Option<Argument_oneof_value>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Argument {}

#[derive(Clone,PartialEq)]
pub enum Argument_oneof_value {
    gazetteer(::std::string::String),
    str(::std::string::String),
    scalar(f64),
    matrix(Matrix),
}

impl Argument {
    pub fn new() -> Argument {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Argument {
        static mut instance: ::protobuf::lazy::Lazy<Argument> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Argument,
        };
        unsafe {
            instance.get(Argument::new)
        }
    }

    // string gazetteer = 1;

    pub fn clear_gazetteer(&mut self) {
        self.value = ::std::option::Option::None;
    }

    pub fn has_gazetteer(&self) -> bool {
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::gazetteer(..)) => true,
            _ => false,
        }
    }

    // Param is passed by value, moved
    pub fn set_gazetteer(&mut self, v: ::std::string::String) {
        self.value = ::std::option::Option::Some(Argument_oneof_value::gazetteer(v))
    }

    // Mutable pointer to the field.
    pub fn mut_gazetteer(&mut self) -> &mut ::std::string::String {
        if let ::std::option::Option::Some(Argument_oneof_value::gazetteer(_)) = self.value {
        } else {
            self.value = ::std::option::Option::Some(Argument_oneof_value::gazetteer(::std::string::String::new()));
        }
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::gazetteer(ref mut v)) => v,
            _ => panic!(),
        }
    }

    // Take field
    pub fn take_gazetteer(&mut self) -> ::std::string::String {
        if self.has_gazetteer() {
            match self.value.take() {
                ::std::option::Option::Some(Argument_oneof_value::gazetteer(v)) => v,
                _ => panic!(),
            }
        } else {
            ::std::string::String::new()
        }
    }

    pub fn get_gazetteer(&self) -> &str {
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::gazetteer(ref v)) => v,
            _ => "",
        }
    }

    // string str = 2;

    pub fn clear_str(&mut self) {
        self.value = ::std::option::Option::None;
    }

    pub fn has_str(&self) -> bool {
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::str(..)) => true,
            _ => false,
        }
    }

    // Param is passed by value, moved
    pub fn set_str(&mut self, v: ::std::string::String) {
        self.value = ::std::option::Option::Some(Argument_oneof_value::str(v))
    }

    // Mutable pointer to the field.
    pub fn mut_str(&mut self) -> &mut ::std::string::String {
        if let ::std::option::Option::Some(Argument_oneof_value::str(_)) = self.value {
        } else {
            self.value = ::std::option::Option::Some(Argument_oneof_value::str(::std::string::String::new()));
        }
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::str(ref mut v)) => v,
            _ => panic!(),
        }
    }

    // Take field
    pub fn take_str(&mut self) -> ::std::string::String {
        if self.has_str() {
            match self.value.take() {
                ::std::option::Option::Some(Argument_oneof_value::str(v)) => v,
                _ => panic!(),
            }
        } else {
            ::std::string::String::new()
        }
    }

    pub fn get_str(&self) -> &str {
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::str(ref v)) => v,
            _ => "",
        }
    }

    // double scalar = 3;

    pub fn clear_scalar(&mut self) {
        self.value = ::std::option::Option::None;
    }

    pub fn has_scalar(&self) -> bool {
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::scalar(..)) => true,
            _ => false,
        }
    }

    // Param is passed by value, moved
    pub fn set_scalar(&mut self, v: f64) {
        self.value = ::std::option::Option::Some(Argument_oneof_value::scalar(v))
    }

    pub fn get_scalar(&self) -> f64 {
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::scalar(v)) => v,
            _ => 0.,
        }
    }

    // .Matrix matrix = 4;

    pub fn clear_matrix(&mut self) {
        self.value = ::std::option::Option::None;
    }

    pub fn has_matrix(&self) -> bool {
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::matrix(..)) => true,
            _ => false,
        }
    }

    // Param is passed by value, moved
    pub fn set_matrix(&mut self, v: Matrix) {
        self.value = ::std::option::Option::Some(Argument_oneof_value::matrix(v))
    }

    // Mutable pointer to the field.
    pub fn mut_matrix(&mut self) -> &mut Matrix {
        if let ::std::option::Option::Some(Argument_oneof_value::matrix(_)) = self.value {
        } else {
            self.value = ::std::option::Option::Some(Argument_oneof_value::matrix(Matrix::new()));
        }
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::matrix(ref mut v)) => v,
            _ => panic!(),
        }
    }

    // Take field
    pub fn take_matrix(&mut self) -> Matrix {
        if self.has_matrix() {
            match self.value.take() {
                ::std::option::Option::Some(Argument_oneof_value::matrix(v)) => v,
                _ => panic!(),
            }
        } else {
            Matrix::new()
        }
    }

    pub fn get_matrix(&self) -> &Matrix {
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::matrix(ref v)) => v,
            _ => Matrix::default_instance(),
        }
    }
}

impl ::protobuf::Message for Argument {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    };
                    self.value = ::std::option::Option::Some(Argument_oneof_value::gazetteer(is.read_string()?));
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    };
                    self.value = ::std::option::Option::Some(Argument_oneof_value::str(is.read_string()?));
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeFixed64 {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    };
                    self.value = ::std::option::Option::Some(Argument_oneof_value::scalar(is.read_double()?));
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    };
                    self.value = ::std::option::Option::Some(Argument_oneof_value::matrix(is.read_message()?));
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let ::std::option::Option::Some(ref v) = self.value {
            match v {
                &Argument_oneof_value::gazetteer(ref v) => {
                    my_size += ::protobuf::rt::string_size(1, &v);
                },
                &Argument_oneof_value::str(ref v) => {
                    my_size += ::protobuf::rt::string_size(2, &v);
                },
                &Argument_oneof_value::scalar(v) => {
                    my_size += 9;
                },
                &Argument_oneof_value::matrix(ref v) => {
                    let len = v.compute_size();
                    my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
                },
            };
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let ::std::option::Option::Some(ref v) = self.value {
            match v {
                &Argument_oneof_value::gazetteer(ref v) => {
                    os.write_string(1, v)?;
                },
                &Argument_oneof_value::str(ref v) => {
                    os.write_string(2, v)?;
                },
                &Argument_oneof_value::scalar(v) => {
                    os.write_double(3, v)?;
                },
                &Argument_oneof_value::matrix(ref v) => {
                    os.write_tag(4, ::protobuf::wire_format::WireTypeLengthDelimited)?;
                    os.write_raw_varint32(v.get_cached_size())?;
                    v.write_to_with_cached_sizes(os)?;
                },
            };
        };
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Argument {
    fn new() -> Argument {
        Argument::new()
    }

    fn descriptor_static(_: ::std::option::Option<Argument>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor::<_>(
                    "gazetteer",
                    Argument::has_gazetteer,
                    Argument::get_gazetteer,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor::<_>(
                    "str",
                    Argument::has_str,
                    Argument::get_str,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_f64_accessor::<_>(
                    "scalar",
                    Argument::has_scalar,
                    Argument::get_scalar,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_message_accessor::<_, Matrix>(
                    "matrix",
                    Argument::has_matrix,
                    Argument::get_matrix,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Argument>(
                    "Argument",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Argument {
    fn clear(&mut self) {
        self.clear_gazetteer();
        self.clear_str();
        self.clear_scalar();
        self.clear_matrix();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Argument {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Argument {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Matrix {
    // message fields
    pub rows: u32,
    pub cols: u32,
    buffer: ::std::vec::Vec<f64>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Matrix {}

impl Matrix {
    pub fn new() -> Matrix {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Matrix {
        static mut instance: ::protobuf::lazy::Lazy<Matrix> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Matrix,
        };
        unsafe {
            instance.get(Matrix::new)
        }
    }

    // uint32 rows = 1;

    pub fn clear_rows(&mut self) {
        self.rows = 0;
    }

    // Param is passed by value, moved
    pub fn set_rows(&mut self, v: u32) {
        self.rows = v;
    }

    pub fn get_rows(&self) -> u32 {
        self.rows
    }

    fn get_rows_for_reflect(&self) -> &u32 {
        &self.rows
    }

    fn mut_rows_for_reflect(&mut self) -> &mut u32 {
        &mut self.rows
    }

    // uint32 cols = 2;

    pub fn clear_cols(&mut self) {
        self.cols = 0;
    }

    // Param is passed by value, moved
    pub fn set_cols(&mut self, v: u32) {
        self.cols = v;
    }

    pub fn get_cols(&self) -> u32 {
        self.cols
    }

    fn get_cols_for_reflect(&self) -> &u32 {
        &self.cols
    }

    fn mut_cols_for_reflect(&mut self) -> &mut u32 {
        &mut self.cols
    }

    // repeated double buffer = 3;

    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    // Param is passed by value, moved
    pub fn set_buffer(&mut self, v: ::std::vec::Vec<f64>) {
        self.buffer = v;
    }

    // Mutable pointer to the field.
    pub fn mut_buffer(&mut self) -> &mut ::std::vec::Vec<f64> {
        &mut self.buffer
    }

    // Take field
    pub fn take_buffer(&mut self) -> ::std::vec::Vec<f64> {
        ::std::mem::replace(&mut self.buffer, ::std::vec::Vec::new())
    }

    pub fn get_buffer(&self) -> &[f64] {
        &self.buffer
    }

    fn get_buffer_for_reflect(&self) -> &::std::vec::Vec<f64> {
        &self.buffer
    }

    fn mut_buffer_for_reflect(&mut self) -> &mut ::std::vec::Vec<f64> {
        &mut self.buffer
    }
}

impl ::protobuf::Message for Matrix {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    };
                    let tmp = is.read_uint32()?;
                    self.rows = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    };
                    let tmp = is.read_uint32()?;
                    self.cols = tmp;
                },
                3 => {
                    ::protobuf::rt::read_repeated_double_into(wire_type, is, &mut self.buffer)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.rows != 0 {
            my_size += ::protobuf::rt::value_size(1, self.rows, ::protobuf::wire_format::WireTypeVarint);
        };
        if self.cols != 0 {
            my_size += ::protobuf::rt::value_size(2, self.cols, ::protobuf::wire_format::WireTypeVarint);
        };
        if !self.buffer.is_empty() {
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(self.buffer.len() as u32) + (self.buffer.len() * 8) as u32;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.rows != 0 {
            os.write_uint32(1, self.rows)?;
        };
        if self.cols != 0 {
            os.write_uint32(2, self.cols)?;
        };
        if !self.buffer.is_empty() {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            // TODO: Data size is computed again, it should be cached
            os.write_raw_varint32((self.buffer.len() * 8) as u32)?;
            for v in &self.buffer {
                os.write_double_no_tag(*v)?;
            };
        };
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Matrix {
    fn new() -> Matrix {
        Matrix::new()
    }

    fn descriptor_static(_: ::std::option::Option<Matrix>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "rows",
                    Matrix::get_rows_for_reflect,
                    Matrix::mut_rows_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "cols",
                    Matrix::get_cols_for_reflect,
                    Matrix::mut_cols_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_vec_accessor::<_, ::protobuf::types::ProtobufTypeDouble>(
                    "buffer",
                    Matrix::get_buffer_for_reflect,
                    Matrix::mut_buffer_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Matrix>(
                    "Matrix",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Matrix {
    fn clear(&mut self) {
        self.clear_rows();
        self.clear_cols();
        self.clear_buffer();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Matrix {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Matrix {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = &[
    0x0a, 0x0b, 0x6d, 0x6f, 0x64, 0x65, 0x6c, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x22, 0xd6, 0x01,
    0x0a, 0x05, 0x4d, 0x6f, 0x64, 0x65, 0x6c, 0x12, 0x1f, 0x0a, 0x04, 0x74, 0x79, 0x70, 0x65, 0x18,
    0x01, 0x20, 0x01, 0x28, 0x0e, 0x32, 0x0b, 0x2e, 0x4d, 0x6f, 0x64, 0x65, 0x6c, 0x2e, 0x54, 0x79,
    0x70, 0x65, 0x52, 0x04, 0x74, 0x79, 0x70, 0x65, 0x12, 0x27, 0x0a, 0x0f, 0x63, 0x6c, 0x61, 0x73,
    0x73, 0x69, 0x66, 0x69, 0x65, 0x72, 0x5f, 0x74, 0x79, 0x70, 0x65, 0x18, 0x02, 0x20, 0x01, 0x28,
    0x09, 0x52, 0x0e, 0x63, 0x6c, 0x61, 0x73, 0x73, 0x69, 0x66, 0x69, 0x65, 0x72, 0x54, 0x79, 0x70,
    0x65, 0x12, 0x27, 0x0a, 0x09, 0x61, 0x72, 0x67, 0x75, 0x6d, 0x65, 0x6e, 0x74, 0x73, 0x18, 0x03,
    0x20, 0x03, 0x28, 0x0b, 0x32, 0x09, 0x2e, 0x41, 0x72, 0x67, 0x75, 0x6d, 0x65, 0x6e, 0x74, 0x52,
    0x09, 0x61, 0x72, 0x67, 0x75, 0x6d, 0x65, 0x6e, 0x74, 0x73, 0x12, 0x24, 0x0a, 0x08, 0x66, 0x65,
    0x61, 0x74, 0x75, 0x72, 0x65, 0x73, 0x18, 0x04, 0x20, 0x03, 0x28, 0x0b, 0x32, 0x08, 0x2e, 0x46,
    0x65, 0x61, 0x74, 0x75, 0x72, 0x65, 0x52, 0x08, 0x66, 0x65, 0x61, 0x74, 0x75, 0x72, 0x65, 0x73,
    0x22, 0x34, 0x0a, 0x04, 0x54, 0x79, 0x70, 0x65, 0x12, 0x15, 0x0a, 0x11, 0x49, 0x4e, 0x54, 0x45,
    0x4e, 0x54, 0x5f, 0x43, 0x4c, 0x41, 0x53, 0x53, 0x49, 0x46, 0x49, 0x45, 0x52, 0x10, 0x00, 0x12,
    0x15, 0x0a, 0x11, 0x54, 0x4f, 0x4b, 0x45, 0x4e, 0x53, 0x5f, 0x43, 0x4c, 0x41, 0x53, 0x53, 0x49,
    0x46, 0x49, 0x45, 0x52, 0x10, 0x01, 0x22, 0x78, 0x0a, 0x07, 0x46, 0x65, 0x61, 0x74, 0x75, 0x72,
    0x65, 0x12, 0x23, 0x0a, 0x0d, 0x66, 0x75, 0x6e, 0x63, 0x74, 0x69, 0x6f, 0x6e, 0x5f, 0x6e, 0x61,
    0x6d, 0x65, 0x18, 0x01, 0x20, 0x01, 0x28, 0x09, 0x52, 0x0c, 0x66, 0x75, 0x6e, 0x63, 0x74, 0x69,
    0x6f, 0x6e, 0x4e, 0x61, 0x6d, 0x65, 0x12, 0x1f, 0x0a, 0x0b, 0x64, 0x6f, 0x6d, 0x61, 0x69, 0x6e,
    0x5f, 0x6e, 0x61, 0x6d, 0x65, 0x18, 0x02, 0x20, 0x01, 0x28, 0x09, 0x52, 0x0a, 0x64, 0x6f, 0x6d,
    0x61, 0x69, 0x6e, 0x4e, 0x61, 0x6d, 0x65, 0x12, 0x27, 0x0a, 0x09, 0x61, 0x72, 0x67, 0x75, 0x6d,
    0x65, 0x6e, 0x74, 0x73, 0x18, 0x03, 0x20, 0x03, 0x28, 0x0b, 0x32, 0x09, 0x2e, 0x41, 0x72, 0x67,
    0x75, 0x6d, 0x65, 0x6e, 0x74, 0x52, 0x09, 0x61, 0x72, 0x67, 0x75, 0x6d, 0x65, 0x6e, 0x74, 0x73,
    0x22, 0x84, 0x01, 0x0a, 0x08, 0x41, 0x72, 0x67, 0x75, 0x6d, 0x65, 0x6e, 0x74, 0x12, 0x1e, 0x0a,
    0x09, 0x67, 0x61, 0x7a, 0x65, 0x74, 0x74, 0x65, 0x65, 0x72, 0x18, 0x01, 0x20, 0x01, 0x28, 0x09,
    0x48, 0x00, 0x52, 0x09, 0x67, 0x61, 0x7a, 0x65, 0x74, 0x74, 0x65, 0x65, 0x72, 0x12, 0x12, 0x0a,
    0x03, 0x73, 0x74, 0x72, 0x18, 0x02, 0x20, 0x01, 0x28, 0x09, 0x48, 0x00, 0x52, 0x03, 0x73, 0x74,
    0x72, 0x12, 0x18, 0x0a, 0x06, 0x73, 0x63, 0x61, 0x6c, 0x61, 0x72, 0x18, 0x03, 0x20, 0x01, 0x28,
    0x01, 0x48, 0x00, 0x52, 0x06, 0x73, 0x63, 0x61, 0x6c, 0x61, 0x72, 0x12, 0x21, 0x0a, 0x06, 0x6d,
    0x61, 0x74, 0x72, 0x69, 0x78, 0x18, 0x04, 0x20, 0x01, 0x28, 0x0b, 0x32, 0x07, 0x2e, 0x4d, 0x61,
    0x74, 0x72, 0x69, 0x78, 0x48, 0x00, 0x52, 0x06, 0x6d, 0x61, 0x74, 0x72, 0x69, 0x78, 0x42, 0x07,
    0x0a, 0x05, 0x76, 0x61, 0x6c, 0x75, 0x65, 0x22, 0x4c, 0x0a, 0x06, 0x4d, 0x61, 0x74, 0x72, 0x69,
    0x78, 0x12, 0x12, 0x0a, 0x04, 0x72, 0x6f, 0x77, 0x73, 0x18, 0x01, 0x20, 0x01, 0x28, 0x0d, 0x52,
    0x04, 0x72, 0x6f, 0x77, 0x73, 0x12, 0x12, 0x0a, 0x04, 0x63, 0x6f, 0x6c, 0x73, 0x18, 0x02, 0x20,
    0x01, 0x28, 0x0d, 0x52, 0x04, 0x63, 0x6f, 0x6c, 0x73, 0x12, 0x1a, 0x0a, 0x06, 0x62, 0x75, 0x66,
    0x66, 0x65, 0x72, 0x18, 0x03, 0x20, 0x03, 0x28, 0x01, 0x52, 0x06, 0x62, 0x75, 0x66, 0x66, 0x65,
    0x72, 0x42, 0x02, 0x10, 0x01, 0x4a, 0x86, 0x0a, 0x0a, 0x06, 0x12, 0x04, 0x00, 0x00, 0x21, 0x01,
    0x0a, 0x08, 0x0a, 0x01, 0x0c, 0x12, 0x03, 0x00, 0x00, 0x12, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x00,
    0x12, 0x04, 0x02, 0x00, 0x0c, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x00, 0x01, 0x12, 0x03, 0x02,
    0x08, 0x0d, 0x0a, 0x0c, 0x0a, 0x04, 0x04, 0x00, 0x04, 0x00, 0x12, 0x04, 0x03, 0x04, 0x06, 0x05,
    0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x04, 0x00, 0x01, 0x12, 0x03, 0x03, 0x09, 0x0d, 0x0a, 0x0d,
    0x0a, 0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x00, 0x12, 0x03, 0x04, 0x08, 0x1e, 0x0a, 0x0e, 0x0a,
    0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x04, 0x08, 0x19, 0x0a, 0x0e, 0x0a,
    0x07, 0x04, 0x00, 0x04, 0x00, 0x02, 0x00, 0x02, 0x12, 0x03, 0x04, 0x1c, 0x1d, 0x0a, 0x0d, 0x0a,
    0x06, 0x04, 0x00, 0x04, 0x00, 0x02, 0x01, 0x12, 0x03, 0x05, 0x08, 0x1e, 0x0a, 0x0e, 0x0a, 0x07,
    0x04, 0x00, 0x04, 0x00, 0x02, 0x01, 0x01, 0x12, 0x03, 0x05, 0x08, 0x19, 0x0a, 0x0e, 0x0a, 0x07,
    0x04, 0x00, 0x04, 0x00, 0x02, 0x01, 0x02, 0x12, 0x03, 0x05, 0x1c, 0x1d, 0x0a, 0x0b, 0x0a, 0x04,
    0x04, 0x00, 0x02, 0x00, 0x12, 0x03, 0x08, 0x04, 0x12, 0x0a, 0x0d, 0x0a, 0x05, 0x04, 0x00, 0x02,
    0x00, 0x04, 0x12, 0x04, 0x08, 0x04, 0x06, 0x05, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00,
    0x06, 0x12, 0x03, 0x08, 0x04, 0x08, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12,
    0x03, 0x08, 0x09, 0x0d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x08,
    0x10, 0x11, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x01, 0x12, 0x03, 0x09, 0x04, 0x1f, 0x0a,
    0x0d, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x04, 0x12, 0x04, 0x09, 0x04, 0x08, 0x12, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x05, 0x12, 0x03, 0x09, 0x04, 0x0a, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x00, 0x02, 0x01, 0x01, 0x12, 0x03, 0x09, 0x0b, 0x1a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00,
    0x02, 0x01, 0x03, 0x12, 0x03, 0x09, 0x1d, 0x1e, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x02,
    0x12, 0x03, 0x0a, 0x04, 0x24, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x04, 0x12, 0x03,
    0x0a, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x06, 0x12, 0x03, 0x0a, 0x0d,
    0x15, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x01, 0x12, 0x03, 0x0a, 0x16, 0x1f, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x03, 0x12, 0x03, 0x0a, 0x22, 0x23, 0x0a, 0x0b, 0x0a,
    0x04, 0x04, 0x00, 0x02, 0x03, 0x12, 0x03, 0x0b, 0x04, 0x22, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00,
    0x02, 0x03, 0x04, 0x12, 0x03, 0x0b, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x03,
    0x06, 0x12, 0x03, 0x0b, 0x0d, 0x14, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x03, 0x01, 0x12,
    0x03, 0x0b, 0x15, 0x1d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x03, 0x03, 0x12, 0x03, 0x0b,
    0x20, 0x21, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x01, 0x12, 0x04, 0x0e, 0x00, 0x12, 0x01, 0x0a, 0x0a,
    0x0a, 0x03, 0x04, 0x01, 0x01, 0x12, 0x03, 0x0e, 0x08, 0x0f, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01,
    0x02, 0x00, 0x12, 0x03, 0x0f, 0x04, 0x1d, 0x0a, 0x0d, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x04,
    0x12, 0x04, 0x0f, 0x04, 0x0e, 0x11, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x05, 0x12,
    0x03, 0x0f, 0x04, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x01, 0x12, 0x03, 0x0f,
    0x0b, 0x18, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x03, 0x12, 0x03, 0x0f, 0x1b, 0x1c,
    0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01, 0x02, 0x01, 0x12, 0x03, 0x10, 0x04, 0x1b, 0x0a, 0x0d, 0x0a,
    0x05, 0x04, 0x01, 0x02, 0x01, 0x04, 0x12, 0x04, 0x10, 0x04, 0x0f, 0x1d, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x01, 0x02, 0x01, 0x05, 0x12, 0x03, 0x10, 0x04, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01,
    0x02, 0x01, 0x01, 0x12, 0x03, 0x10, 0x0b, 0x16, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x01,
    0x03, 0x12, 0x03, 0x10, 0x19, 0x1a, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01, 0x02, 0x02, 0x12, 0x03,
    0x11, 0x04, 0x24, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x02, 0x04, 0x12, 0x03, 0x11, 0x04,
    0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x02, 0x06, 0x12, 0x03, 0x11, 0x0d, 0x15, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x02, 0x01, 0x12, 0x03, 0x11, 0x16, 0x1f, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x01, 0x02, 0x02, 0x03, 0x12, 0x03, 0x11, 0x22, 0x23, 0x0a, 0x0a, 0x0a, 0x02, 0x04,
    0x02, 0x12, 0x04, 0x14, 0x00, 0x1b, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x02, 0x01, 0x12, 0x03,
    0x14, 0x08, 0x10, 0x0a, 0x0c, 0x0a, 0x04, 0x04, 0x02, 0x08, 0x00, 0x12, 0x04, 0x15, 0x04, 0x1a,
    0x05, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x08, 0x00, 0x01, 0x12, 0x03, 0x15, 0x0a, 0x0f, 0x0a,
    0x0b, 0x0a, 0x04, 0x04, 0x02, 0x02, 0x00, 0x12, 0x03, 0x16, 0x08, 0x1d, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x02, 0x02, 0x00, 0x05, 0x12, 0x03, 0x16, 0x08, 0x0e, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02,
    0x02, 0x00, 0x01, 0x12, 0x03, 0x16, 0x0f, 0x18, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x00,
    0x03, 0x12, 0x03, 0x16, 0x1b, 0x1c, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x02, 0x02, 0x01, 0x12, 0x03,
    0x17, 0x08, 0x17, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x01, 0x05, 0x12, 0x03, 0x17, 0x08,
    0x0e, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x01, 0x01, 0x12, 0x03, 0x17, 0x0f, 0x12, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x01, 0x03, 0x12, 0x03, 0x17, 0x15, 0x16, 0x0a, 0x0b, 0x0a,
    0x04, 0x04, 0x02, 0x02, 0x02, 0x12, 0x03, 0x18, 0x08, 0x1a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02,
    0x02, 0x02, 0x05, 0x12, 0x03, 0x18, 0x08, 0x0e, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x02,
    0x01, 0x12, 0x03, 0x18, 0x0f, 0x15, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x02, 0x03, 0x12,
    0x03, 0x18, 0x18, 0x19, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x02, 0x02, 0x03, 0x12, 0x03, 0x19, 0x08,
    0x1a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x03, 0x06, 0x12, 0x03, 0x19, 0x08, 0x0e, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x03, 0x01, 0x12, 0x03, 0x19, 0x0f, 0x15, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x02, 0x02, 0x03, 0x03, 0x12, 0x03, 0x19, 0x18, 0x19, 0x0a, 0x0a, 0x0a, 0x02, 0x04,
    0x03, 0x12, 0x04, 0x1d, 0x00, 0x21, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x03, 0x01, 0x12, 0x03,
    0x1d, 0x08, 0x0e, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x03, 0x02, 0x00, 0x12, 0x03, 0x1e, 0x04, 0x14,
    0x0a, 0x0d, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x00, 0x04, 0x12, 0x04, 0x1e, 0x04, 0x1d, 0x10, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x00, 0x05, 0x12, 0x03, 0x1e, 0x04, 0x0a, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x03, 0x02, 0x00, 0x01, 0x12, 0x03, 0x1e, 0x0b, 0x0f, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x03, 0x02, 0x00, 0x03, 0x12, 0x03, 0x1e, 0x12, 0x13, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x03, 0x02,
    0x01, 0x12, 0x03, 0x1f, 0x04, 0x14, 0x0a, 0x0d, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x01, 0x04, 0x12,
    0x04, 0x1f, 0x04, 0x1e, 0x14, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x01, 0x05, 0x12, 0x03,
    0x1f, 0x04, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x01, 0x01, 0x12, 0x03, 0x1f, 0x0b,
    0x0f, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x01, 0x03, 0x12, 0x03, 0x1f, 0x12, 0x13, 0x0a,
    0x0b, 0x0a, 0x04, 0x04, 0x03, 0x02, 0x02, 0x12, 0x03, 0x20, 0x04, 0x2d, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x03, 0x02, 0x02, 0x04, 0x12, 0x03, 0x20, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03,
    0x02, 0x02, 0x05, 0x12, 0x03, 0x20, 0x0d, 0x13, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x02,
    0x01, 0x12, 0x03, 0x20, 0x14, 0x1a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x02, 0x03, 0x12,
    0x03, 0x20, 0x1d, 0x1e, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x02, 0x08, 0x12, 0x03, 0x20,
    0x1f, 0x2c, 0x0a, 0x0f, 0x0a, 0x08, 0x04, 0x03, 0x02, 0x02, 0x08, 0xe7, 0x07, 0x00, 0x12, 0x03,
    0x20, 0x20, 0x2b, 0x0a, 0x10, 0x0a, 0x09, 0x04, 0x03, 0x02, 0x02, 0x08, 0xe7, 0x07, 0x00, 0x02,
    0x12, 0x03, 0x20, 0x20, 0x26, 0x0a, 0x11, 0x0a, 0x0a, 0x04, 0x03, 0x02, 0x02, 0x08, 0xe7, 0x07,
    0x00, 0x02, 0x00, 0x12, 0x03, 0x20, 0x20, 0x26, 0x0a, 0x12, 0x0a, 0x0b, 0x04, 0x03, 0x02, 0x02,
    0x08, 0xe7, 0x07, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x20, 0x20, 0x26, 0x0a, 0x10, 0x0a, 0x09,
    0x04, 0x03, 0x02, 0x02, 0x08, 0xe7, 0x07, 0x00, 0x03, 0x12, 0x03, 0x20, 0x27, 0x2b, 0x62, 0x06,
    0x70, 0x72, 0x6f, 0x74, 0x6f, 0x33,
];

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
