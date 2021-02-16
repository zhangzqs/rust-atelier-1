use crate::syntax::*;
use crate::FILE_EXTENSION;
use atelier_core::error::{ErrorKind, Result as ModelResult, ResultExt};
use atelier_core::io::ModelWriter;
use atelier_core::model::shapes::{AppliedTrait, HasTraits, MemberShape, ShapeKind, TopLevelShape};
use atelier_core::model::values::{Number, Value as NodeValue};
use atelier_core::model::{HasIdentity, Model, ShapeID};
use serde_json::{to_writer, to_writer_pretty, Map, Number as JsonNumber, Value};
use std::io::Write;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

///
/// Write a [Model](../atelier_core/model/struct.Model.html) in the JSON AST representation.
///
#[allow(missing_debug_implementations)]
pub struct JsonWriter {
    pretty_print: bool,
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl Default for JsonWriter {
    fn default() -> Self {
        Self {
            pretty_print: false,
        }
    }
}

impl ModelWriter for JsonWriter {
    fn write(&mut self, w: &mut impl Write, model: &Model) -> ModelResult<()> {
        let mut top: Map<String, Value> = Default::default();

        let _ = top.insert(
            K_SMITHY.to_string(),
            Value::String(model.smithy_version().to_string()),
        );

        let _ = top.insert(K_SHAPES.to_string(), self.shapes(model));

        if self.pretty_print {
            to_writer_pretty(w, &Value::Object(top))
                .chain_err(|| ErrorKind::Serialization(FILE_EXTENSION.to_string()).to_string())
        } else {
            to_writer(w, &Value::Object(top))
                .chain_err(|| ErrorKind::Serialization(FILE_EXTENSION.to_string()).to_string())
        }
    }
}

impl<'a> JsonWriter {
    pub fn new(pretty_print: bool) -> Self {
        Self { pretty_print }
    }

    fn shapes(&self, model: &Model) -> Value {
        let mut shape_map: Map<String, Value> = Default::default();
        for shape in model.shapes() {
            let _ = shape_map.insert(shape.id().to_string(), self.shape(shape));
        }
        if model.has_metadata() {
            let mut meta_map: Map<String, Value> = Default::default();
            for (key, value) in model.metadata() {
                let _ = meta_map.insert(key.to_string(), self.value(value));
            }
            let _ = shape_map.insert(K_METADATA.to_string(), Value::Object(meta_map));
        }
        Value::Object(shape_map)
    }

