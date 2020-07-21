/*!
This crate provides a Rust native core model for the AWS [Smithy](https://github.com/awslabs/smithy) Interface Definition Language.

This crate is the foundation for the Atelier set of crates, and provides the following components:

1. The [model](model/index.html) elements themselves that represents a Smithy model. This API is the
   in-memory representation shared by all Atelier crates and tools.
1. The model [builder](model/builder/index.html) API that allow for a more _fluent_ and less repetative construction of a
   core model.
1. The [prelude](prelude/index.html) model containing the set of shapes defined in the Smithy specification.
1. Model [actions](action/index.html) that are used to implement linters, validators, and transformations.
1. Traits for [reading/writing](io/index.html) models in different representations.
1. Trait and simple implementation for a model [registry](registry/index.html).
1. A common [error](error/index.html) module to be used by all Atelier crates.

## Data Model

The following is a diagrammatic representation of the core model. For the most part this is a
direct transformation from the ABNF in the specification, although some of the distinctions between
different ID types (`Identifier`, `ShapeID`) are not illustrated. It also shows all the
shape types as subclasses of `Shape`.

```text
┌───────────────┐
│ «enumeration» │
│   NodeValue   │
├───────────────┤                 ┌─────────┐
│Array          │                 ○         ○ prelude
│Object         │                ╱│╲        ┼
│Number         │               ┌─────────────┐
│Boolean        │metadata       │    Model    │
│ShapeID        │┼○─────────────├─────────────┤
│TextBlock      │               │namespace    │
│String         │control_data   │             │┼──────┐       ┌─────────────┐
│None           │┼○─────────────│             │       │       │   ShapeID   │
└───────────────┘               └─────────────┘       │       ├─────────────┤
  ┼     ┼     ┌───────────────┐        ┼              │      ╱│namespace?   │
  │     │     │     Trait     │        │              └─────○─│shape_name   │
  │     └─────├───────────────┤        │           references╲│member_name? │
  │           │id             │        │                      │             │
  │           └───────────────┘        │                      └─────────────┘
  │             ╲│╱       ╲│╱          │                             ┼ id
  │              ○         ○           │                             │
  │     ┌────────┘         └───────┐   ○                             │
  │     ┼                          ┼  ╱│╲ shapes                     │
┌───────────────┐               ┌─────────────┐                      │
│    Member     │╲member┌──────┼│    Shape    │┼─────────────────────┘
├───────────────┤─○─────┘       └─────────────┘   ┌─────────────────────────┐
│id             │╱                     △          │         Service         │
└───────────────┘                      │          ├─────────────────────────┤
┌───────────────┐                      │          │version                  │
│ «enumeration» │──────────────────────┼──────────│operations: [Operation]? │
│    Simple     │ ┌────────────┐       │          │resources: [Resource]?   │
├───────────────┤ │    List    │       │          └─────────────────────────┘
│Blob           │ ├────────────┤       │          ┌─────────────────────────┐
│Boolean        │ │member      │───────┤          │        Operation        │
│Document       │ └────────────┘       │          ├─────────────────────────┤
│String         │ ┌────────────┐       │          │input: Structure?        │
│Byte           │ │    Set     │       ├──────────│output: Structure?       │
│Short          │ ├────────────┤       │          │errors: [Structure]?     │
│Integer        │ │member      │───────┤          └─────────────────────────┘
│Long           │ └────────────┘       │          ┌─────────────────────────┐
│Float          │ ┌────────────┐       │          │        Resource         │
│Double         │ │    Map     │       │          ├─────────────────────────┤
│BigInteger     │ ├────────────┤       │          │identifiers?             │
│BigDecimal     │ │key         │       │          │create: Operation?       │
│Timestamp      │ │value       │───────┤          │put: Operation?          │
└───────────────┘ └────────────┘       │          │read: Operation?         │
                  ┌────────────┐       ├──────────│update: Operation?       │
                  │ Structure  │───────┤          │delete: Operation?       │
                  └────────────┘       │          │list: : Operation?       │
                  ┌────────────┐       │          │operations: [Operation]? │
                  │   Union    │───────┤          │collection_operations:   │
                  └────────────┘       │          │    [Operation]?         │
                  ┌────────────┐       │          │resources: [Resource]?   │
                  │   Apply    │───────┘          └─────────────────────────┘
                  └────────────┘
```

# The Semantic Model API Example

The following example demonstrates the core model API to create a model for a simple service. The
service, `MessageOfTheDay` has a single resource `Message`. The resource has an identifier for the
date, but the `read` operation does not make the date member required and so will return the message
for the current date.

This API acts as a set of generic data objects and as such has a tendency to be verbose in the
construction of models. The need to create a lot of `Identifier` and `ShapeID` instances, for example,
does impact the readability.

```rust
use atelier_core::model::shapes::{
    AppliedTrait, MemberShape, Operation, Resource, Service, Shape, ShapeKind, Simple,
    StructureOrUnion, TopLevelShape,
};
use atelier_core::model::values::Value;
use atelier_core::model::{Model, NamespaceID};
use atelier_core::prelude::PRELUDE_NAMESPACE;
use atelier_core::Version;

let prelude: NamespaceID = PRELUDE_NAMESPACE.parse().unwrap();
let namespace: NamespaceID = "example.motd".parse().unwrap();

// ----------------------------------------------------------------------------------------
let mut date = TopLevelShape::new(
    namespace.make_shape("Date".parse().unwrap()),
    ShapeKind::Simple(Simple::String),
);
let mut pattern_trait = AppliedTrait::new(prelude.make_shape("pattern".parse().unwrap()));
pattern_trait.set_value(Value::String(r"^\d\d\d\d\-\d\d-\d\d$".to_string()));
date.apply_trait(pattern_trait);

// ----------------------------------------------------------------------------------------
let shape_name = namespace.make_shape("BadDateValue".parse().unwrap());
let mut body = StructureOrUnion::new();
body.add_member(
    shape_name.make_member("errorMessage".parse().unwrap()),
    prelude.make_shape("String".parse().unwrap()),
);
let mut error = TopLevelShape::new(shape_name, ShapeKind::Structure(body));
let error_trait = AppliedTrait::with_value(
    prelude.make_shape("error".parse().unwrap()),
    "client".to_string().into(),
);
error.apply_trait(error_trait);

// ----------------------------------------------------------------------------------------
let shape_name = namespace.make_shape("GetMessageOutput".parse().unwrap());
let mut output = StructureOrUnion::new();
let mut message = MemberShape::new(
    shape_name.make_member("message".parse().unwrap()),
    prelude.make_shape("String".parse().unwrap()),
);
let required = AppliedTrait::new(prelude.make_shape("required".parse().unwrap()));
message.apply_trait(required);
let _ = output.add_a_member(message);
let output = TopLevelShape::new(
    namespace.make_shape("GetMessageOutput".parse().unwrap()),
    ShapeKind::Structure(output),
);

// ----------------------------------------------------------------------------------------
let shape_name = namespace.make_shape("GetMessageInput".parse().unwrap());
let mut input = StructureOrUnion::new();
input.add_member(
    shape_name.make_member("date".parse().unwrap()),
    date.id().clone(),
);
let input = TopLevelShape::new(
    namespace.make_shape("GetMessageInput".parse().unwrap()),
    ShapeKind::Structure(input),
);

// ----------------------------------------------------------------------------------------
let mut get_message = Operation::default();
get_message.set_input_shape(&input);
get_message.set_output_shape(&output);
get_message.add_error_shape(&error);
let mut get_message = TopLevelShape::new(
    namespace.make_shape("GetMessage".parse().unwrap()),
    ShapeKind::Operation(get_message),
);
let required = AppliedTrait::new(prelude.make_shape("readonly".parse().unwrap()));
get_message.apply_trait(required);

// ----------------------------------------------------------------------------------------
let mut message = Resource::default();
message.add_identifier("date".to_string(), Value::String(date.id().to_string()));
message.set_read_operation_shape(&get_message);
let message = TopLevelShape::new(
    namespace.make_shape("Message".parse().unwrap()),
    ShapeKind::Resource(message),
);

// ----------------------------------------------------------------------------------------
let mut service = Service::new("2020-06-21");
service.add_resource_shape(&message);
let mut service = TopLevelShape::new(
    namespace.make_shape("MessageOfTheDay".parse().unwrap()),
    ShapeKind::Service(service),
);
let documentation = AppliedTrait::with_value(
    prelude.make_shape("documentation".parse().unwrap()),
    Value::String("Provides a Message of the day.".to_string()),
);
service.apply_trait(documentation);

// ----------------------------------------------------------------------------------------
let mut model = Model::new(Version::V10);
model.add_shape(message);
model.add_shape(date);
model.add_shape(get_message);
model.add_shape(input);
model.add_shape(output);
model.add_shape(error);

println!("{:#?}", model);
```

# The Model Builder API Example

The following example demonstrates the builder interface to create the same service as the example
above. Hopefully this is more readable as it tends to be less repetative, uses  `&str` for
identifiers, and includes helper functions for common traits for example. It provides this better
_construction experience_ (there are no read methods on builder objects) by compromising two aspects:

1. The API itself is very repetative; this means the same method may be on multiple objects, but
makes it easier to use. For example, you want to add the documentation trait to a shape, so you can:
   1. construct a `Trait` entity using the core model and the `Builder::add_trait` method,
   1. use the `TraitBuilder::documentation` method which also takes the string to use as the trait
      value and returns a new `TraitBuilder`, or
   1. use the `Builder::documentation` method that hides all the details of a trait and just takes
      a string.
1. It hides a lot of the `Identifier` and `ShapeID` construction and so any of those calls to
   `from_str` may fail when the code unwraps the result. This means the builder can panic in ways
   the core model does not.

```rust
use atelier_core::error::ErrorSource;
use atelier_core::builder::values::{ArrayBuilder, ObjectBuilder};
use atelier_core::builder::{
    ListBuilder, MemberBuilder, ModelBuilder, OperationBuilder, ResourceBuilder,
    ServiceBuilder, SimpleShapeBuilder, StructureBuilder, TraitBuilder,
};
use atelier_core::model::{Identifier, Model, ShapeID};
use atelier_core::Version;

let model: Model = ModelBuilder::new(Version::V10, "example.motd")
    .service(
        ServiceBuilder::new("MessageOfTheDay", "2020-06-21")
            .documentation("Provides a Message of the day.")
            .resource("Message")
    )
    .resource(
        ResourceBuilder::new("Message")
            .identifier("date", "Date")
            .read("GetMessage")
    )
    .simple_shape(
        SimpleShapeBuilder::string("Date")
            .apply_trait(TraitBuilder::pattern(r"^\d\d\d\d\-\d\d-\d\d$").into())
    )
    .operation(
        OperationBuilder::new("GetMessage")
            .readonly()
            .input("GetMessageInput")
            .output("GetMessageOutput")
            .error_source("BadDateValue")
    )
    .structure(
        StructureBuilder::new("GetMessageInput")
            .member("date", "Date")
    )
    .structure(
        StructureBuilder::new("GetMessageOutput")
            .add_member(MemberBuilder::string("message").required().into())
    )
    .structure(
        StructureBuilder::new("BadDateValue")
            .error(ErrorSource::Client)
            .add_member(MemberBuilder::string("errorMessage").required().into())
    )
    .into();
```
*/

#![warn(
    // ---------- Stylistic
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    // ---------- Public
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    // ---------- Unsafe
    unsafe_code,
    // ---------- Unused
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
)]

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

use std::fmt::{Display, Formatter};
use std::str::FromStr;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

///
/// Versions of the Smithy specification.
///
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub enum Version {
    /// Version 1.0 (initial, and current)
    V10,
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl Default for Version {
    fn default() -> Self {
        Self::current()
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "1.0")
    }
}

impl FromStr for Version {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "1.0" {
            Ok(Self::V10)
        } else {
            Err(error::ErrorKind::InvalidVersionNumber(s.to_string()).into())
        }
    }
}

impl Version {
    ///
    /// Returns the most current version of the Smithy specification.
    ///
    pub fn current() -> Self {
        Self::V10
    }
}

// ------------------------------------------------------------------------------------------------
// Modules
// ------------------------------------------------------------------------------------------------

#[doc(hidden)]
#[macro_use]
mod macros;

pub mod action;

pub mod builder;

pub mod error;

pub mod io;

pub mod model;

pub mod prelude;

pub mod syntax;
