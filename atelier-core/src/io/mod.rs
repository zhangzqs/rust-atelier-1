/*!
Traits for reading and writing models in different formats. Separate crates implement the ability
to handle different representations, such as the original Smithy, JSON AST, and OpenAPI.

This module also provides some useful `Writer` implementations, all are features, all are included
by default.

* **debug**; uses the `Debug` implementation of Model to write out the internal structure.
* **uml**; uses [PlantUML](https://plantuml.com/) to generate diagrams of a model structure.

# Example Model Writer

The example below is pretty much the implementation of the `debug` module, it writes the model
using the `Debug` implementation associated with those objects.

```rust
# use atelier_core::io::ModelWriter;
# use atelier_core::model::Model;
# use atelier_core::error::Result;
# use std::io::Write;
#[derive(Debug)]
pub struct Debugger {}

impl Default for Debugger {
    fn default() -> Self {
        Self {}
    }
}

impl<'a> ModelWriter<'a> for Debugger {
    const REPRESENTATION: &'static str = "Debug";

    fn write(&mut self, w: &mut impl Write, model: &'a Model) -> Result<()> {
        write!(w, "{:#?}", model)?;
        Ok(())
    }
}
```

*/

use crate::error::Result;
use crate::model::Model;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

///
/// Trait implemented to write a model in a specific representation.
///
pub trait ModelWriter<'a>: Default {
    /// The display name of the representation this trait writes.
    const REPRESENTATION: &'static str;
    ///
    /// Write the `model` to given the implementation of `Write`.
    ///
    fn write(&mut self, w: &mut impl std::io::Write, model: &'a Model) -> Result<()>;
}

///
/// Trait implemented to read a model from a specific representation.
///
pub trait ModelReader: Default {
    /// The display name of the representation this trait reads.
    const REPRESENTATION: &'static str;
    ///
    ///  Read a model from the given implementation of `Read`.
    ///
    fn read(&mut self, r: &mut impl std::io::Read) -> Result<Model>;
}

// ------------------------------------------------------------------------------------------------
// Public Functions
// ------------------------------------------------------------------------------------------------

///
/// Read a model from the string-like value `s` using the given `ModelReader`. This is simply a
/// short-cut that saves some repetitive boiler-plate.
///
pub fn read_model_from_string<S>(r: &mut impl ModelReader, s: S) -> Result<Model>
where
    S: AsRef<[u8]>,
{
    use std::io::Cursor;
    let mut buffer = Cursor::new(s);
    r.read(&mut buffer)
}

///
/// Write the `model` into a string `s` using the given `ModelWriter`. This is simply a
/// short-cut that saves some repetitive boiler-plate.
///
pub fn write_model_to_string<'a>(w: &mut impl ModelWriter<'a>, model: &'a Model) -> Result<String> {
    use std::io::Cursor;
    let mut buffer = Cursor::new(Vec::new());
    w.write(&mut buffer, model)?;
    Ok(String::from_utf8(buffer.into_inner()).unwrap())
}

// ------------------------------------------------------------------------------------------------
// Modules
// ------------------------------------------------------------------------------------------------

#[cfg(feature = "debug")]
pub mod debug;

#[cfg(feature = "uml")]
pub mod plant_uml;

#[cfg(feature = "tree")]
pub mod tree;