    fn shape(&self, shape: &TopLevelShape) -> Value {
        let mut shape_map: Map<String, Value> = Default::default();
        if shape.has_traits() {
            let _ = shape_map.insert(K_TRAITS.to_string(), self.traits(shape.traits()));
        }
        match shape.body() {
            ShapeKind::Simple(v) => {
                let _ = shape_map.insert(K_TYPE.to_string(), Value::String(v.to_string()));
            }
            ShapeKind::List(v) => {
                let _ = shape_map.insert(K_TYPE.to_string(), Value::String(V_LIST.to_string()));
                let _ = shape_map.insert(K_MEMBER.to_string(), self.reference(v.member().target()));
            }
            ShapeKind::Set(v) => {
                let _ = shape_map.insert(K_TYPE.to_string(), Value::String(V_SET.to_string()));
                let _ = shape_map.insert(K_MEMBER.to_string(), self.reference(v.member().target()));
            }
            ShapeKind::Map(v) => {
                let _ = shape_map.insert(K_TYPE.to_string(), Value::String(V_MAP.to_string()));
                let _ = shape_map.insert(K_KEY.to_string(), self.reference(v.key().target()));
                let _ = shape_map.insert(K_VALUE.to_string(), self.reference(v.value().target()));
            }
            ShapeKind::Structure(v) => {
                let _ =
                    shape_map.insert(K_TYPE.to_string(), Value::String(V_STRUCTURE.to_string()));
                if v.has_members() {
                    let _ = shape_map.insert(K_MEMBERS.to_string(), self.members(v.members()));
                }
            }
            ShapeKind::Union(v) => {
                let _ = shape_map.insert(K_TYPE.to_string(), Value::String(V_UNION.to_string()));
                if v.has_members() {
                    let _ = shape_map.insert(K_MEMBERS.to_string(), self.members(v.members()));
                }
            }
            ShapeKind::Service(v) => {
                let _ = shape_map.insert(K_TYPE.to_string(), Value::String(V_SERVICE.to_string()));
                let _ = shape_map.insert(
                    K_VERSION.to_string(),
                    Value::String(v.version().to_string()),
                );
                if v.has_operations() {
                    let _ = shape_map.insert(
                        K_OPERATIONS.to_string(),
                        Value::Array(v.operations().map(|o| self.reference(o)).collect()),
                    );
                }
                if v.has_resources() {
                    let _ = shape_map.insert(
                        K_RESOURCES.to_string(),
                        Value::Array(v.resources().map(|o| self.reference(o)).collect()),
                    );
                }
            }
            ShapeKind::Operation(v) => {
                let _ =
                    shape_map.insert(K_TYPE.to_string(), Value::String(V_OPERATION.to_string()));
                if let Some(v) = v.input() {
                    let _ = shape_map.insert(K_INPUT.to_string(), self.reference(v));
                }
                if let Some(v) = v.output() {
                    let _ = shape_map.insert(K_OUTPUT.to_string(), self.reference(v));
                }
                if v.has_errors() {
                    let _ = shape_map.insert(
                        K_ERRORS.to_string(),
                        Value::Array(v.errors().map(|o| self.reference(o)).collect()),
                    );
                }
            }
            ShapeKind::Resource(v) => {
                let _ = shape_map.insert(K_TYPE.to_string(), Value::String(V_RESOURCE.to_string()));
                if v.has_identifiers() {
                    let mut id_map: Map<String, Value> = Default::default();
                    for (id, ref_id) in v.identifiers() {
                        let _ = id_map.insert(id.to_string(), Value::String(ref_id.to_string()));
                    }
                    let _ = shape_map.insert(K_IDENTIFIERS.to_string(), Value::Object(id_map));
                }
                if let Some(v) = v.create() {
                    let _ = shape_map.insert(K_CREATE.to_string(), self.reference(v));
                }
                if let Some(v) = v.put() {
                    let _ = shape_map.insert(K_PUT.to_string(), self.reference(v));
                }
                if let Some(v) = v.read() {
                    let _ = shape_map.insert(K_READ.to_string(), self.reference(v));
                }
                if let Some(v) = v.update() {
                    let _ = shape_map.insert(K_UPDATE.to_string(), self.reference(v));
                }
                if let Some(v) = v.delete() {
                    let _ = shape_map.insert(K_DELETE.to_string(), self.reference(v));
                }
                if let Some(v) = v.list() {
                    let _ = shape_map.insert(K_LIST.to_string(), self.reference(v));
                }
                if v.has_operations() {
                    let _ = shape_map.insert(
                        K_OPERATIONS.to_string(),
                        Value::Array(v.operations().map(|o| self.reference(o)).collect()),
                    );
                }
                if v.has_collection_operations() {
                    let _ = shape_map.insert(
                        K_OPERATIONS.to_string(),
                        Value::Array(
                            v.collection_operations()
                                .map(|o| self.reference(o))
                                .collect(),
                        ),
                    );
                }
                if v.has_resources() {
                    let _ = shape_map.insert(
                        K_COLLECTION_OPERATIONS.to_string(),
                        Value::Array(v.resources().map(|o| self.reference(o)).collect()),
                    );
                }
            }
            ShapeKind::Unresolved => {
                let _ = shape_map.insert(K_TYPE.to_string(), Value::String(V_APPLY.to_string()));
            }
        }
        Value::Object(shape_map)
    }

    fn traits(&self, traits: &[AppliedTrait]) -> Value {
        let mut trait_map: Map<String, Value> = Default::default();
        for a_trait in traits {
            let _ = trait_map.insert(
                a_trait.id().to_string(),
                match a_trait.value() {
                    None => Value::Object(Default::default()),
                    Some(value) => self.value(value),
                },
            );
        }
        Value::Object(trait_map)
    }

    fn members(&self, members: impl Iterator<Item = &'a MemberShape>) -> Value {
        let mut members_map: Map<String, Value> = Default::default();
        for member in members {
            let mut member_map: Map<String, Value> = Default::default();
            if member.has_traits() {
                let _ = member_map.insert(K_TRAITS.to_string(), self.traits(member.traits()));
            }
            let _ = member_map.insert(
                K_TARGET.to_string(),
                Value::String(member.target().to_string()),
            );
            let _ = members_map.insert(member.id().to_string(), Value::Object(member_map));
        }
        Value::Object(members_map)
    }

    fn value(&self, value: &NodeValue) -> Value {
        match value {
            NodeValue::None => Value::Null,
            NodeValue::Array(v) => Value::Array(v.iter().map(|v| self.value(v)).collect()),
            NodeValue::Object(v) => {
                let mut object: Map<String, Value> = Default::default();
                for (k, v) in v {
                    let _ = object.insert(k.clone(), self.value(v));
                }
                Value::Object(object)
            }
            NodeValue::Number(v) => match v {
                Number::Integer(v) => Value::Number((*v).into()),
                Number::Float(v) => Value::Number(JsonNumber::from_f64(*v).unwrap()),
            },
            NodeValue::Boolean(v) => Value::Bool(*v),
            NodeValue::String(v) => Value::String(v.clone()),
        }
    }

    fn reference(&self, id: &'a ShapeID) -> Value {
        let mut shape_map: Map<String, Value> = Default::default();
        let _ = shape_map.insert(K_TARGET.to_string(), Value::String(id.to_string()));
        Value::Object(shape_map)
    }
}
