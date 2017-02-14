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
pub struct Configuration {
    // message fields
    pub intent_name: ::std::string::String,
    pub intent_classifier_name: ::std::string::String,
    pub tokens_classifier_name: ::std::string::String,
    slots: ::protobuf::RepeatedField<Configuration_Slot>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Configuration {}

impl Configuration {
    pub fn new() -> Configuration {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Configuration {
        static mut instance: ::protobuf::lazy::Lazy<Configuration> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Configuration,
        };
        unsafe {
            instance.get(Configuration::new)
        }
    }

    // string intent_name = 1;

    pub fn clear_intent_name(&mut self) {
        self.intent_name.clear();
    }

    // Param is passed by value, moved
    pub fn set_intent_name(&mut self, v: ::std::string::String) {
        self.intent_name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_intent_name(&mut self) -> &mut ::std::string::String {
        &mut self.intent_name
    }

    // Take field
    pub fn take_intent_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.intent_name, ::std::string::String::new())
    }

    pub fn get_intent_name(&self) -> &str {
        &self.intent_name
    }

    fn get_intent_name_for_reflect(&self) -> &::std::string::String {
        &self.intent_name
    }

    fn mut_intent_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.intent_name
    }

    // string intent_classifier_name = 2;

    pub fn clear_intent_classifier_name(&mut self) {
        self.intent_classifier_name.clear();
    }

    // Param is passed by value, moved
    pub fn set_intent_classifier_name(&mut self, v: ::std::string::String) {
        self.intent_classifier_name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_intent_classifier_name(&mut self) -> &mut ::std::string::String {
        &mut self.intent_classifier_name
    }

    // Take field
    pub fn take_intent_classifier_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.intent_classifier_name, ::std::string::String::new())
    }

    pub fn get_intent_classifier_name(&self) -> &str {
        &self.intent_classifier_name
    }

    fn get_intent_classifier_name_for_reflect(&self) -> &::std::string::String {
        &self.intent_classifier_name
    }

    fn mut_intent_classifier_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.intent_classifier_name
    }

    // string tokens_classifier_name = 3;

    pub fn clear_tokens_classifier_name(&mut self) {
        self.tokens_classifier_name.clear();
    }

    // Param is passed by value, moved
    pub fn set_tokens_classifier_name(&mut self, v: ::std::string::String) {
        self.tokens_classifier_name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_tokens_classifier_name(&mut self) -> &mut ::std::string::String {
        &mut self.tokens_classifier_name
    }

    // Take field
    pub fn take_tokens_classifier_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.tokens_classifier_name, ::std::string::String::new())
    }

    pub fn get_tokens_classifier_name(&self) -> &str {
        &self.tokens_classifier_name
    }

    fn get_tokens_classifier_name_for_reflect(&self) -> &::std::string::String {
        &self.tokens_classifier_name
    }

    fn mut_tokens_classifier_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.tokens_classifier_name
    }

    // repeated .Configuration.Slot slots = 4;

    pub fn clear_slots(&mut self) {
        self.slots.clear();
    }

    // Param is passed by value, moved
    pub fn set_slots(&mut self, v: ::protobuf::RepeatedField<Configuration_Slot>) {
        self.slots = v;
    }

    // Mutable pointer to the field.
    pub fn mut_slots(&mut self) -> &mut ::protobuf::RepeatedField<Configuration_Slot> {
        &mut self.slots
    }

    // Take field
    pub fn take_slots(&mut self) -> ::protobuf::RepeatedField<Configuration_Slot> {
        ::std::mem::replace(&mut self.slots, ::protobuf::RepeatedField::new())
    }

    pub fn get_slots(&self) -> &[Configuration_Slot] {
        &self.slots
    }

    fn get_slots_for_reflect(&self) -> &::protobuf::RepeatedField<Configuration_Slot> {
        &self.slots
    }

    fn mut_slots_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<Configuration_Slot> {
        &mut self.slots
    }
}

