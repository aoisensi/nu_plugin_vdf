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
            example: r#"'"Key" "Value"' | from vdf"#,
            description: "Parse a simple VDF string",
            result: Some(Value::test_record(
                nu_protocol::record! { "Key" => Value::test_string("Value") },
            )),
        }]
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