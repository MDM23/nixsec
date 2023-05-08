use std::{
    collections::BTreeMap,
    fs::{self, read_to_string},
    path::{Path, PathBuf},
};

use nixel::{Binding, BindingKeyValue, Expression, FunctionHead, Part, Span};

#[derive(Clone, Debug)]
pub struct ConfigFile {
    path: PathBuf,
    source: String,
    secrets_span: Span,
    data: ConfigFileData,
}

#[derive(Clone, Debug, Default)]
pub struct ConfigFileData {
    admins: BTreeMap<String, String>,
    hosts: BTreeMap<String, String>,
    derivations: BTreeMap<String, Derivation>,
    secrets: BTreeMap<String, Secret>,
}

#[derive(Clone, Debug, Default)]
pub struct Derivation {
    input: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Secret {
    hosts: Vec<String>,
    enc: String,
}

impl ConfigFile {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ()> {
        let source = read_to_string(path.as_ref()).expect("Could not open file");

        let ast = nixel::parse(source.clone());
        let mut data = ConfigFileData::default();
        let mut secret_span = None;

        let Expression::Map(map) = ast.expression.as_ref() else {
            return Err(());
        };

        for b in map.bindings.iter() {
            let Binding::KeyValue(BindingKeyValue { from, to }) = b else {
                continue
            };

            let Some(Part::Raw(part)) = from.first() else {
                continue
            };

            match &*part.content {
                "admins" => {
                    data.admins = Self::read_string_object(to.as_ref());
                }
                "hosts" => {
                    data.hosts = Self::read_string_object(to.as_ref());
                }
                "derivations" => {
                    data.derivations = Self::get_derivations(to.as_ref());
                }
                "secrets" => {
                    secret_span = Some(Span {
                        start: part.span.start.clone(),
                        end: to.span().end,
                    });

                    data.secrets = Self::get_secrets(to.as_ref());
                }
                _ => (),
            };
        }

        Ok(Self {
            path: path.as_ref().to_owned(),
            source: source.to_owned(),
            secrets_span: secret_span.unwrap(),
            data,
        })
    }

    pub fn write(&self) {
        let mut iter = self.source.chars();
        let mut buffer = String::new();
        let mut indent = 0;
        let mut current_line = 1;

        if self.secrets_span.start.line > 1 {
            loop {
                let Some(ch) = iter.next() else {
                    break;
                };

                buffer.push(ch);
                if ch == '\n' {
                    current_line += 1;
                }
                if current_line == self.secrets_span.start.line {
                    break;
                }
            }
        }

        if self.secrets_span.start.column > 1 {
            let mut current_column = 1;
            loop {
                let Some(ch) = iter.next() else {
                    break;
                };

                buffer.push(ch);
                if ch == ' ' {
                    indent += 1;
                }
                current_column += 1;
                if current_column == self.secrets_span.start.column {
                    break;
                }
            }
        }

        buffer.push_str(&self.secrets_to_nix(indent));

        if self.secrets_span.end.line > 1 {
            loop {
                let Some(ch) = iter.next() else {
                    break;
                };

                if ch == '\n' {
                    current_line += 1;
                }
                if current_line == self.secrets_span.end.line {
                    break;
                }
            }
        }

        if self.secrets_span.end.column > 1 {
            let mut current_column = 1;
            loop {
                let Some(_) = iter.next() else {
                    break;
                };

                current_column += 1;
                if current_column == self.secrets_span.end.column {
                    break;
                }
            }
        }

        buffer.push_str(&iter.collect::<String>());

        let mut target = self.path.clone();
        target.set_extension("new.nix");

        fs::write(target, buffer).expect("Unable to write file");

        // dbg!(&buffer, indent);
    }
}

impl ConfigFile {
    fn read_string_object(expr: &Expression) -> BTreeMap<String, String> {
        let mut result = BTreeMap::new();

        let Expression::Map(map) = expr else {
            return result;
        };

        for b in map.bindings.iter() {
            let Binding::KeyValue(BindingKeyValue { from, to }) = b else {
                continue
            };

            let [Part::Raw(name)] = from.as_ref() else {
                continue
            };

            let Expression::String(pubkey) = to.as_ref() else {
                continue
            };

            let [Part::Raw(pubkey)] = pubkey.parts.as_ref() else {
                continue
            };

            result.insert(name.content.to_string(), pubkey.content.to_string());
        }

        result
    }

    fn read_string_list(expr: &Expression) -> Vec<String> {
        let mut result = Vec::new();

        let Expression::List(list) = expr else {
            return result;
        };

        for e in list.elements.iter() {
            let Expression::String(host) = e else {
                continue
            };

            let [Part::Raw(host)] = host.parts.as_ref() else {
                continue
            };

            result.push(host.content.to_string());
        }

        result
    }

    fn get_secrets(secrets: &Expression) -> BTreeMap<String, Secret> {
        let mut result = BTreeMap::new();

        let Expression::Map(map) = secrets else {
            return result;
        };

        for b in map.bindings.iter() {
            let Binding::KeyValue(BindingKeyValue { from, to }) = b else {
                continue
            };

            let [Part::Raw(name), Part::Raw(attr)] = from.as_ref() else {
                continue
            };

            let secret = result.entry(name.content.to_string()).or_default();

            match attr.content.as_ref() {
                "hosts" => {
                    secret.hosts = Self::read_string_list(to);
                }
                "enc" => {
                    let Expression::IndentedString(enc) = to.as_ref() else {
                        continue
                    };

                    let [Part::Raw(enc)] = enc.parts.as_ref() else {
                        continue
                    };

                    secret.enc = enc.content.trim().to_string();
                }
                _ => (),
            };
        }

        result
    }

    fn get_derivations(derivations: &Expression) -> BTreeMap<String, Derivation> {
        let mut result = BTreeMap::new();

        let Expression::Map(map) = derivations else {
            return result;
        };

        for b in map.bindings.iter() {
            let Binding::KeyValue(BindingKeyValue { from, to }) = b else {
                continue
            };

            let [Part::Raw(name)] = from.as_ref() else {
                continue
            };

            let derivation = result.entry(name.content.to_string()).or_default();

            let Expression::Function(func) = to.as_ref() else {
                return result;
            };

            let FunctionHead::Destructured(head) = &func.head else {
                return result;
            };

            for arg in head.arguments.iter() {
                derivation.input.push(arg.identifier.to_string());
            }
        }

        result
    }

    fn secrets_to_nix(&self, base_indent: usize) -> String {
        let tab = " ".repeat(base_indent);
        let tab2 = tab.repeat(2);
        let tab3 = tab.repeat(3);

        let mut buf = String::from("secrets = {\n");

        for (name, secret) in self.data.secrets.iter() {
            buf.push_str(&tab2);
            buf.push_str(name);
            buf.push_str(&format!(".hosts = {:?};\n", secret.hosts));
            buf.push_str(&tab2);
            buf.push_str(name);
            buf.push_str(&format!(".enc = ''\n{tab3}"));
            buf.push_str(
                &secret
                    .enc
                    .lines()
                    .map(|l| l.to_string())
                    .collect::<Vec<String>>()
                    .join(&format!("\n{tab3}")),
            );
            buf.push_str("'';\n");
        }

        buf.push_str(&tab);
        buf.push('}');

        buf
    }
}