impl ::protobuf::Message for Configuration {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.intent_name)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.intent_classifier_name)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.tokens_classifier_name)?;
                },
                4 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.slots)?;
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
        if !self.intent_name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.intent_name);
        };
        if !self.intent_classifier_name.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.intent_classifier_name);
        };
        if !self.tokens_classifier_name.is_empty() {
            my_size += ::protobuf::rt::string_size(3, &self.tokens_classifier_name);
        };
        for value in &self.slots {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.intent_name.is_empty() {
            os.write_string(1, &self.intent_name)?;
        };
        if !self.intent_classifier_name.is_empty() {
            os.write_string(2, &self.intent_classifier_name)?;
        };
        if !self.tokens_classifier_name.is_empty() {
            os.write_string(3, &self.tokens_classifier_name)?;
        };
        for v in &self.slots {
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

impl ::protobuf::MessageStatic for Configuration {
    fn new() -> Configuration {
        Configuration::new()
    }

    fn descriptor_static(_: ::std::option::Option<Configuration>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "intent_name",
                    Configuration::get_intent_name_for_reflect,
                    Configuration::mut_intent_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "intent_classifier_name",
                    Configuration::get_intent_classifier_name_for_reflect,
                    Configuration::mut_intent_classifier_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "tokens_classifier_name",
                    Configuration::get_tokens_classifier_name_for_reflect,
                    Configuration::mut_tokens_classifier_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Configuration_Slot>>(
                    "slots",
                    Configuration::get_slots_for_reflect,
                    Configuration::mut_slots_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Configuration>(
                    "Configuration",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Configuration {
    fn clear(&mut self) {
        self.clear_intent_name();
        self.clear_intent_classifier_name();
        self.clear_tokens_classifier_name();
        self.clear_slots();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Configuration {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Configuration {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Configuration_Slot {
    // message fields
    pub name: ::std::string::String,
    resolver: ::protobuf::SingularPtrField<Configuration_Resolver>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Configuration_Slot {}

impl Configuration_Slot {
    pub fn new() -> Configuration_Slot {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Configuration_Slot {
        static mut instance: ::protobuf::lazy::Lazy<Configuration_Slot> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Configuration_Slot,
        };
        unsafe {
            instance.get(Configuration_Slot::new)
        }
    }

    // string name = 1;

    pub fn clear_name(&mut self) {
        self.name.clear();
    }

    // Param is passed by value, moved
    pub fn set_name(&mut self, v: ::std::string::String) {
        self.name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_name(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // Take field
    pub fn take_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.name, ::std::string::String::new())
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn get_name_for_reflect(&self) -> &::std::string::String {
        &self.name
    }

    fn mut_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // .Configuration.Resolver resolver = 2;

    pub fn clear_resolver(&mut self) {
        self.resolver.clear();
    }

    pub fn has_resolver(&self) -> bool {
        self.resolver.is_some()
    }

    // Param is passed by value, moved
    pub fn set_resolver(&mut self, v: Configuration_Resolver) {
        self.resolver = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_resolver(&mut self) -> &mut Configuration_Resolver {
        if self.resolver.is_none() {
            self.resolver.set_default();
        };
        self.resolver.as_mut().unwrap()
    }

    // Take field
    pub fn take_resolver(&mut self) -> Configuration_Resolver {
        self.resolver.take().unwrap_or_else(|| Configuration_Resolver::new())
    }

    pub fn get_resolver(&self) -> &Configuration_Resolver {
        self.resolver.as_ref().unwrap_or_else(|| Configuration_Resolver::default_instance())
    }

    fn get_resolver_for_reflect(&self) -> &::protobuf::SingularPtrField<Configuration_Resolver> {
        &self.resolver
    }

    fn mut_resolver_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Configuration_Resolver> {
        &mut self.resolver
    }
}

impl ::protobuf::Message for Configuration_Slot {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.name)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.resolver)?;
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
        if !self.name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.name);
        };
        if let Some(v) = self.resolver.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.name.is_empty() {
            os.write_string(1, &self.name)?;
        };
        if let Some(v) = self.resolver.as_ref() {
            os.write_tag(2, ::protobuf::wire_format::WireTypeLengthDelimited)?;
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

impl ::protobuf::MessageStatic for Configuration_Slot {
    fn new() -> Configuration_Slot {
        Configuration_Slot::new()
    }

    fn descriptor_static(_: ::std::option::Option<Configuration_Slot>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "name",
                    Configuration_Slot::get_name_for_reflect,
                    Configuration_Slot::mut_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Configuration_Resolver>>(
                    "resolver",
                    Configuration_Slot::get_resolver_for_reflect,
                    Configuration_Slot::mut_resolver_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Configuration_Slot>(
                    "Configuration_Slot",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Configuration_Slot {
    fn clear(&mut self) {
        self.clear_name();
        self.clear_resolver();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Configuration_Slot {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Configuration_Slot {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Configuration_Resolver {
    // message fields
    pub name: ::std::string::String,
    arguments: ::protobuf::RepeatedField<Argument>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Configuration_Resolver {}

impl Configuration_Resolver {
    pub fn new() -> Configuration_Resolver {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Configuration_Resolver {
        static mut instance: ::protobuf::lazy::Lazy<Configuration_Resolver> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Configuration_Resolver,
        };
        unsafe {
            instance.get(Configuration_Resolver::new)
        }
    }

    // string name = 1;

    pub fn clear_name(&mut self) {
        self.name.clear();
    }

    // Param is passed by value, moved
    pub fn set_name(&mut self, v: ::std::string::String) {
        self.name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_name(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // Take field
    pub fn take_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.name, ::std::string::String::new())
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn get_name_for_reflect(&self) -> &::std::string::String {
        &self.name
    }

    fn mut_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // repeated .Argument arguments = 2;

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

impl ::protobuf::Message for Configuration_Resolver {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.name)?;
                },
                2 => {
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
        if !self.name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.name);
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
        if !self.name.is_empty() {
            os.write_string(1, &self.name)?;
        };
        for v in &self.arguments {
            os.write_tag(2, ::protobuf::wire_format::WireTypeLengthDelimited)?;
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

impl ::protobuf::MessageStatic for Configuration_Resolver {
    fn new() -> Configuration_Resolver {
        Configuration_Resolver::new()
    }

    fn descriptor_static(_: ::std::option::Option<Configuration_Resolver>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "name",
                    Configuration_Resolver::get_name_for_reflect,
                    Configuration_Resolver::mut_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Argument>>(
                    "arguments",
                    Configuration_Resolver::get_arguments_for_reflect,
                    Configuration_Resolver::mut_arguments_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Configuration_Resolver>(
                    "Configuration_Resolver",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Configuration_Resolver {
    fn clear(&mut self) {
        self.clear_name();
        self.clear_arguments();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Configuration_Resolver {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Configuration_Resolver {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

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
    pub field_type: Feature_Type,
    arguments: ::protobuf::RepeatedField<Argument>,
    // message oneof groups
    domain: ::std::option::Option<Feature_oneof_domain>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Feature {}

#[derive(Clone,PartialEq)]
pub enum Feature_oneof_domain {
    known_domain(Feature_Domain),
    other_domain(::std::string::String),
}

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

    // .Feature.Type type = 1;

    pub fn clear_field_type(&mut self) {
        self.field_type = Feature_Type::HAS_GAZETTEER_HITS;
    }

    // Param is passed by value, moved
    pub fn set_field_type(&mut self, v: Feature_Type) {
        self.field_type = v;
    }

    pub fn get_field_type(&self) -> Feature_Type {
        self.field_type
    }

    fn get_field_type_for_reflect(&self) -> &Feature_Type {
        &self.field_type
    }

    fn mut_field_type_for_reflect(&mut self) -> &mut Feature_Type {
        &mut self.field_type
    }

    // .Feature.Domain known_domain = 2;

    pub fn clear_known_domain(&mut self) {
        self.domain = ::std::option::Option::None;
    }

    pub fn has_known_domain(&self) -> bool {
        match self.domain {
            ::std::option::Option::Some(Feature_oneof_domain::known_domain(..)) => true,
            _ => false,
        }
    }

    // Param is passed by value, moved
    pub fn set_known_domain(&mut self, v: Feature_Domain) {
        self.domain = ::std::option::Option::Some(Feature_oneof_domain::known_domain(v))
    }

    pub fn get_known_domain(&self) -> Feature_Domain {
        match self.domain {
            ::std::option::Option::Some(Feature_oneof_domain::known_domain(v)) => v,
            _ => Feature_Domain::SHARED_SCALAR,
        }
    }

    // string other_domain = 3;

    pub fn clear_other_domain(&mut self) {
        self.domain = ::std::option::Option::None;
    }

    pub fn has_other_domain(&self) -> bool {
        match self.domain {
            ::std::option::Option::Some(Feature_oneof_domain::other_domain(..)) => true,
            _ => false,
        }
    }

    // Param is passed by value, moved
    pub fn set_other_domain(&mut self, v: ::std::string::String) {
        self.domain = ::std::option::Option::Some(Feature_oneof_domain::other_domain(v))
    }

    // Mutable pointer to the field.
    pub fn mut_other_domain(&mut self) -> &mut ::std::string::String {
        if let ::std::option::Option::Some(Feature_oneof_domain::other_domain(_)) = self.domain {
        } else {
            self.domain = ::std::option::Option::Some(Feature_oneof_domain::other_domain(::std::string::String::new()));
        }
        match self.domain {
            ::std::option::Option::Some(Feature_oneof_domain::other_domain(ref mut v)) => v,
            _ => panic!(),
        }
    }

    // Take field
    pub fn take_other_domain(&mut self) -> ::std::string::String {
        if self.has_other_domain() {
            match self.domain.take() {
                ::std::option::Option::Some(Feature_oneof_domain::other_domain(v)) => v,
                _ => panic!(),
            }
        } else {
            ::std::string::String::new()
        }
    }

    pub fn get_other_domain(&self) -> &str {
        match self.domain {
            ::std::option::Option::Some(Feature_oneof_domain::other_domain(ref v)) => v,
            _ => "",
        }
    }

    // repeated .Argument arguments = 4;

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
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    };
                    let tmp = is.read_enum()?;
                    self.field_type = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    };
                    self.domain = ::std::option::Option::Some(Feature_oneof_domain::known_domain(is.read_enum()?));
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    };
                    self.domain = ::std::option::Option::Some(Feature_oneof_domain::other_domain(is.read_string()?));
                },
                4 => {
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
        if self.field_type != Feature_Type::HAS_GAZETTEER_HITS {
            my_size += ::protobuf::rt::enum_size(1, self.field_type);
        };
        for value in &self.arguments {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        if let ::std::option::Option::Some(ref v) = self.domain {
            match v {
                &Feature_oneof_domain::known_domain(v) => {
                    my_size += ::protobuf::rt::enum_size(2, v);
                },
                &Feature_oneof_domain::other_domain(ref v) => {
                    my_size += ::protobuf::rt::string_size(3, &v);
                },
            };
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.field_type != Feature_Type::HAS_GAZETTEER_HITS {
            os.write_enum(1, self.field_type.value())?;
        };
        for v in &self.arguments {
            os.write_tag(4, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
        if let ::std::option::Option::Some(ref v) = self.domain {
            match v {
                &Feature_oneof_domain::known_domain(v) => {
                    os.write_enum(2, v.value())?;
                },
                &Feature_oneof_domain::other_domain(ref v) => {
                    os.write_string(3, v)?;
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
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeEnum<Feature_Type>>(
                    "type",
                    Feature::get_field_type_for_reflect,
                    Feature::mut_field_type_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_enum_accessor::<_, Feature_Domain>(
                    "known_domain",
                    Feature::has_known_domain,
                    Feature::get_known_domain,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor::<_>(
                    "other_domain",
                    Feature::has_other_domain,
                    Feature::get_other_domain,
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
        self.clear_field_type();
        self.clear_known_domain();
        self.clear_other_domain();
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

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum Feature_Type {
    HAS_GAZETTEER_HITS = 0,
    NGRAM_MATCHER = 1,
    IS_FIRST_WORD = 2,
    IS_LAST_WORD = 3,
    IS_CAPITALIZED = 4,
    IS_DATE = 5,
    CONTAINS_POSSESSIVE = 6,
}

impl ::protobuf::ProtobufEnum for Feature_Type {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<Feature_Type> {
        match value {
            0 => ::std::option::Option::Some(Feature_Type::HAS_GAZETTEER_HITS),
            1 => ::std::option::Option::Some(Feature_Type::NGRAM_MATCHER),
            2 => ::std::option::Option::Some(Feature_Type::IS_FIRST_WORD),
            3 => ::std::option::Option::Some(Feature_Type::IS_LAST_WORD),
            4 => ::std::option::Option::Some(Feature_Type::IS_CAPITALIZED),
            5 => ::std::option::Option::Some(Feature_Type::IS_DATE),
            6 => ::std::option::Option::Some(Feature_Type::CONTAINS_POSSESSIVE),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [Feature_Type] = &[
            Feature_Type::HAS_GAZETTEER_HITS,
            Feature_Type::NGRAM_MATCHER,
            Feature_Type::IS_FIRST_WORD,
            Feature_Type::IS_LAST_WORD,
            Feature_Type::IS_CAPITALIZED,
            Feature_Type::IS_DATE,
            Feature_Type::CONTAINS_POSSESSIVE,
        ];
        values
    }

    fn enum_descriptor_static(_: Option<Feature_Type>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("Feature_Type", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for Feature_Type {
}

impl ::std::default::Default for Feature_Type {
    fn default() -> Self {
        Feature_Type::HAS_GAZETTEER_HITS
    }
}

impl ::protobuf::reflect::ProtobufValue for Feature_Type {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum Feature_Domain {
    SHARED_SCALAR = 0,
    SHARED_VECTOR = 1,
}

impl ::protobuf::ProtobufEnum for Feature_Domain {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<Feature_Domain> {
        match value {
            0 => ::std::option::Option::Some(Feature_Domain::SHARED_SCALAR),
            1 => ::std::option::Option::Some(Feature_Domain::SHARED_VECTOR),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [Feature_Domain] = &[
            Feature_Domain::SHARED_SCALAR,
            Feature_Domain::SHARED_VECTOR,
        ];
        values
    }

    fn enum_descriptor_static(_: Option<Feature_Domain>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("Feature_Domain", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for Feature_Domain {
}

impl ::std::default::Default for Feature_Domain {
    fn default() -> Self {
        Feature_Domain::SHARED_SCALAR
    }
}

impl ::protobuf::reflect::ProtobufValue for Feature_Domain {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
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
    model(::std::string::String),
    boolean(bool),
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

    // string model = 5;

    pub fn clear_model(&mut self) {
        self.value = ::std::option::Option::None;
    }

    pub fn has_model(&self) -> bool {
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::model(..)) => true,
            _ => false,
        }
    }

    // Param is passed by value, moved
    pub fn set_model(&mut self, v: ::std::string::String) {
        self.value = ::std::option::Option::Some(Argument_oneof_value::model(v))
    }

    // Mutable pointer to the field.
    pub fn mut_model(&mut self) -> &mut ::std::string::String {
        if let ::std::option::Option::Some(Argument_oneof_value::model(_)) = self.value {
        } else {
            self.value = ::std::option::Option::Some(Argument_oneof_value::model(::std::string::String::new()));
        }
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::model(ref mut v)) => v,
            _ => panic!(),
        }
    }

    // Take field
    pub fn take_model(&mut self) -> ::std::string::String {
        if self.has_model() {
            match self.value.take() {
                ::std::option::Option::Some(Argument_oneof_value::model(v)) => v,
                _ => panic!(),
            }
        } else {
            ::std::string::String::new()
        }
    }

    pub fn get_model(&self) -> &str {
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::model(ref v)) => v,
            _ => "",
        }
    }

    // bool boolean = 6;

    pub fn clear_boolean(&mut self) {
        self.value = ::std::option::Option::None;
    }

    pub fn has_boolean(&self) -> bool {
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::boolean(..)) => true,
            _ => false,
        }
    }

    // Param is passed by value, moved
    pub fn set_boolean(&mut self, v: bool) {
        self.value = ::std::option::Option::Some(Argument_oneof_value::boolean(v))
    }

    pub fn get_boolean(&self) -> bool {
        match self.value {
            ::std::option::Option::Some(Argument_oneof_value::boolean(v)) => v,
            _ => false,
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
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    };
                    self.value = ::std::option::Option::Some(Argument_oneof_value::model(is.read_string()?));
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    };
                    self.value = ::std::option::Option::Some(Argument_oneof_value::boolean(is.read_bool()?));
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
                &Argument_oneof_value::model(ref v) => {
                    my_size += ::protobuf::rt::string_size(5, &v);
                },
                &Argument_oneof_value::boolean(v) => {
                    my_size += 2;
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
                &Argument_oneof_value::model(ref v) => {
                    os.write_string(5, v)?;
                },
                &Argument_oneof_value::boolean(v) => {
                    os.write_bool(6, v)?;
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
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor::<_>(
                    "model",
                    Argument::has_model,
                    Argument::get_model,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_bool_accessor::<_>(
                    "boolean",
                    Argument::has_boolean,
                    Argument::get_boolean,
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
        self.clear_model();
        self.clear_boolean();
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
    0x0a, 0x0b, 0x6d, 0x6f, 0x64, 0x65, 0x6c, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x22, 0xe1, 0x02,
    0x0a, 0x0d, 0x43, 0x6f, 0x6e, 0x66, 0x69, 0x67, 0x75, 0x72, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x12,
    0x1f, 0x0a, 0x0b, 0x69, 0x6e, 0x74, 0x65, 0x6e, 0x74, 0x5f, 0x6e, 0x61, 0x6d, 0x65, 0x18, 0x01,
    0x20, 0x01, 0x28, 0x09, 0x52, 0x0a, 0x69, 0x6e, 0x74, 0x65, 0x6e, 0x74, 0x4e, 0x61, 0x6d, 0x65,
    0x12, 0x34, 0x0a, 0x16, 0x69, 0x6e, 0x74, 0x65, 0x6e, 0x74, 0x5f, 0x63, 0x6c, 0x61, 0x73, 0x73,
    0x69, 0x66, 0x69, 0x65, 0x72, 0x5f, 0x6e, 0x61, 0x6d, 0x65, 0x18, 0x02, 0x20, 0x01, 0x28, 0x09,
    0x52, 0x14, 0x69, 0x6e, 0x74, 0x65, 0x6e, 0x74, 0x43, 0x6c, 0x61, 0x73, 0x73, 0x69, 0x66, 0x69,
    0x65, 0x72, 0x4e, 0x61, 0x6d, 0x65, 0x12, 0x34, 0x0a, 0x16, 0x74, 0x6f, 0x6b, 0x65, 0x6e, 0x73,
    0x5f, 0x63, 0x6c, 0x61, 0x73, 0x73, 0x69, 0x66, 0x69, 0x65, 0x72, 0x5f, 0x6e, 0x61, 0x6d, 0x65,
    0x18, 0x03, 0x20, 0x01, 0x28, 0x09, 0x52, 0x14, 0x74, 0x6f, 0x6b, 0x65, 0x6e, 0x73, 0x43, 0x6c,
    0x61, 0x73, 0x73, 0x69, 0x66, 0x69, 0x65, 0x72, 0x4e, 0x61, 0x6d, 0x65, 0x12, 0x29, 0x0a, 0x05,
    0x73, 0x6c, 0x6f, 0x74, 0x73, 0x18, 0x04, 0x20, 0x03, 0x28, 0x0b, 0x32, 0x13, 0x2e, 0x43, 0x6f,
    0x6e, 0x66, 0x69, 0x67, 0x75, 0x72, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x2e, 0x53, 0x6c, 0x6f, 0x74,
    0x52, 0x05, 0x73, 0x6c, 0x6f, 0x74, 0x73, 0x1a, 0x4f, 0x0a, 0x04, 0x53, 0x6c, 0x6f, 0x74, 0x12,
    0x12, 0x0a, 0x04, 0x6e, 0x61, 0x6d, 0x65, 0x18, 0x01, 0x20, 0x01, 0x28, 0x09, 0x52, 0x04, 0x6e,
    0x61, 0x6d, 0x65, 0x12, 0x33, 0x0a, 0x08, 0x72, 0x65, 0x73, 0x6f, 0x6c, 0x76, 0x65, 0x72, 0x18,
    0x02, 0x20, 0x01, 0x28, 0x0b, 0x32, 0x17, 0x2e, 0x43, 0x6f, 0x6e, 0x66, 0x69, 0x67, 0x75, 0x72,
    0x61, 0x74, 0x69, 0x6f, 0x6e, 0x2e, 0x52, 0x65, 0x73, 0x6f, 0x6c, 0x76, 0x65, 0x72, 0x52, 0x08,
    0x72, 0x65, 0x73, 0x6f, 0x6c, 0x76, 0x65, 0x72, 0x1a, 0x47, 0x0a, 0x08, 0x52, 0x65, 0x73, 0x6f,
    0x6c, 0x76, 0x65, 0x72, 0x12, 0x12, 0x0a, 0x04, 0x6e, 0x61, 0x6d, 0x65, 0x18, 0x01, 0x20, 0x01,
    0x28, 0x09, 0x52, 0x04, 0x6e, 0x61, 0x6d, 0x65, 0x12, 0x27, 0x0a, 0x09, 0x61, 0x72, 0x67, 0x75,
    0x6d, 0x65, 0x6e, 0x74, 0x73, 0x18, 0x02, 0x20, 0x03, 0x28, 0x0b, 0x32, 0x09, 0x2e, 0x41, 0x72,
    0x67, 0x75, 0x6d, 0x65, 0x6e, 0x74, 0x52, 0x09, 0x61, 0x72, 0x67, 0x75, 0x6d, 0x65, 0x6e, 0x74,
    0x73, 0x22, 0xd6, 0x01, 0x0a, 0x05, 0x4d, 0x6f, 0x64, 0x65, 0x6c, 0x12, 0x1f, 0x0a, 0x04, 0x74,
    0x79, 0x70, 0x65, 0x18, 0x01, 0x20, 0x01, 0x28, 0x0e, 0x32, 0x0b, 0x2e, 0x4d, 0x6f, 0x64, 0x65,
    0x6c, 0x2e, 0x54, 0x79, 0x70, 0x65, 0x52, 0x04, 0x74, 0x79, 0x70, 0x65, 0x12, 0x27, 0x0a, 0x0f,
    0x63, 0x6c, 0x61, 0x73, 0x73, 0x69, 0x66, 0x69, 0x65, 0x72, 0x5f, 0x74, 0x79, 0x70, 0x65, 0x18,
    0x02, 0x20, 0x01, 0x28, 0x09, 0x52, 0x0e, 0x63, 0x6c, 0x61, 0x73, 0x73, 0x69, 0x66, 0x69, 0x65,
    0x72, 0x54, 0x79, 0x70, 0x65, 0x12, 0x27, 0x0a, 0x09, 0x61, 0x72, 0x67, 0x75, 0x6d, 0x65, 0x6e,
    0x74, 0x73, 0x18, 0x03, 0x20, 0x03, 0x28, 0x0b, 0x32, 0x09, 0x2e, 0x41, 0x72, 0x67, 0x75, 0x6d,
    0x65, 0x6e, 0x74, 0x52, 0x09, 0x61, 0x72, 0x67, 0x75, 0x6d, 0x65, 0x6e, 0x74, 0x73, 0x12, 0x24,
    0x0a, 0x08, 0x66, 0x65, 0x61, 0x74, 0x75, 0x72, 0x65, 0x73, 0x18, 0x04, 0x20, 0x03, 0x28, 0x0b,
    0x32, 0x08, 0x2e, 0x46, 0x65, 0x61, 0x74, 0x75, 0x72, 0x65, 0x52, 0x08, 0x66, 0x65, 0x61, 0x74,
    0x75, 0x72, 0x65, 0x73, 0x22, 0x34, 0x0a, 0x04, 0x54, 0x79, 0x70, 0x65, 0x12, 0x15, 0x0a, 0x11,
    0x49, 0x4e, 0x54, 0x45, 0x4e, 0x54, 0x5f, 0x43, 0x4c, 0x41, 0x53, 0x53, 0x49, 0x46, 0x49, 0x45,
    0x52, 0x10, 0x00, 0x12, 0x15, 0x0a, 0x11, 0x54, 0x4f, 0x4b, 0x45, 0x4e, 0x53, 0x5f, 0x43, 0x4c,
    0x41, 0x53, 0x53, 0x49, 0x46, 0x49, 0x45, 0x52, 0x10, 0x01, 0x22, 0xfd, 0x02, 0x0a, 0x07, 0x46,
    0x65, 0x61, 0x74, 0x75, 0x72, 0x65, 0x12, 0x21, 0x0a, 0x04, 0x74, 0x79, 0x70, 0x65, 0x18, 0x01,
    0x20, 0x01, 0x28, 0x0e, 0x32, 0x0d, 0x2e, 0x46, 0x65, 0x61, 0x74, 0x75, 0x72, 0x65, 0x2e, 0x54,
    0x79, 0x70, 0x65, 0x52, 0x04, 0x74, 0x79, 0x70, 0x65, 0x12, 0x34, 0x0a, 0x0c, 0x6b, 0x6e, 0x6f,
    0x77, 0x6e, 0x5f, 0x64, 0x6f, 0x6d, 0x61, 0x69, 0x6e, 0x18, 0x02, 0x20, 0x01, 0x28, 0x0e, 0x32,
    0x0f, 0x2e, 0x46, 0x65, 0x61, 0x74, 0x75, 0x72, 0x65, 0x2e, 0x44, 0x6f, 0x6d, 0x61, 0x69, 0x6e,
    0x48, 0x00, 0x52, 0x0b, 0x6b, 0x6e, 0x6f, 0x77, 0x6e, 0x44, 0x6f, 0x6d, 0x61, 0x69, 0x6e, 0x12,
    0x23, 0x0a, 0x0c, 0x6f, 0x74, 0x68, 0x65, 0x72, 0x5f, 0x64, 0x6f, 0x6d, 0x61, 0x69, 0x6e, 0x18,
    0x03, 0x20, 0x01, 0x28, 0x09, 0x48, 0x00, 0x52, 0x0b, 0x6f, 0x74, 0x68, 0x65, 0x72, 0x44, 0x6f,
    0x6d, 0x61, 0x69, 0x6e, 0x12, 0x27, 0x0a, 0x09, 0x61, 0x72, 0x67, 0x75, 0x6d, 0x65, 0x6e, 0x74,
    0x73, 0x18, 0x04, 0x20, 0x03, 0x28, 0x0b, 0x32, 0x09, 0x2e, 0x41, 0x72, 0x67, 0x75, 0x6d, 0x65,
    0x6e, 0x74, 0x52, 0x09, 0x61, 0x72, 0x67, 0x75, 0x6d, 0x65, 0x6e, 0x74, 0x73, 0x22, 0x90, 0x01,
    0x0a, 0x04, 0x54, 0x79, 0x70, 0x65, 0x12, 0x16, 0x0a, 0x12, 0x48, 0x41, 0x53, 0x5f, 0x47, 0x41,
    0x5a, 0x45, 0x54, 0x54, 0x45, 0x45, 0x52, 0x5f, 0x48, 0x49, 0x54, 0x53, 0x10, 0x00, 0x12, 0x11,
    0x0a, 0x0d, 0x4e, 0x47, 0x52, 0x41, 0x4d, 0x5f, 0x4d, 0x41, 0x54, 0x43, 0x48, 0x45, 0x52, 0x10,
    0x01, 0x12, 0x11, 0x0a, 0x0d, 0x49, 0x53, 0x5f, 0x46, 0x49, 0x52, 0x53, 0x54, 0x5f, 0x57, 0x4f,
    0x52, 0x44, 0x10, 0x02, 0x12, 0x10, 0x0a, 0x0c, 0x49, 0x53, 0x5f, 0x4c, 0x41, 0x53, 0x54, 0x5f,
    0x57, 0x4f, 0x52, 0x44, 0x10, 0x03, 0x12, 0x12, 0x0a, 0x0e, 0x49, 0x53, 0x5f, 0x43, 0x41, 0x50,
    0x49, 0x54, 0x41, 0x4c, 0x49, 0x5a, 0x45, 0x44, 0x10, 0x04, 0x12, 0x0b, 0x0a, 0x07, 0x49, 0x53,
    0x5f, 0x44, 0x41, 0x54, 0x45, 0x10, 0x05, 0x12, 0x17, 0x0a, 0x13, 0x43, 0x4f, 0x4e, 0x54, 0x41,
    0x49, 0x4e, 0x53, 0x5f, 0x50, 0x4f, 0x53, 0x53, 0x45, 0x53, 0x53, 0x49, 0x56, 0x45, 0x10, 0x06,
    0x22, 0x2e, 0x0a, 0x06, 0x44, 0x6f, 0x6d, 0x61, 0x69, 0x6e, 0x12, 0x11, 0x0a, 0x0d, 0x53, 0x48,
    0x41, 0x52, 0x45, 0x44, 0x5f, 0x53, 0x43, 0x41, 0x4c, 0x41, 0x52, 0x10, 0x00, 0x12, 0x11, 0x0a,
    0x0d, 0x53, 0x48, 0x41, 0x52, 0x45, 0x44, 0x5f, 0x56, 0x45, 0x43, 0x54, 0x4f, 0x52, 0x10, 0x01,
    0x42, 0x08, 0x0a, 0x06, 0x64, 0x6f, 0x6d, 0x61, 0x69, 0x6e, 0x22, 0xb8, 0x01, 0x0a, 0x08, 0x41,
    0x72, 0x67, 0x75, 0x6d, 0x65, 0x6e, 0x74, 0x12, 0x1e, 0x0a, 0x09, 0x67, 0x61, 0x7a, 0x65, 0x74,
    0x74, 0x65, 0x65, 0x72, 0x18, 0x01, 0x20, 0x01, 0x28, 0x09, 0x48, 0x00, 0x52, 0x09, 0x67, 0x61,
    0x7a, 0x65, 0x74, 0x74, 0x65, 0x65, 0x72, 0x12, 0x12, 0x0a, 0x03, 0x73, 0x74, 0x72, 0x18, 0x02,
    0x20, 0x01, 0x28, 0x09, 0x48, 0x00, 0x52, 0x03, 0x73, 0x74, 0x72, 0x12, 0x18, 0x0a, 0x06, 0x73,
    0x63, 0x61, 0x6c, 0x61, 0x72, 0x18, 0x03, 0x20, 0x01, 0x28, 0x01, 0x48, 0x00, 0x52, 0x06, 0x73,
    0x63, 0x61, 0x6c, 0x61, 0x72, 0x12, 0x21, 0x0a, 0x06, 0x6d, 0x61, 0x74, 0x72, 0x69, 0x78, 0x18,
    0x04, 0x20, 0x01, 0x28, 0x0b, 0x32, 0x07, 0x2e, 0x4d, 0x61, 0x74, 0x72, 0x69, 0x78, 0x48, 0x00,
    0x52, 0x06, 0x6d, 0x61, 0x74, 0x72, 0x69, 0x78, 0x12, 0x16, 0x0a, 0x05, 0x6d, 0x6f, 0x64, 0x65,
    0x6c, 0x18, 0x05, 0x20, 0x01, 0x28, 0x09, 0x48, 0x00, 0x52, 0x05, 0x6d, 0x6f, 0x64, 0x65, 0x6c,
    0x12, 0x1a, 0x0a, 0x07, 0x62, 0x6f, 0x6f, 0x6c, 0x65, 0x61, 0x6e, 0x18, 0x06, 0x20, 0x01, 0x28,
    0x08, 0x48, 0x00, 0x52, 0x07, 0x62, 0x6f, 0x6f, 0x6c, 0x65, 0x61, 0x6e, 0x42, 0x07, 0x0a, 0x05,
    0x76, 0x61, 0x6c, 0x75, 0x65, 0x22, 0x4c, 0x0a, 0x06, 0x4d, 0x61, 0x74, 0x72, 0x69, 0x78, 0x12,
    0x12, 0x0a, 0x04, 0x72, 0x6f, 0x77, 0x73, 0x18, 0x01, 0x20, 0x01, 0x28, 0x0d, 0x52, 0x04, 0x72,
    0x6f, 0x77, 0x73, 0x12, 0x12, 0x0a, 0x04, 0x63, 0x6f, 0x6c, 0x73, 0x18, 0x02, 0x20, 0x01, 0x28,
    0x0d, 0x52, 0x04, 0x63, 0x6f, 0x6c, 0x73, 0x12, 0x1a, 0x0a, 0x06, 0x62, 0x75, 0x66, 0x66, 0x65,
    0x72, 0x18, 0x03, 0x20, 0x03, 0x28, 0x01, 0x52, 0x06, 0x62, 0x75, 0x66, 0x66, 0x65, 0x72, 0x42,
    0x02, 0x10, 0x01, 0x4a, 0xbd, 0x14, 0x0a, 0x06, 0x12, 0x04, 0x00, 0x00, 0x46, 0x01, 0x0a, 0x08,
    0x0a, 0x01, 0x0c, 0x12, 0x03, 0x00, 0x00, 0x12, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x00, 0x12, 0x04,
    0x02, 0x00, 0x11, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x00, 0x01, 0x12, 0x03, 0x02, 0x08, 0x15,
    0x0a, 0x0c, 0x0a, 0x04, 0x04, 0x00, 0x03, 0x00, 0x12, 0x04, 0x03, 0x04, 0x06, 0x05, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x00, 0x03, 0x00, 0x01, 0x12, 0x03, 0x03, 0x0c, 0x10, 0x0a, 0x0d, 0x0a, 0x06,
    0x04, 0x00, 0x03, 0x00, 0x02, 0x00, 0x12, 0x03, 0x04, 0x08, 0x18, 0x0a, 0x0f, 0x0a, 0x07, 0x04,
    0x00, 0x03, 0x00, 0x02, 0x00, 0x04, 0x12, 0x04, 0x04, 0x08, 0x03, 0x12, 0x0a, 0x0e, 0x0a, 0x07,
    0x04, 0x00, 0x03, 0x00, 0x02, 0x00, 0x05, 0x12, 0x03, 0x04, 0x08, 0x0e, 0x0a, 0x0e, 0x0a, 0x07,
    0x04, 0x00, 0x03, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x04, 0x0f, 0x13, 0x0a, 0x0e, 0x0a, 0x07,
    0x04, 0x00, 0x03, 0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x04, 0x16, 0x17, 0x0a, 0x0d, 0x0a, 0x06,
    0x04, 0x00, 0x03, 0x00, 0x02, 0x01, 0x12, 0x03, 0x05, 0x08, 0x1e, 0x0a, 0x0f, 0x0a, 0x07, 0x04,
    0x00, 0x03, 0x00, 0x02, 0x01, 0x04, 0x12, 0x04, 0x05, 0x08, 0x04, 0x18, 0x0a, 0x0e, 0x0a, 0x07,
    0x04, 0x00, 0x03, 0x00, 0x02, 0x01, 0x06, 0x12, 0x03, 0x05, 0x08, 0x10, 0x0a, 0x0e, 0x0a, 0x07,
    0x04, 0x00, 0x03, 0x00, 0x02, 0x01, 0x01, 0x12, 0x03, 0x05, 0x11, 0x19, 0x0a, 0x0e, 0x0a, 0x07,
    0x04, 0x00, 0x03, 0x00, 0x02, 0x01, 0x03, 0x12, 0x03, 0x05, 0x1c, 0x1d, 0x0a, 0x0c, 0x0a, 0x04,
    0x04, 0x00, 0x03, 0x01, 0x12, 0x04, 0x08, 0x04, 0x0b, 0x05, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00,
    0x03, 0x01, 0x01, 0x12, 0x03, 0x08, 0x0c, 0x14, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x03, 0x01,
    0x02, 0x00, 0x12, 0x03, 0x09, 0x08, 0x18, 0x0a, 0x0f, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x01, 0x02,
    0x00, 0x04, 0x12, 0x04, 0x09, 0x08, 0x08, 0x16, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x01,
    0x02, 0x00, 0x05, 0x12, 0x03, 0x09, 0x08, 0x0e, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x01,
    0x02, 0x00, 0x01, 0x12, 0x03, 0x09, 0x0f, 0x13, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x01,
    0x02, 0x00, 0x03, 0x12, 0x03, 0x09, 0x16, 0x17, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x00, 0x03, 0x01,
    0x02, 0x01, 0x12, 0x03, 0x0a, 0x08, 0x28, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x01, 0x02,
    0x01, 0x04, 0x12, 0x03, 0x0a, 0x08, 0x10, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x01, 0x02,
    0x01, 0x06, 0x12, 0x03, 0x0a, 0x11, 0x19, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x01, 0x02,
    0x01, 0x01, 0x12, 0x03, 0x0a, 0x1a, 0x23, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x00, 0x03, 0x01, 0x02,
    0x01, 0x03, 0x12, 0x03, 0x0a, 0x26, 0x27, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x00, 0x12,
    0x03, 0x0d, 0x04, 0x1b, 0x0a, 0x0d, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x04, 0x12, 0x04, 0x0d,
    0x04, 0x0b, 0x05, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x05, 0x12, 0x03, 0x0d, 0x04,
    0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x0d, 0x0b, 0x16, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x0d, 0x19, 0x1a, 0x0a, 0x0b, 0x0a,
    0x04, 0x04, 0x00, 0x02, 0x01, 0x12, 0x03, 0x0e, 0x04, 0x26, 0x0a, 0x0d, 0x0a, 0x05, 0x04, 0x00,
    0x02, 0x01, 0x04, 0x12, 0x04, 0x0e, 0x04, 0x0d, 0x1b, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02,
    0x01, 0x05, 0x12, 0x03, 0x0e, 0x04, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x01,
    0x12, 0x03, 0x0e, 0x0b, 0x21, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x03, 0x12, 0x03,
    0x0e, 0x24, 0x25, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x02, 0x12, 0x03, 0x0f, 0x04, 0x26,
    0x0a, 0x0d, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x04, 0x12, 0x04, 0x0f, 0x04, 0x0e, 0x26, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x05, 0x12, 0x03, 0x0f, 0x04, 0x0a, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x00, 0x02, 0x02, 0x01, 0x12, 0x03, 0x0f, 0x0b, 0x21, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x00, 0x02, 0x02, 0x03, 0x12, 0x03, 0x0f, 0x24, 0x25, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02,
    0x03, 0x12, 0x03, 0x10, 0x04, 0x1c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x03, 0x04, 0x12,
    0x03, 0x10, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x03, 0x06, 0x12, 0x03, 0x10,
    0x0d, 0x11, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x03, 0x01, 0x12, 0x03, 0x10, 0x12, 0x17,
    0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x03, 0x03, 0x12, 0x03, 0x10, 0x1a, 0x1b, 0x0a, 0x0a,
    0x0a, 0x02, 0x04, 0x01, 0x12, 0x04, 0x13, 0x00, 0x1d, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x01,
    0x01, 0x12, 0x03, 0x13, 0x08, 0x0d, 0x0a, 0x0c, 0x0a, 0x04, 0x04, 0x01, 0x04, 0x00, 0x12, 0x04,
    0x14, 0x04, 0x17, 0x05, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x04, 0x00, 0x01, 0x12, 0x03, 0x14,
    0x09, 0x0d, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x01, 0x04, 0x00, 0x02, 0x00, 0x12, 0x03, 0x15, 0x08,
    0x1e, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x01, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x15, 0x08,
    0x19, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x01, 0x04, 0x00, 0x02, 0x00, 0x02, 0x12, 0x03, 0x15, 0x1c,
    0x1d, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x01, 0x04, 0x00, 0x02, 0x01, 0x12, 0x03, 0x16, 0x08, 0x1e,
    0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x01, 0x04, 0x00, 0x02, 0x01, 0x01, 0x12, 0x03, 0x16, 0x08, 0x19,
    0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x01, 0x04, 0x00, 0x02, 0x01, 0x02, 0x12, 0x03, 0x16, 0x1c, 0x1d,
    0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01, 0x02, 0x00, 0x12, 0x03, 0x19, 0x04, 0x12, 0x0a, 0x0d, 0x0a,
    0x05, 0x04, 0x01, 0x02, 0x00, 0x04, 0x12, 0x04, 0x19, 0x04, 0x17, 0x05, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x01, 0x02, 0x00, 0x06, 0x12, 0x03, 0x19, 0x04, 0x08, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01,
    0x02, 0x00, 0x01, 0x12, 0x03, 0x19, 0x09, 0x0d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00,
    0x03, 0x12, 0x03, 0x19, 0x10, 0x11, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01, 0x02, 0x01, 0x12, 0x03,
    0x1a, 0x04, 0x1f, 0x0a, 0x0d, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x01, 0x04, 0x12, 0x04, 0x1a, 0x04,
    0x19, 0x12, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x01, 0x05, 0x12, 0x03, 0x1a, 0x04, 0x0a,
    0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x01, 0x01, 0x12, 0x03, 0x1a, 0x0b, 0x1a, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x01, 0x02, 0x01, 0x03, 0x12, 0x03, 0x1a, 0x1d, 0x1e, 0x0a, 0x0b, 0x0a, 0x04,
    0x04, 0x01, 0x02, 0x02, 0x12, 0x03, 0x1b, 0x04, 0x24, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02,
    0x02, 0x04, 0x12, 0x03, 0x1b, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x02, 0x06,
    0x12, 0x03, 0x1b, 0x0d, 0x15, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x02, 0x01, 0x12, 0x03,
    0x1b, 0x16, 0x1f, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x02, 0x03, 0x12, 0x03, 0x1b, 0x22,
    0x23, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01, 0x02, 0x03, 0x12, 0x03, 0x1c, 0x04, 0x22, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x01, 0x02, 0x03, 0x04, 0x12, 0x03, 0x1c, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x01, 0x02, 0x03, 0x06, 0x12, 0x03, 0x1c, 0x0d, 0x14, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01,
    0x02, 0x03, 0x01, 0x12, 0x03, 0x1c, 0x15, 0x1d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x03,
    0x03, 0x12, 0x03, 0x1c, 0x20, 0x21, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x02, 0x12, 0x04, 0x1f, 0x00,
    0x35, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x02, 0x01, 0x12, 0x03, 0x1f, 0x08, 0x0f, 0x0a, 0x0c,
    0x0a, 0x04, 0x04, 0x02, 0x04, 0x00, 0x12, 0x04, 0x20, 0x04, 0x28, 0x05, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x02, 0x04, 0x00, 0x01, 0x12, 0x03, 0x20, 0x09, 0x0d, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x02,
    0x04, 0x00, 0x02, 0x00, 0x12, 0x03, 0x21, 0x08, 0x1f, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04,
    0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x21, 0x08, 0x1a, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04,
    0x00, 0x02, 0x00, 0x02, 0x12, 0x03, 0x21, 0x1d, 0x1e, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x02, 0x04,
    0x00, 0x02, 0x01, 0x12, 0x03, 0x22, 0x08, 0x1a, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00,
    0x02, 0x01, 0x01, 0x12, 0x03, 0x22, 0x08, 0x15, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00,
    0x02, 0x01, 0x02, 0x12, 0x03, 0x22, 0x18, 0x19, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x02, 0x04, 0x00,
    0x02, 0x02, 0x12, 0x03, 0x23, 0x08, 0x1a, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00, 0x02,
    0x02, 0x01, 0x12, 0x03, 0x23, 0x08, 0x15, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00, 0x02,
    0x02, 0x02, 0x12, 0x03, 0x23, 0x18, 0x19, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x02, 0x04, 0x00, 0x02,
    0x03, 0x12, 0x03, 0x24, 0x08, 0x19, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00, 0x02, 0x03,
    0x01, 0x12, 0x03, 0x24, 0x08, 0x14, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00, 0x02, 0x03,
    0x02, 0x12, 0x03, 0x24, 0x17, 0x18, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x02, 0x04, 0x00, 0x02, 0x04,
    0x12, 0x03, 0x25, 0x08, 0x1b, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00, 0x02, 0x04, 0x01,
    0x12, 0x03, 0x25, 0x08, 0x16, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00, 0x02, 0x04, 0x02,
    0x12, 0x03, 0x25, 0x19, 0x1a, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x02, 0x04, 0x00, 0x02, 0x05, 0x12,
    0x03, 0x26, 0x08, 0x14, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00, 0x02, 0x05, 0x01, 0x12,
    0x03, 0x26, 0x08, 0x0f, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00, 0x02, 0x05, 0x02, 0x12,
    0x03, 0x26, 0x12, 0x13, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x02, 0x04, 0x00, 0x02, 0x06, 0x12, 0x03,
    0x27, 0x08, 0x20, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00, 0x02, 0x06, 0x01, 0x12, 0x03,
    0x27, 0x08, 0x1b, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00, 0x02, 0x06, 0x02, 0x12, 0x03,
    0x27, 0x1e, 0x1f, 0x0a, 0x0c, 0x0a, 0x04, 0x04, 0x02, 0x04, 0x01, 0x12, 0x04, 0x2a, 0x04, 0x2d,
    0x05, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x04, 0x01, 0x01, 0x12, 0x03, 0x2a, 0x09, 0x0f, 0x0a,
    0x0d, 0x0a, 0x06, 0x04, 0x02, 0x04, 0x01, 0x02, 0x00, 0x12, 0x03, 0x2b, 0x08, 0x1a, 0x0a, 0x0e,
    0x0a, 0x07, 0x04, 0x02, 0x04, 0x01, 0x02, 0x00, 0x01, 0x12, 0x03, 0x2b, 0x08, 0x15, 0x0a, 0x0e,
    0x0a, 0x07, 0x04, 0x02, 0x04, 0x01, 0x02, 0x00, 0x02, 0x12, 0x03, 0x2b, 0x18, 0x19, 0x0a, 0x0d,
    0x0a, 0x06, 0x04, 0x02, 0x04, 0x01, 0x02, 0x01, 0x12, 0x03, 0x2c, 0x08, 0x1a, 0x0a, 0x0e, 0x0a,
    0x07, 0x04, 0x02, 0x04, 0x01, 0x02, 0x01, 0x01, 0x12, 0x03, 0x2c, 0x08, 0x15, 0x0a, 0x0e, 0x0a,
    0x07, 0x04, 0x02, 0x04, 0x01, 0x02, 0x01, 0x02, 0x12, 0x03, 0x2c, 0x18, 0x19, 0x0a, 0x0b, 0x0a,
    0x04, 0x04, 0x02, 0x02, 0x00, 0x12, 0x03, 0x2f, 0x04, 0x12, 0x0a, 0x0d, 0x0a, 0x05, 0x04, 0x02,
    0x02, 0x00, 0x04, 0x12, 0x04, 0x2f, 0x04, 0x2d, 0x05, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02,
    0x00, 0x06, 0x12, 0x03, 0x2f, 0x04, 0x08, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x00, 0x01,
    0x12, 0x03, 0x2f, 0x09, 0x0d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x00, 0x03, 0x12, 0x03,
    0x2f, 0x10, 0x11, 0x0a, 0x0c, 0x0a, 0x04, 0x04, 0x02, 0x08, 0x00, 0x12, 0x04, 0x30, 0x04, 0x33,
    0x05, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x08, 0x00, 0x01, 0x12, 0x03, 0x30, 0x0a, 0x10, 0x0a,
    0x0b, 0x0a, 0x04, 0x04, 0x02, 0x02, 0x01, 0x12, 0x03, 0x31, 0x08, 0x20, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x02, 0x02, 0x01, 0x06, 0x12, 0x03, 0x31, 0x08, 0x0e, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02,
    0x02, 0x01, 0x01, 0x12, 0x03, 0x31, 0x0f, 0x1b, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x01,
    0x03, 0x12, 0x03, 0x31, 0x1e, 0x1f, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x02, 0x02, 0x02, 0x12, 0x03,
    0x32, 0x08, 0x20, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x02, 0x05, 0x12, 0x03, 0x32, 0x08,
    0x0e, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x02, 0x01, 0x12, 0x03, 0x32, 0x0f, 0x1b, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x02, 0x03, 0x12, 0x03, 0x32, 0x1e, 0x1f, 0x0a, 0x0b, 0x0a,
    0x04, 0x04, 0x02, 0x02, 0x03, 0x12, 0x03, 0x34, 0x04, 0x24, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02,
    0x02, 0x03, 0x04, 0x12, 0x03, 0x34, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x03,
    0x06, 0x12, 0x03, 0x34, 0x0d, 0x15, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x03, 0x01, 0x12,
    0x03, 0x34, 0x16, 0x1f, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x03, 0x03, 0x12, 0x03, 0x34,
    0x22, 0x23, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x03, 0x12, 0x04, 0x37, 0x00, 0x40, 0x01, 0x0a, 0x0a,
    0x0a, 0x03, 0x04, 0x03, 0x01, 0x12, 0x03, 0x37, 0x08, 0x10, 0x0a, 0x0c, 0x0a, 0x04, 0x04, 0x03,
    0x08, 0x00, 0x12, 0x04, 0x38, 0x04, 0x3f, 0x05, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x08, 0x00,
    0x01, 0x12, 0x03, 0x38, 0x0a, 0x0f, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x03, 0x02, 0x00, 0x12, 0x03,
    0x39, 0x08, 0x1d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x00, 0x05, 0x12, 0x03, 0x39, 0x08,
    0x0e, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x00, 0x01, 0x12, 0x03, 0x39, 0x0f, 0x18, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x00, 0x03, 0x12, 0x03, 0x39, 0x1b, 0x1c, 0x0a, 0x0b, 0x0a,
    0x04, 0x04, 0x03, 0x02, 0x01, 0x12, 0x03, 0x3a, 0x08, 0x17, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03,
    0x02, 0x01, 0x05, 0x12, 0x03, 0x3a, 0x08, 0x0e, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x01,
    0x01, 0x12, 0x03, 0x3a, 0x0f, 0x12, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x01, 0x03, 0x12,
    0x03, 0x3a, 0x15, 0x16, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x03, 0x02, 0x02, 0x12, 0x03, 0x3b, 0x08,
    0x1a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x02, 0x05, 0x12, 0x03, 0x3b, 0x08, 0x0e, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x02, 0x01, 0x12, 0x03, 0x3b, 0x0f, 0x15, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x03, 0x02, 0x02, 0x03, 0x12, 0x03, 0x3b, 0x18, 0x19, 0x0a, 0x0b, 0x0a, 0x04, 0x04,
    0x03, 0x02, 0x03, 0x12, 0x03, 0x3c, 0x08, 0x1a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x03,
    0x06, 0x12, 0x03, 0x3c, 0x08, 0x0e, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x03, 0x01, 0x12,
    0x03, 0x3c, 0x0f, 0x15, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x03, 0x03, 0x12, 0x03, 0x3c,
    0x18, 0x19, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x03, 0x02, 0x04, 0x12, 0x03, 0x3d, 0x08, 0x19, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x04, 0x05, 0x12, 0x03, 0x3d, 0x08, 0x0e, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x03, 0x02, 0x04, 0x01, 0x12, 0x03, 0x3d, 0x0f, 0x14, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x03, 0x02, 0x04, 0x03, 0x12, 0x03, 0x3d, 0x17, 0x18, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x03, 0x02,
    0x05, 0x12, 0x03, 0x3e, 0x08, 0x19, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x05, 0x05, 0x12,
    0x03, 0x3e, 0x08, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x05, 0x01, 0x12, 0x03, 0x3e,
    0x0d, 0x14, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02, 0x05, 0x03, 0x12, 0x03, 0x3e, 0x17, 0x18,
    0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x04, 0x12, 0x04, 0x42, 0x00, 0x46, 0x01, 0x0a, 0x0a, 0x0a, 0x03,
    0x04, 0x04, 0x01, 0x12, 0x03, 0x42, 0x08, 0x0e, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x04, 0x02, 0x00,
    0x12, 0x03, 0x43, 0x04, 0x14, 0x0a, 0x0d, 0x0a, 0x05, 0x04, 0x04, 0x02, 0x00, 0x04, 0x12, 0x04,
    0x43, 0x04, 0x42, 0x10, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x04, 0x02, 0x00, 0x05, 0x12, 0x03, 0x43,
    0x04, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x04, 0x02, 0x00, 0x01, 0x12, 0x03, 0x43, 0x0b, 0x0f,
    0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x04, 0x02, 0x00, 0x03, 0x12, 0x03, 0x43, 0x12, 0x13, 0x0a, 0x0b,
    0x0a, 0x04, 0x04, 0x04, 0x02, 0x01, 0x12, 0x03, 0x44, 0x04, 0x14, 0x0a, 0x0d, 0x0a, 0x05, 0x04,
    0x04, 0x02, 0x01, 0x04, 0x12, 0x04, 0x44, 0x04, 0x43, 0x14, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x04,
    0x02, 0x01, 0x05, 0x12, 0x03, 0x44, 0x04, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x04, 0x02, 0x01,
    0x01, 0x12, 0x03, 0x44, 0x0b, 0x0f, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x04, 0x02, 0x01, 0x03, 0x12,
    0x03, 0x44, 0x12, 0x13, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x04, 0x02, 0x02, 0x12, 0x03, 0x45, 0x04,
    0x2d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x04, 0x02, 0x02, 0x04, 0x12, 0x03, 0x45, 0x04, 0x0c, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x04, 0x02, 0x02, 0x05, 0x12, 0x03, 0x45, 0x0d, 0x13, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x04, 0x02, 0x02, 0x01, 0x12, 0x03, 0x45, 0x14, 0x1a, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x04, 0x02, 0x02, 0x03, 0x12, 0x03, 0x45, 0x1d, 0x1e, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x04, 0x02,
    0x02, 0x08, 0x12, 0x03, 0x45, 0x1f, 0x2c, 0x0a, 0x0f, 0x0a, 0x08, 0x04, 0x04, 0x02, 0x02, 0x08,
    0xe7, 0x07, 0x00, 0x12, 0x03, 0x45, 0x20, 0x2b, 0x0a, 0x10, 0x0a, 0x09, 0x04, 0x04, 0x02, 0x02,
    0x08, 0xe7, 0x07, 0x00, 0x02, 0x12, 0x03, 0x45, 0x20, 0x26, 0x0a, 0x11, 0x0a, 0x0a, 0x04, 0x04,
    0x02, 0x02, 0x08, 0xe7, 0x07, 0x00, 0x02, 0x00, 0x12, 0x03, 0x45, 0x20, 0x26, 0x0a, 0x12, 0x0a,
    0x0b, 0x04, 0x04, 0x02, 0x02, 0x08, 0xe7, 0x07, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x45, 0x20,
    0x26, 0x0a, 0x10, 0x0a, 0x09, 0x04, 0x04, 0x02, 0x02, 0x08, 0xe7, 0x07, 0x00, 0x03, 0x12, 0x03,
    0x45, 0x27, 0x2b, 0x62, 0x06, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x33,
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
