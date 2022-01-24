use std::fmt;
use quick_xml::{events, Reader};
use serde::{Serialize, Deserialize};

fn parse_int(s: &str) -> Result<usize, ParseError> {
    if s.starts_with("0x") {
        usize::from_str_radix(&s[2..], 16)
    } else {
        usize::from_str_radix(s, 10)
    }.map_err(|e| ParseError::InvalidInteger(e))
}
fn clean(string: String) -> String {
    let mut trimmed = String::new();
    let mut first= true;
    for str in string.split_terminator('\n') {
        if !first { trimmed.push('\n') } else { first = false }
        trimmed.push_str(str.trim());
    }
    trimmed.replace("  ", " ")
}

// Note: owned strings are required as TOML allows string normalisation

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Protocol {
    name: String,
    summary: Option<String>,
    description: Option<String>,
    copyright: Option<String>,
    #[serde(rename = "interface", default)]
    interfaces: Vec<Interface>
}
impl Protocol {
    pub fn load_toml(string: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(string)
    }
    pub fn to_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&self)
    }
    pub fn load_xml(xml: &str) -> Result<Self, ParseError> {

        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        let mut protocol = None;

        // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
        let mut buffer = Vec::with_capacity(5);
        #[derive(Debug)]
        enum EventType {
            Normal,
            Destructor
        }
        impl EventType {
            fn is_destructor(&self) -> bool {
                match self {
                    Self::Normal => false,
                    Self::Destructor => true
                }
            }
        }
        #[derive(Debug)]
        enum State {
            Protocol {
                name: Option<String>,
                summary: Option<String>,
                description: Option<String>,
                copyright: Option<String>,
                interfaces: Vec<Interface>
            },
            Interface {
                name: Option<String>,
                summary: Option<String>,
                description: Option<String>,
                version: Option<u16>,
                enums: Vec<Enum>,
                requests: Vec<Request>,
                events: Vec<Event>,
            },
            Request {
                name: Option<String>,
                since: Option<u16>,
                destructor: bool,
                summary: Option<String>,
                description: Option<String>,
                args: Vec<Arg>
            },
            Event {
                name: Option<String>,
                destructor: bool,
                since: Option<u16>,
                summary: Option<String>,
                description: Option<String>,
                args: Vec<Arg>
            },
            Enum {
                name: Option<String>,
                since: Option<u16>,
                summary: Option<String>,
                description: Option<String>,
                entries: Vec<Entry>
            },
            Entry {
                name: Option<String>,
                since: Option<u16>,
                summary: Option<String>,
                description: Option<String>,
                value: Option<u32>
            },
            Copyright {
                copyright: Option<String>
            },
            Description {
                summary: Option<String>,
                description: Option<String>
            }
        }
        impl State {
            fn name(&mut self, v: String) -> Result<(), ParseError> {
                match self {
                    Self::Protocol { name, ..} => Ok(*name = Some(v)),
                    Self::Interface { name, ..} => Ok(*name = Some(v)),
                    Self::Request { name, ..} => Ok(*name = Some(v)),
                    Self::Event { name, ..} => Ok(*name = Some(v)),
                    Self::Enum { name, ..} => Ok(*name = Some(v)),
                    Self::Entry { name, ..} => Ok(*name = Some(v)),
                    _ => Err(ParseError::NoField("name", self.debug()))
                }
            }
            fn copyright(&mut self, v: String) -> Result<(), ParseError> {
                let v = clean(v);
                match self {
                    Self::Protocol { copyright, ..} => Ok(*copyright = Some(v)),
                    _ => Err(ParseError::NoField("copyright", self.debug()))
                }
            }
            fn version(&mut self, v: String) -> Result<(), ParseError> {
                match self {
                    Self::Interface { version, ..} => Ok(*version = Some(parse_int(&v)? as u16)),
                    _ => Err(ParseError::NoField("version", self.debug()))
                }
            }
            fn since(&mut self, v: String) -> Result<(), ParseError> {
                match self {
                    Self::Request { since, ..} => Ok(*since = Some(parse_int(&v)? as u16)),
                    Self::Event { since, ..} => Ok(*since = Some(parse_int(&v)? as u16)),
                    Self::Enum { since, ..} => Ok(*since = Some(parse_int(&v)? as u16)),
                    Self::Entry { since, ..} => Ok(*since = Some(parse_int(&v)? as u16)),
                    _ => Err(ParseError::NoField("since", self.debug()))
                }
            }
            fn summary(&mut self, v: String) -> Result<(), ParseError> {
                let v = clean(v);
                match self {
                    Self::Protocol { summary, ..} => Ok(*summary = Some(v)),
                    Self::Interface { summary, ..} => Ok(*summary = Some(v)),
                    Self::Request { summary, ..} => Ok(*summary = Some(v)),
                    Self::Event { summary, ..} => Ok(*summary = Some(v)),
                    Self::Enum { summary, ..} => Ok(*summary = Some(v)),
                    Self::Entry { summary, ..} => Ok(*summary = Some(v)),
                    Self::Description { summary, ..} => Ok(*summary = Some(v)),
                    _ => Err(ParseError::NoField("summary", self.debug()))
                }
            }
            fn description(&mut self, v: String) -> Result<(), ParseError> {
                let v = clean(v);
                match self {
                    Self::Protocol { description, ..} => Ok(*description = Some(v)),
                    Self::Interface { description, ..} => Ok(*description = Some(v)),
                    Self::Request { description, ..} => Ok(*description = Some(v)),
                    Self::Event { description, ..} => Ok(*description = Some(v)),
                    Self::Enum { description, ..} => Ok(*description = Some(v)),
                    Self::Entry { description, ..} => Ok(*description = Some(v)),
                    _ => Err(ParseError::NoField("description", self.debug()))
                }
            }
            fn kind(&mut self, v: String) -> Result<(), ParseError> {
                match self {
                    Self::Request { destructor, ..} | Self::Event { destructor, ..} if v == "destructor" => Ok(*destructor = true),
                    _ => Err(ParseError::NoField("type", self.debug()))
                }
            }
            fn arg(&mut self, arg: Arg) -> Result<(), ParseError> {
                match self {
                    Self::Request { args, ..} => Ok(args.push(arg)),
                    Self::Event { args, ..} => Ok(args.push(arg)),
                    _ => Err(ParseError::NoField("arg", self.debug()))
                }
            }
            fn value(&mut self, v: u32) -> Result<(), ParseError> {
                match self {
                    Self::Entry { value, ..} => Ok(*value = Some(v)),
                    _ => Err(ParseError::NoField("value", self.debug()))
                }
            }
            fn entry(&mut self, entry: Entry) -> Result<(), ParseError> {
                match self {
                    Self::Enum { entries, ..} => Ok(entries.push(entry)),
                    _ => Err(ParseError::NoField("entry", self.debug()))
                }
            }
            fn interface(&mut self, interface: Interface) -> Result<(), ParseError> {
                match self {
                    Self::Protocol { interfaces, ..} => Ok(interfaces.push(interface)),
                    _ => Err(ParseError::NoField("interface", self.debug()))
                }
            }
            fn request(&mut self, request: Request) -> Result<(), ParseError> {
                match self {
                    Self::Interface { requests, ..} => Ok(requests.push(request)),
                    _ => Err(ParseError::NoField("request", self.debug()))
                }
            }
            fn event(&mut self, event: Event) -> Result<(), ParseError> {
                match self {
                    Self::Interface { events, ..} => Ok(events.push(event)),
                    _ => Err(ParseError::NoField("event", self.debug()))
                }
            }
            fn enumeration(&mut self, enumeration: Enum) -> Result<(), ParseError> {
                match self {
                    Self::Interface { enums, ..} => Ok(enums.push(enumeration)),
                    _ => Err(ParseError::NoField("enum", self.debug()))
                }
            }
            fn debug(&self) -> &'static str {
                match self {
                    Self::Protocol {..} => "protocol",
                    Self::Interface {..} => "interface",
                    Self::Request {..} => "request",
                    Self::Event {..} => "event",
                    Self::Enum {..} => "enum",
                    Self::Entry {..} => "entry",
                    Self::Copyright {..} => "copyright",
                    Self::Description {..} => "description"
                }
            }
        }
        fn for_each<F: FnMut(quick_xml::events::attributes::Attribute) -> Result<(), ParseError>>(attributes: quick_xml::events::attributes::Attributes, mut f: F) -> Result<(), ParseError> {
            for a in attributes {
                let a = a?;
                f(a)?
            }
            Ok(())
        }
        let mut state = vec![];
        loop {
            use events::Event;
            match reader.read_event(&mut buffer) {
                Ok(Event::Start(ref e)) => {
                    match e.name() {
                        b"protocol" => {
                            let mut protocol = State::Protocol {
                                name: None,
                                summary: None,
                                description: None,
                                copyright: None,
                                interfaces: vec![]
                            };
                            for_each(e.attributes(), |a| match a.key {
                                b"name" => protocol.name(a.unescape_and_decode_value(&reader)?),
                                _ => Ok(())
                            })?;
                            state.push(protocol)
                        },
                        b"copyright" => state.push(State::Copyright { copyright: None }),
                        b"interface" => {
                            let mut interface = State::Interface {
                                name: None,
                                summary: None,
                                description: None,
                                version: None,
                                requests: vec![],
                                events: vec![],
                                enums: vec![]
                            };
                            for_each(e.attributes(), |a| match a.key {
                                b"name" => interface.name(a.unescape_and_decode_value(&reader)?),
                                b"version" => interface.version(a.unescape_and_decode_value(&reader)?),
                                _ => Ok(())
                            })?;
                            state.push(interface)
                        },
                        b"description" => {
                            let mut description = State::Description {
                                summary: None,
                                description: None
                            };
                            for_each(e.attributes(), |a| match a.key {
                                b"summary" => description.summary(a.unescape_and_decode_value(&reader)?),
                                _ => Ok(())
                            })?;
                            state.push(description)
                        },
                        b"request" => {
                            let mut request = State::Request {
                                name: None,
                                since: None,
                                summary: None,
                                description: None,
                                destructor: false,
                                args: vec![]
                            };
                            for_each(e.attributes(), |a| match a.key {
                                b"name" => request.name(a.unescape_and_decode_value(&reader)?),
                                b"since" => request.since(a.unescape_and_decode_value(&reader)?),
                                b"type" => request.kind(a.unescape_and_decode_value(&reader)?),
                                _ => Ok(())
                            })?;
                            state.push(request)
                        },
                        b"event" => {
                            let mut event = State::Event {
                                name: None,
                                destructor: false,
                                since: None,
                                summary: None,
                                description: None,
                                args: vec![]
                            };
                            for_each(e.attributes(), |a| match a.key {
                                b"name" => event.name(a.unescape_and_decode_value(&reader)?),
                                b"since" => event.since(a.unescape_and_decode_value(&reader)?),
                                b"type" => event.kind(a.unescape_and_decode_value(&reader)?),
                                _ => Ok(())
                            })?;
                            state.push(event)
                        },
                        b"enum" => {
                            let mut enumeration = State::Enum {
                                name: None,
                                since: None,
                                summary: None,
                                description: None,
                                entries: vec![]
                            };
                            for_each(e.attributes(), |a| match a.key {
                                b"name" => enumeration.name(a.unescape_and_decode_value(&reader)?),
                                b"since" => enumeration.since(a.unescape_and_decode_value(&reader)?),
                                _ => Ok(())
                            })?;
                            state.push(enumeration)
                        },
                        b"entry" => {
                            let mut entry = State::Entry {
                                name: None,
                                since: None,
                                summary: None,
                                description: None,
                                value: None
                            };
                            for_each(e.attributes(), |a| match a.key {
                                b"name" => entry.name(a.unescape_and_decode_value(&reader)?),
                                b"since" => entry.since(a.unescape_and_decode_value(&reader)?),
                                b"value" => entry.value(parse_int(&a.unescape_and_decode_value(&reader)?)? as u32),
                                _ => Ok(())
                            })?;
                            state.push(entry)
                        },
                        n => return Err(ParseError::UnexpectedTag(String::from_utf8_lossy(n).into())),
                    }
                },
                Ok(Event::Empty(e)) => {
                    match e.name() {
                        b"arg" => {
                            let parent = state.last_mut().ok_or(ParseError::ExpectedParent("arg"))?;
                            let mut name = None;
                            let mut interface = None;
                            let mut nullable = None;
                            let mut kind = None;
                            let mut enumeration = None;
                            let mut summary = None;
                            for_each(e.attributes(), |a| match a.key {
                                b"name" => Ok(name = Some(a.unescape_and_decode_value(&reader)?)),
                                b"interface" => Ok(interface = Some(a.unescape_and_decode_value(&reader)?)),
                                b"type" => Ok(kind = Some(a.unescape_and_decode_value(&reader)?)),
                                b"allow-null" => Ok(nullable = Some(a.unescape_and_decode_value(&reader)?)),
                                b"enum" => Ok(enumeration = Some(a.unescape_and_decode_value(&reader)?)),
                                b"summary" => Ok(summary = Some(a.unescape_and_decode_value(&reader)?)),
                                _ => Ok(())
                            })?;
                            parent.arg(Arg {
                                name: name.ok_or(ParseError::MissingField("name", "arg"))?,
                                summary,
                                nullable: if let Some(val) = nullable { Some(val.parse()?) } else { None },
                                kind: DataType::from_str(&kind.ok_or(ParseError::MissingField("type", "arg"))?)?,
                                interface,
                                enumeration
                            })?;
                        },
                        b"entry" => {
                            let parent = state.last_mut().ok_or(ParseError::ExpectedParent("entry"))?;
                            let mut name = None;
                            let mut value = None;
                            let mut summary = None;
                            let mut since = None;
                            for_each(e.attributes(), |a| match a.key {
                                b"name" => Ok(name = Some(a.unescape_and_decode_value(&reader)?)),
                                b"value" => Ok(value = Some(a.unescape_and_decode_value(&reader)?)),
                                b"summary" => Ok(summary = Some(a.unescape_and_decode_value(&reader)?)),
                                b"since" => Ok(since = Some(a.unescape_and_decode_value(&reader)?)),
                                _ => Ok(())
                            })?;
                            let since = if let Some(since) = since {
                                Some(parse_int(&since)? as u16)
                            } else {
                                None
                            };
                            parent.entry(Entry {
                                name: name.ok_or(ParseError::MissingField("name", "arg"))?,
                                value: parse_int(&value.ok_or(ParseError::MissingField("value", "arg"))?)? as u32,
                                description: None,
                                summary,
                                since
                            })?;
                        },
                        b"description" => {
                            let parent = state.last_mut().ok_or(ParseError::ExpectedParent("description"))?;
                            let mut summary = None;
                            for_each(e.attributes(), |a| match a.key {
                                b"summary" => Ok(summary = Some(a.unescape_and_decode_value(&reader)?)),
                                _ => Ok(())
                            })?;
                            if let Some (summary) = summary { parent.summary(summary)? };
                        },
                        b"request" => {
                            let parent = state.last_mut().ok_or(ParseError::ExpectedParent("interface"))?;
                            let mut name = None;
                            let mut since = None;
                            let mut summary = None;
                            let mut kind = None;
                            for_each(e.attributes(), |a| match a.key {
                                b"name" => Ok(name = Some(a.unescape_and_decode_value(&reader)?)),
                                b"since" => Ok(since = Some(parse_int(&a.unescape_and_decode_value(&reader)?)? as u16)),
                                b"summary" => Ok(summary = Some(a.unescape_and_decode_value(&reader)?)),
                                b"type" => Ok(kind = Some(a.unescape_and_decode_value(&reader)?)),
                                _ => Ok(())
                            })?;
                            let destructor = if let Some(kind) = kind { kind == "destructor" } else { false };
                            parent.request(Request {
                                name: name.ok_or(ParseError::MissingField("name", "request"))?,
                                since,
                                destructor,
                                summary,
                                description: None,
                                args: vec![],
                            })?;
                        },
                        n => return Err(ParseError::UnexpectedTag(String::from_utf8_lossy(n).into())),
                    }
                }
                Ok(Event::Text(e)) => {
                    let string = e.unescape_and_decode(&reader)?;
                    match state.last_mut().ok_or(ParseError::UnexpectedText)? {
                        State::Copyright { copyright } => {
                            *copyright = Some(string);
                        }
                        State::Description { description, ..} => {
                            *description = Some(string)
                        }
                        state => return Err(ParseError::TagContents(state.debug()))
                    }
                },
                Ok(Event::End(_)) => {
                    match state.pop().unwrap() {
                        State::Protocol { name, summary, description, copyright, interfaces } => protocol = Some(Protocol {
                            name: name.ok_or(ParseError::MissingField("name", "protocol"))?,
                            summary,
                            description,
                            copyright,
                            interfaces
                        }),
                        State::Interface { name, version, summary, description, requests, events, enums } => {
                            let parent = state.last_mut().ok_or(ParseError::ExpectedParent("interface"))?;
                            parent.interface(Interface {
                                name: name.ok_or(ParseError::MissingField("name", "interface"))?,
                                summary,
                                description,
                                version: version.ok_or(ParseError::MissingField("version", "interface"))?,
                                enums,
                                requests,
                                events
                            })?;
                        },
                        State::Request { name,since, destructor, summary, description, args } => {
                            let parent = state.last_mut().ok_or(ParseError::ExpectedParent("request"))?;
                            parent.request(Request {
                                name: name.ok_or(ParseError::MissingField("name", "request"))?,
                                summary,
                                destructor,
                                description,
                                since,
                                args,
                            })?;
                        },
                        State::Event { name, destructor, since, summary, description, args } => {
                            let parent = state.last_mut().ok_or(ParseError::ExpectedParent("event"))?;
                            parent.event(crate::protocol::Event {
                                name: name.ok_or(ParseError::MissingField("name", "event"))?,
                                destructor,
                                summary,
                                description,
                                since,
                                args,
                            })?;
                        },
                        State::Enum { name,since, summary, description, entries } => {
                            let parent = state.last_mut().ok_or(ParseError::ExpectedParent("enum"))?;
                            parent.enumeration(Enum {
                                name: name.ok_or(ParseError::MissingField("name", "enum"))?,
                                summary,
                                description,
                                since,
                                entries,
                            })?;
                        },
                        State::Entry { name,since, summary, description, value } => {
                            let parent = state.last_mut().ok_or(ParseError::ExpectedParent("enum"))?;
                            parent.entry(Entry {
                                name: name.ok_or(ParseError::MissingField("name", "entry"))?,
                                summary,
                                description,
                                since,
                                value: value.ok_or(ParseError::MissingField("value", "entry"))?,
                            })?;
                        },
                        State::Description { summary, description } => {
                            let parent = state.last_mut().ok_or(ParseError::ExpectedParent("description"))?;
                            if let Some(summary) = summary { parent.summary(summary)? }
                            if let Some(description) = description { parent.description(description)? }
                        },
                        State::Copyright { copyright } => {
                            let parent = state.last_mut().ok_or(ParseError::ExpectedParent("description"))?;
                            if let Some(copyright) = copyright { parent.copyright(copyright)? }
                        }
                    }
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                Err(e) => return Err(ParseError::Xml(e)),
                _ => (), // There are several other `Event`s we do not consider here
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buffer.clear();
        }
        protocol.ok_or(ParseError::Missing("protocol"))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Interface {
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub version: u16,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "enum", default)]
    pub enums: Vec<Enum>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "request", default)]
    pub requests: Vec<Request>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "event", default)]
    pub events: Vec<Event>
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Enum {
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub since: Option<u16>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "entry", default)]
    pub entries: Vec<Entry>
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Request {
    pub name: String,
    pub since: Option<u16>,
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub destructor: bool,
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "arg", default)]
    pub args: Vec<Arg>
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub name: String,
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub destructor: bool,
    pub since: Option<u16>,
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "arg", default)]
    pub args: Vec<Arg>
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub since: Option<u16>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub value: u32
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Arg {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: DataType,
    #[serde(rename = "allow-null")]
    pub nullable: Option<bool>,
    pub interface: Option<String>,
    #[serde(rename = "enum")]
    pub enumeration: Option<String>,
    pub summary: Option<String>
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    Int,
    Uint,
    Fixed,
    String,
    Array,
    Fd,
    Object,
    NewId
}
impl DataType {
    fn from_str(s: &str) -> Result<Self, ParseError> {
        match s {
            "int" => Ok(Self::Int),
            "uint" => Ok(Self::Uint),
            "fixed" => Ok(Self::Fixed),
            "string" => Ok(Self::String),
            "array" => Ok(Self::Array),
            "fd" => Ok(Self::Fd),
            "object" => Ok(Self::Object),
            "new_id" => Ok(Self::NewId),
            d => Err(ParseError::InvalidDataType(d.into()))
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    Xml(quick_xml::Error),
    Missing(&'static str),
    UnexpectedTag(String),
    TagContents(&'static str),
    NoField(&'static str, &'static str),
    MissingField(&'static str, &'static str),
    UnexpectedText,
    ExpectedParent(&'static str),
    InvalidInteger(std::num::ParseIntError),
    InvalidBool(std::str::ParseBoolError),
    InvalidDataType(String)
}
impl From<quick_xml::Error> for ParseError {
    fn from(e: quick_xml::Error) -> Self {
        Self::Xml(e)
    }
}
impl From<std::num::ParseIntError> for ParseError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::InvalidInteger(e)
    }
}
impl From<std::str::ParseBoolError> for ParseError {
    fn from(e: std::str::ParseBoolError) -> Self {
        Self::InvalidBool(e)
    }
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Xml(e) => write!(f, "{}", e),
            Self::Missing(thing) => write!(f, "Missing {}", thing),
            Self::UnexpectedTag(tag) => write!(f, "Unexpected tag {:?}", tag),
            Self::TagContents(tag) => write!(f, "Tag {:?} has contents but should not", tag),
            Self::NoField(field, tag) => write!(f, "Tag {:?} unexpectedly contains attribute {:?}", tag, field),
            Self::MissingField(field, tag) => write!(f, "Attribute {:?} is required but tag {:?} does not provide it", field, tag),
            Self::UnexpectedText => write!(f, "Expected a tag but got text"),
            Self::ExpectedParent(tag) => write!(f, "Tag {:?} requires nesting", tag),
            Self::InvalidInteger(i) => write!(f, "{}", i),
            Self::InvalidBool(b) => write!(f, "{}", b),
            Self::InvalidDataType(d) => write!(f, "{:?} is not a valid data type", d),
        }
    }
}