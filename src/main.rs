mod vdf_value;

use nu_plugin::{serve_plugin, EvaluatedCall, MsgPackSerializer, Plugin, SimplePluginCommand};
use nu_protocol::{Category, Example, LabeledError, PluginSignature, Signature, Value};

struct FromVdf;

impl SimplePluginCommand for FromVdf {
    type Plugin = VdfPlugin;

    fn name(&self) -> &str {
        "from vdf"
    }

    fn description(&self) -> &str {
        "Parse a VDF text into a structured value."
    }

    fn signature(&self) -> Signature {
        let mut signature = PluginSignature::build(self.name());
        signature.sig = signature.sig
            .switch("lossy", "Allow lossy parsing", Some('l'))
            .input_output_types(vec![(nu_protocol::Type::String, nu_protocol::Type::Record(vec![].into()))])
            .category(Category::Formats);
        signature.sig
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: r#""Key" "Value" | from vdf"#,
            description: "Parse a simple VDF string",
            result: Some(Value::test_record(
                nu_protocol::record! { "Key" => Value::test_string("Value") },
            )),
        },
        Example {
            example: r#""RootKey"\n{\n    "SubKey1" "Value1"\n    "SubKey2"\n    {\n        "NestedKey" "NestedValue"\n    }\n    "SubKey3" "Value3"\n}" | from vdf"#,
            description: "Parse a nested VDF string",
            result: Some(Value::test_record(
                nu_protocol::record! {
                    "RootKey" => Value::test_record(
                        nu_protocol::record! {
                            "SubKey1" => Value::test_string("Value1"),
                            "SubKey2" => Value::test_record(
                                nu_protocol::record! {
                                    "NestedKey" => Value::test_string("NestedValue"),
                                }
                            ),
                            "SubKey3" => Value::test_string("Value3"),
                        }
                    ),
                }
            )),
        }
        ]
    }

    fn run(
        &self,
        _plugin: &VdfPlugin,
        _engine: &nu_plugin::EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let span = call.head;
        let lossy = call.has_flag("lossy")?;
        let input_string = input.as_str()?;
        match vdf_value::parse(&input_string, lossy) {
            Ok(vdf) => Ok(vdf.into_value(span)),
            Err(e) => Err(LabeledError::new(e).with_label("Error parsing VDF", span)),
        }
    }
}

struct VdfPlugin;

impl Plugin for VdfPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn commands(&self) -> Vec<Box<dyn nu_plugin::PluginCommand<Plugin = Self>>> {
        vec![Box::new(FromVdf)]
    }
}

fn main() {
    serve_plugin(&VdfPlugin, MsgPackSerializer);
}

#[cfg(test)]
mod tests {
    use super::vdf_value::{parse, VdfValue};
    use std::collections::BTreeMap;

    #[test]
    fn test_parse_simple_vdf() {
        let input = r#""Key" "Value""#;
        let expected = VdfValue::Table({
            let mut map = BTreeMap::new();
            map.insert("Key".to_string(), VdfValue::Value("Value".to_string()));
            map
        });
        assert_eq!(parse(input, false).unwrap(), expected);
    }

    #[test]
    fn test_parse_nested_vdf() {
        let input = r#""RootKey"
{
    "SubKey1" "Value1"
    "SubKey2"
    {
        "NestedKey" "NestedValue"
    }
    "SubKey3" "Value3"
}"#;
        let expected = VdfValue::Table({
            let mut root_map = BTreeMap::new();
            let mut sub_map = BTreeMap::new();
            let mut nested_map = BTreeMap::new();

            nested_map.insert("NestedKey".to_string(), VdfValue::Value("NestedValue".to_string()));
            sub_map.insert("SubKey1".to_string(), VdfValue::Value("Value1".to_string()));
            sub_map.insert("SubKey2".to_string(), VdfValue::Table(nested_map));
            sub_map.insert("SubKey3".to_string(), VdfValue::Value("Value3".to_string()));

            root_map.insert("RootKey".to_string(), VdfValue::Table(sub_map));
            root_map
        });
        assert_eq!(parse(input, false).unwrap(), expected);
    }

    #[test]
    fn test_parse_vdf_with_comments() {
        let input = r#""RootKey" // This is a comment
{
    "SubKey1" "Value1" // Another comment
    // This is a full line comment
    "SubKey2"
    {
        "NestedKey" "NestedValue"
    }
    "SubKey3" "Value3"
}"#;
        let expected = VdfValue::Table({
            let mut root_map = BTreeMap::new();
            let mut sub_map = BTreeMap::new();
            let mut nested_map = BTreeMap::new();

            nested_map.insert("NestedKey".to_string(), VdfValue::Value("NestedValue".to_string()));
            sub_map.insert("SubKey1".to_string(), VdfValue::Value("Value1".to_string()));
            sub_map.insert("SubKey2".to_string(), VdfValue::Table(nested_map));
            sub_map.insert("SubKey3".to_string(), VdfValue::Value("Value3".to_string()));

            root_map.insert("RootKey".to_string(), VdfValue::Table(sub_map));
            root_map
        });
        assert_eq!(parse(input, false).unwrap(), expected);
    }
}
