//! Settings UI command argument: typed for the generated TypeScript bindings, but received as raw
//! JSON so that the validator shared with the CLI, not Tauri's argument deserialization, decides
//! whether a value is accepted (same code, message and reason on both surfaces).
use serde::Deserialize;
use specta::{datatype::reference::Reference, DataType, Generics, Type, TypeCollection};
use std::marker::PhantomData;

/// Raw JSON value of a settings command argument, declared to TypeScript as `T`.
// `fn() -> T` keeps the marker `Send` and `'static` whatever `T` is: no `T` is ever stored.
#[derive(Deserialize)]
#[serde(transparent, bound = "")]
pub struct UiValue<T> {
    value: serde_json::Value,
    #[serde(skip)]
    declared: PhantomData<fn() -> T>,
}
impl<T> UiValue<T> {
    /// Wrap a value as the webview would send it.
    pub fn new(value: serde_json::Value) -> Self {
        Self {
            value,
            declared: PhantomData,
        }
    }
    /// The raw value, validated later by the shared validator.
    pub fn into_json(self) -> serde_json::Value {
        self.value
    }
}
/// The bindings show the declared type `T`, so the frontend keeps its compile-time checks.
impl<T: Type> Type for UiValue<T> {
    fn inline(types: &mut TypeCollection, generics: Generics) -> DataType {
        T::inline(types, generics)
    }
    fn reference(types: &mut TypeCollection, generics: &[DataType]) -> Reference {
        T::reference(types, generics)
    }
}
#[cfg(test)]
mod tests {
    use super::UiValue;
    use serde_json::json;
    use specta::{Generics, Type, TypeCollection};

    #[test]
    fn any_json_reaches_the_shared_validator() -> Result<(), serde_json::Error> {
        for value in [
            json!(-1),
            json!("verbose"),
            json!(1.5),
            json!(null),
            json!([1]),
        ] {
            let received = serde_json::from_value::<UiValue<u64>>(value.clone())?;
            assert_eq!(received.into_json(), value);
        }
        Ok(())
    }
    #[test]
    fn bindings_keep_the_declared_type() {
        let declared = |inline: fn(&mut TypeCollection, Generics) -> specta::DataType| {
            format!(
                "{:?}",
                inline(&mut TypeCollection::default(), Generics::Definition)
            )
        };
        assert_eq!(
            declared(UiValue::<crate::settings::LogLevel>::inline),
            declared(crate::settings::LogLevel::inline)
        );
        assert_eq!(
            declared(UiValue::<Option<u16>>::inline),
            declared(Option::<u16>::inline)
        );
    }
}
