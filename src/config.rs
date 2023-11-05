use std::{cell::RefCell, cmp::Ordering, fmt::Display, rc::Rc};

use crate::errors;

type ConfigRef<T> = Rc<T>;

type NetworkRef = ConfigRef<Network>;

fn make_config_ref<T>(value: T) -> ConfigRef<T> {
    Rc::new(value)
}

#[derive(Debug)]
pub struct Network {
    build_time: chrono::DateTime<chrono::Local>,
    baudrate: u32,
    nodes: Vec<NodeRef>,
    messages: Vec<MessageRef>,
    types: Vec<TypeRef>,
}

pub type NodeRef = ConfigRef<Node>;

#[derive(Debug)]
pub struct Node {
    name: String,
    description: Option<String>,

    types: Vec<TypeRef>,

    commands: Vec<CommandRef>,
    extern_commands: Vec<(String, CommandRef)>,

    tx_streams: Vec<StreamRef>,
    rx_streams: Vec<StreamRef>,

    rx_messages: Vec<MessageRef>,
    tx_messages: Vec<MessageRef>,

    object_entries: Vec<ObjectEntryRef>,
    get_resp_message: MessageRef,
    get_req_message: MessageRef,
    set_resp_message: MessageRef,
    set_req_message: MessageRef,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Global,
    Static,
}

pub type TypeRef = ConfigRef<Type>;

#[derive(Debug, PartialEq)]
pub enum Type {
    Primitive(SignalType),
    Struct {
        name: String,
        description: Option<String>,
        attribs: Vec<(String, TypeRef)>,
        visibility: Visibility,
    },
    Enum {
        name: String,
        description: Option<String>,
        size: u8,
        entries: Vec<(String, u64)>,
        visibility: Visibility,
    },
    Array {
        len: usize,
        ty: TypeRef,
    },
}

pub type CommandRef = ConfigRef<Command>;

#[derive(Debug)]
pub struct Command {
    name: String,
    description: Option<String>,
    tx_message: MessageRef,
    rx_message: MessageRef,
    visibility: Visibility,
}

type StreamRef = ConfigRef<Stream>;

#[derive(Debug)]
pub struct Stream {
    name: String,
    description: Option<String>,
    mappings: Vec<Option<ObjectEntryRef>>,
    message: MessageRef,
    visibility: Visibility,
}

pub type ObjectEntryRef = ConfigRef<ObjectEntry>;

#[derive(Debug, Clone)]
pub enum ObjectEntryAccess {
    Const,  // no write
    Local,  // local write public read
    Global, // public write
}

#[derive(Debug)]
pub struct ObjectEntry {
    name: String,
    description: Option<String>,
    id: u32,
    ty: TypeRef,
    access: ObjectEntryAccess,
    visibility: Visibility,
}

#[derive(Debug)]
pub enum MessageId {
    StandardId(u32),
    ExtendedId(u32),
}

pub type MessageRef = ConfigRef<Message>;

#[derive(Debug)]
pub struct Message {
    name: String,
    description: Option<String>,
    id: MessageId,
    encoding: Option<MessageEncoding>,
    signals: Vec<SignalRef>,
    visbility: Visibility,
}

// describes how to map Type to signals.
// vector of elements with name and type of the encoded Types
pub type MessageEncoding = Vec<TypeSignalEncoding>;

#[derive(Debug)]
pub struct TypeSignalEncoding {
    name: String,
    ty: TypeRef,
    signals: Vec<SignalRef>,
}

#[derive(Debug)]
pub enum SignalSign {
    Signed,
    Unsigned,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignalType {
    UnsignedInt { size: u8 },
    SignedInt { size: u8 },
    Decimal { size: u8, offset: f64, scale: f64 },
}

pub type SignalRef = ConfigRef<Signal>;

#[derive(Debug, Clone)]
pub struct Signal {
    name: String,
    description: Option<String>,
    ty: SignalType,
    value_table: Option<ValueTableRef>,
    offset: usize,
}

pub type ValueTableRef = ConfigRef<ValueTable>;
#[derive(Debug, Clone)]
pub struct ValueTable(Vec<(String, u64)>);

impl Network {
    pub fn new(
        baudrate: u32,
        build_time: chrono::DateTime<chrono::Local>,
        nodes: Vec<NodeRef>,
        messages: Vec<MessageRef>,
        types: Vec<TypeRef>,
    ) -> Network {
        Network {
            types,
            build_time,
            baudrate,
            nodes,
            messages,
        }
    }
    pub fn nodes(&self) -> &Vec<NodeRef> {
        &self.nodes
    }
    pub fn messages(&self) -> &Vec<MessageRef> {
        &self.messages
    }
    pub fn baudrate(&self) -> u32 {
        self.baudrate
    }
    pub fn build_time(&self) -> &chrono::DateTime<chrono::Local> {
        &self.build_time
    }
    pub fn types(&self) -> &Vec<TypeRef> {
        &self.types
    }
}

impl Node {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn types(&self) -> &Vec<TypeRef> {
        &self.types
    }
    pub fn commands(&self) -> &Vec<CommandRef> {
        &self.commands
    }
    pub fn extern_commands(&self) -> &Vec<(String, CommandRef)> {
        &self.extern_commands
    }
    pub fn tx_streams(&self) -> &Vec<StreamRef> {
        &self.tx_streams
    }
    pub fn rx_streams(&self) -> &Vec<StreamRef> {
        &self.rx_streams
    }
    pub fn tx_messages(&self) -> &Vec<MessageRef> {
        &self.tx_messages
    }
    pub fn rx_messages(&self) -> &Vec<MessageRef> {
        &self.rx_messages
    }
    pub fn object_entries(&self) -> &Vec<ObjectEntryRef> {
        &self.object_entries
    }
    pub fn description(&self) -> Option<&String> {
        match &self.description {
            Some(some) => Some(&some),
            None => None,
        }
    }
    pub fn get_resp_message(&self) -> &Message {
        &self.get_resp_message
    }
    pub fn get_req_message(&self) -> &Message {
        &self.get_req_message
    }
    pub fn set_resp_message(&self) -> &Message {
        &self.set_resp_message
    }
    pub fn set_req_message(&self) -> &Message {
        &self.set_req_message
    }
}

impl Type {
    pub fn name(&self) -> String {
        match &self {
            Type::Primitive(signal_type) => match signal_type {
                SignalType::UnsignedInt { size } => {
                    return format!("u{size}");
                }
                SignalType::SignedInt { size } => {
                    return format!("i{size}");
                }
                SignalType::Decimal {
                    size,
                    offset,
                    scale,
                } => {
                    return format!("d{size}<offset={offset}, scale={scale}>");
                }
            },
            Type::Struct {
                name,
                description: _,
                attribs: _,
                visibility: _,
            } => name.to_owned(),
            Type::Enum {
                name,
                description: _,
                size: _,
                entries: _,
                visibility: _,
            } => name.to_owned(),
            Type::Array { len, ty } => format!("{}[{len}]", ty.name()),
        }
    }
}

impl Command {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> Option<&String> {
        match &self.description {
            Some(some) => Some(&some),
            None => None,
        }
    }
    pub fn tx_message(&self) -> &Message {
        &self.tx_message
    }
    pub fn rx_message(&self) -> &Message {
        &self.rx_message
    }
}

impl Stream {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> Option<&str> {
        match &self.description {
            Some(some) => Some(&some),
            None => None,
        }
    }
    pub fn mapping(&self) -> &Vec<Option<ObjectEntryRef>> {
        &self.mappings
    }
    pub fn message(&self) -> &MessageRef {
        &self.message
    }
}

impl ObjectEntry {
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> Option<&str> {
        match &self.description {
            Some(some) => Some(&some),
            None => None,
        }
    }
    pub fn ty(&self) -> &TypeRef {
        &self.ty
    }
}

impl Message {
    pub fn id(&self) -> &MessageId {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> Option<&str> {
        match &self.description {
            Some(some) => Some(&some),
            None => None,
        }
    }
    pub fn encoding(&self) -> Option<&MessageEncoding> {
        self.encoding.as_ref()
    }
    pub fn signals(&self) -> &Vec<SignalRef> {
        &self.signals
    }
}

impl TypeSignalEncoding {
    pub fn new(name: String, ty: TypeRef, signals: Vec<SignalRef>) -> TypeSignalEncoding {
        TypeSignalEncoding { name, ty, signals }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn ty(&self) -> &TypeRef {
        &self.ty
    }
    pub fn signals(&self) -> &Vec<SignalRef> {
        &self.signals
    }
}

impl SignalType {
    pub fn offset(&self) -> f64 {
        match &self {
            SignalType::UnsignedInt { size: _ } => 0.0,
            SignalType::SignedInt { size: _ } => 0.0,
            SignalType::Decimal {
                size: _,
                offset,
                scale: _,
            } => *offset,
        }
    }
    pub fn size(&self) -> u8 {
        match &self {
            SignalType::UnsignedInt { size } => *size,
            SignalType::SignedInt { size } => *size,
            SignalType::Decimal {
                size,
                offset: _,
                scale: _,
            } => *size,
        }
    }
    pub fn scale(&self) -> f64 {
        match &self {
            SignalType::UnsignedInt { size: _ } => 1.0,
            SignalType::SignedInt { size: _ } => 1.0,
            SignalType::Decimal {
                size: _,
                offset: _,
                scale,
            } => *scale,
        }
    }
    pub fn sign(&self) -> SignalSign {
        match &self {
            SignalType::UnsignedInt { size: _ } => SignalSign::Unsigned,
            SignalType::SignedInt { size: _ } => SignalSign::Signed,
            SignalType::Decimal {
                size: _,
                offset: _,
                scale: _,
            } => SignalSign::Unsigned,
        }
    }
}

impl Signal {
    pub fn new(name : &str, description : Option<&str>, ty : SignalType) -> Signal {
        Signal {
            name : name.to_owned(),
            description : description.map(|s| s.to_owned()),
            ty,
            offset : 0,
            value_table : None,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> Option<&str> {
        match &self.description {
            Some(some) => Some(&some),
            None => None,
        }
    }
    pub fn ty(&self) -> &SignalType {
        &self.ty
    }
    pub fn scale(&self) -> f64 {
        self.ty.scale()
    }
    pub fn byte_offset(&self) -> usize {
        self.offset
    }
    pub fn offset(&self) -> f64 {
        self.ty.offset()
    }
    pub fn sign(&self) -> SignalSign {
        self.ty.sign()
    }
    pub fn size(&self) -> u8 {
        self.ty.size()
    }
}

impl Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            MessageId::StandardId(id) => write!(f, "0x{:X} ({id})", id),
            MessageId::ExtendedId(id) => write!(f, "0x{:X}x ({id})", id),
        }
    }
}

impl Display for SignalSign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            SignalSign::Signed => write!(f, "signed"),
            SignalSign::Unsigned => write!(f, "unsigned"),
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s1 = "  ";
        let s2 = format!("{s1}{s1}");
        let s3 = format!("{s2}{s1}");
        let s4 = format!("{s2}{s2}");
        let s5 = format!("{s4}{s1}");
        writeln!(f, "Network:")?;
        writeln!(f, "{s1}baudrate : {}", self.baudrate)?;
        writeln!(f, "{s1}build_time : {}", self.build_time)?;
        writeln!(f, "{s1}types:")?;
        for ty in &self.types {
            let vis = match ty as &Type {
                Type::Primitive(_) => "Global".to_owned(),
                Type::Struct {
                    name: _,
                    description: _,
                    attribs: _,
                    visibility,
                } => format!("{visibility:?}"),
                Type::Enum {
                    name: _,
                    description: _,
                    size: _,
                    entries: _,
                    visibility,
                } => format!("{visibility:?}"),
                Type::Array { len: _, ty: _ } => "Static".to_owned(),
            };
            write!(f, "{s2}{} ({})", ty.name(), vis)?;
            match ty as &Type {
                Type::Primitive(_) => {
                    write!(f, "\n")?;
                }
                Type::Struct {
                    name: _,
                    description: _,
                    attribs,
                    visibility: _,
                } => {
                    writeln!(f, ": (struct)")?;
                    for (attrib_name, attrib_type) in attribs {
                        writeln!(f, "{s3}{} : {}", attrib_name, attrib_type.name())?;
                    }
                }
                Type::Enum {
                    name: _,
                    description: _,
                    size: _,
                    entries,
                    visibility: _,
                } => {
                    writeln!(f, ": (enum)")?;
                    for (entry_name, entry_value) in entries {
                        writeln!(f, "{s3}{} = {}", entry_name, entry_value)?;
                    }
                }
                Type::Array { len: _, ty: _ } => {
                    write!(f, "\n")?;
                }
            }
        }
        writeln!(f, "{s1}messages:")?;
        for message in &self.messages {
            writeln!(f, "{s2}{}:", message.name())?;
            if message.description().is_some() {
                writeln!(f, "{s3}description = {}", message.description().unwrap())?;
            }
            writeln!(f, "{s3}id = {}", message.id())?;
            if message.encoding().is_some() {
                let encodings = message.encoding().unwrap();
                writeln!(f, "{s3}map_to_types:")?;
                for encoding in encodings {
                    write!(f, "{s4}{} : ", encoding.name())?;
                    match &encoding.ty as &Type {
                        Type::Primitive(signal_type) => match signal_type {
                            SignalType::UnsignedInt { size } => write!(f, "u{size}")?,
                            SignalType::SignedInt { size } => {
                                write!(f, "i{size}")?;
                            }
                            SignalType::Decimal {
                                size,
                                offset,
                                scale,
                            } => {
                                write!(f, "d{size}<offset={offset}, scale={scale}>")?;
                            }
                        },
                        Type::Struct {
                            name,
                            description: _,
                            attribs: _,
                            visibility: _,
                        } => {
                            write!(f, "{name} (struct)")?;
                        }
                        Type::Enum {
                            name,
                            description: _,
                            size: _,
                            entries: _,
                            visibility: _,
                        } => {
                            write!(f, "{name} (enum)")?;
                        }
                        Type::Array { len, ty } => {
                            write!(f, "{}[{len}]", ty.name())?;
                        }
                    }
                    writeln!(f)?;
                }
            }
            if !message.signals.is_empty() {
                writeln!(f, "{s3}signals:")?;
                for signal in message.signals() {
                    writeln!(f, "{s4}{}:", signal.name())?;
                    if signal.description().is_some() {
                        writeln!(f, "{s5}description = {}", signal.description().unwrap())?;
                        writeln!(f, "{s5}size = {}", signal.size())?;
                        writeln!(f, "{s5}sign = {}", signal.sign())?;
                        writeln!(f, "{s5}scale = {}", signal.scale())?;
                        writeln!(f, "{s5}offset = {}", signal.offset())?;
                    }
                }
            }
        }
        writeln!(f, "{s1}nodes:")?;
        for node in &self.nodes {
            writeln!(f, "{s2}{}:", node.name)?;
            writeln!(f, "{s3}tx_messages:")?;
            for tx_message in node.tx_messages() {
                writeln!(f, "{s4}{}", tx_message.name())?;
            }
            writeln!(f, "{s3}rx_messages:")?;
            for rx_message in node.rx_messages() {
                writeln!(f, "{s4}{}", rx_message.name())?;
            }
            writeln!(f, "{s3}commands:")?;
            for tx_commands in node.commands() {
                writeln!(f, "{s4}{}", tx_commands.name())?;
            }
            writeln!(f, "{s3}extern_commands:")?;
            for (node_name, rx_commands) in node.extern_commands() {
                writeln!(f, "{s4}{}::{}", node_name, rx_commands.name())?;
            }
            writeln!(f, "{s3}object_entries:")?;
            for entry in node.object_entries() {
                writeln!(f, "{s4}{} : {}", entry.name(), entry.ty().name())?;
            }
            writeln!(f, "{s3}tx_streams:")?;
            for stream in node.tx_streams() {
                writeln!(f, "{s4}{} [{}]", stream.name(), stream.message().name())?;
                for oe in stream.mapping() {
                    let oe_name = match oe {
                        Some(oe) => oe.name(),
                        None => "None",
                    };
                    let oe_ty = match oe {
                        Some(oe) => oe.ty().name(),
                        None => "?".to_owned(),
                    };
                    writeln!(f, "{s5}<-{} : {}", oe_name, oe_ty)?;
                }
            }
            writeln!(f, "{s3}rx_streams:")?;
            for stream in node.rx_streams() {
                writeln!(f, "{s4}{} [{}]", stream.name(), stream.message().name())?;
                for oe in stream.mapping() {
                    let oe_name = match oe {
                        Some(oe) => oe.name(),
                        None => "None",
                    };
                    let oe_ty = match oe {
                        Some(oe) => oe.ty().name(),
                        None => "?".to_owned(),
                    };
                    writeln!(f, "{s5}->{} : {}", oe_name, oe_ty)?;
                }
            }
            writeln!(f, "{s3}types:")?;
            for ty in node.types() {
                writeln!(f, "{s4}{}", ty.name())?;
            }
        }
        Ok(())
    }
}

// **************************************************************************
// **************************************************************************
// **************************************************************************
//                             NETWORK-BUILDER
// **************************************************************************
// **************************************************************************
// **************************************************************************

type BuilderRef<T> = Rc<RefCell<T>>;

fn make_builder_ref<T>(value: T) -> BuilderRef<T> {
    Rc::new(RefCell::new(value))
}

#[derive(Debug, Clone)]
pub struct NetworkBuilder(BuilderRef<NetworkData>);

#[derive(Debug)]
pub struct NetworkData {
    baudrate: Option<u32>,
    messages: BuilderRef<Vec<MessageBuilder>>,
    types: BuilderRef<Vec<TypeBuilder>>,
    nodes: BuilderRef<Vec<NodeBuilder>>,
}

#[derive(Debug)]
pub enum MessagePriority {
    Default,
    Realtime,
    High,
    Normal,
    Low,
    SuperLow,
}

#[derive(Debug)]
enum MessageIdTemplate {
    StdId(u32),
    ExtId(u32),
    AnyStd(MessagePriority),
    AnyExt(MessagePriority),
    AnyAny(MessagePriority),
}

#[derive(Clone, Debug)]
pub struct MessageBuilder(BuilderRef<MessageData>);

#[derive(Debug)]
pub struct MessageData {
    name: String,
    description: Option<String>,
    id: MessageIdTemplate,
    format: MessageFormat,
    network_builder: NetworkBuilder,
    visibility: Visibility,
}

#[derive(Debug)]
pub enum MessageFormat {
    Signals(MessageSignalFormatBuilder),
    Types(MessageTypeFormatBuilder),
    Empty,
}

#[derive(Clone, Debug)]
pub struct MessageSignalFormatBuilder(BuilderRef<MessageSignalFormatData>);
#[derive(Debug)]
pub struct MessageSignalFormatData(Vec<Signal>);
#[derive(Clone, Debug)]
pub struct MessageTypeFormatBuilder(BuilderRef<MessageTypeFormatData>);
#[derive(Debug)]
pub struct MessageTypeFormatData(Vec<(String, String)>);

#[derive(Clone, Debug)]
pub struct EnumBuilder(BuilderRef<EnumData>);
#[derive(Debug)]
pub struct EnumData {
    name: String,
    description: Option<String>,
    entries: Vec<(String, Option<u64>)>,
    visibility: Visibility,
}

#[derive(Debug, Clone)]
pub struct StructBuilder(BuilderRef<StructData>);
#[derive(Debug)]
pub struct StructData {
    name: String,
    description: Option<String>,
    attributes: Vec<(String, String)>,
    visibility: Visibility,
}

#[derive(Debug, Clone)]
pub enum TypeBuilder {
    Enum(EnumBuilder),
    Struct(StructBuilder),
}

#[derive(Debug, Clone)]
pub struct NodeBuilder(BuilderRef<NodeData>);
#[derive(Debug)]
pub struct NodeData {
    name: String,
    description: Option<String>,
    commands: Vec<CommandBuilder>,
    extern_commands: Vec<CommandBuilder>,
    get_req_message: MessageBuilder,
    get_resp_message: MessageBuilder,
    set_req_message: MessageBuilder,
    set_resp_message: MessageBuilder,
    network_builder: NetworkBuilder,
    rx_messages: Vec<MessageBuilder>,
    tx_messages: Vec<MessageBuilder>,
    object_entries: Vec<ObjectEntryBuilder>,
    tx_streams: Vec<StreamBuilder>,
    rx_streams: Vec<ReceiveStreamBuilder>,
}

#[derive(Debug, Clone)]
pub struct ObjectEntryBuilder(BuilderRef<ObjectEntryData>);
#[derive(Debug)]
pub struct ObjectEntryData {
    name: String,
    description: Option<String>,
    unit: Option<String>,
    ty: String,
    access: ObjectEntryAccess,
    visibility: Visibility,
}

#[derive(Debug, Clone)]
pub struct CommandBuilder(BuilderRef<CommandData>);
#[derive(Debug)]
pub struct CommandData {
    tx_node: NodeBuilder,
    name: String,
    description: Option<String>,
    call_message: MessageBuilder,
    call_message_format: MessageTypeFormatBuilder,
    resp_message: MessageBuilder,
    visibility: Visibility,
}

#[derive(Debug, Clone)]
pub struct StreamBuilder(BuilderRef<StreamData>);
#[derive(Debug)]
pub struct StreamData {
    name: String,
    description: Option<String>,
    message: MessageBuilder,
    format: MessageTypeFormatBuilder,
    tx_node: NodeBuilder,
    object_entries: Vec<ObjectEntryBuilder>,
    visbility: Visibility,
}

#[derive(Debug, Clone)]
pub struct ReceiveStreamBuilder(BuilderRef<ReceiveStreamData>);
#[derive(Debug)]
pub struct ReceiveStreamData {
    stream_builder: StreamBuilder,
    rx_node: NodeBuilder,
    object_entries: Vec<(usize, ObjectEntryBuilder)>,
    visibility: Visibility,
}

impl NetworkBuilder {
    pub fn new() -> NetworkBuilder {
        let network_builder = NetworkBuilder(make_builder_ref(NetworkData {
            baudrate: None,
            messages: make_builder_ref(vec![]),
            types: make_builder_ref(vec![]),
            nodes: make_builder_ref(vec![]),
        }));

        // Setup header types.
        let command_resp_type = network_builder.define_enum("command_resp_erno");
        command_resp_type.hide();
        command_resp_type.add_entry("Ok", Some(0)).unwrap();
        command_resp_type.add_entry("Error", Some(1)).unwrap();

        let command_resp_header_builder = network_builder.define_struct("command_resp_header");
        command_resp_header_builder.hide();
        command_resp_header_builder
            .add_attribute("erno", "command_resp_erno")
            .unwrap();

        let command_req_header_builder = network_builder.define_struct("command_req_header");
        command_req_header_builder.hide();

        let set_resp_erno = network_builder.define_enum("set_resp_erno");
        set_resp_erno.hide();
        set_resp_erno.add_entry("Ok", Some(0)).unwrap();
        set_resp_erno.add_entry("Error", Some(1)).unwrap();

        let get_req_header = network_builder.define_struct("get_req_header");
        get_req_header.hide();
        get_req_header
            .add_attribute("object_entry_index", "u16")
            .unwrap();

        let get_resp_header = network_builder.define_struct("get_resp_header");
        get_resp_header.hide();
        get_resp_header
            .add_attribute("object_entry_index", "u16")
            .unwrap();

        let set_req_header = network_builder.define_struct("set_req_header");
        set_req_header.hide();
        set_req_header
            .add_attribute("object_entry_index", "u16")
            .unwrap();

        let set_resp_header = network_builder.define_struct("set_resp_header");
        set_resp_header.hide();
        set_resp_header
            .add_attribute("object_entry_index", "u16")
            .unwrap();
        set_resp_header
            .add_attribute("erno", "command_resp_erno")
            .unwrap();

        network_builder
    }
    pub fn set_baudrate(&self, baudrate: u32) {
        let mut network_data = self.0.borrow_mut();
        network_data.baudrate = Some(baudrate);
    }

    pub fn create_message(&self, name: &str) -> MessageBuilder {
        let network_data = self.0.borrow();
        let message_builder = MessageBuilder::new(name, &self);
        network_data
            .messages
            .borrow_mut()
            .push(message_builder.clone());
        message_builder
    }
    pub fn define_enum(&self, name: &str) -> EnumBuilder {
        let network_data = self.0.borrow();
        let type_builder = EnumBuilder::new(name);
        network_data
            .types
            .borrow_mut()
            .push(TypeBuilder::Enum(type_builder.clone()));
        type_builder
    }
    pub fn define_struct(&self, name: &str) -> StructBuilder {
        let network_data = self.0.borrow();
        let type_builder = StructBuilder::new(name);
        network_data
            .types
            .borrow_mut()
            .push(TypeBuilder::Struct(type_builder.clone()));
        type_builder
    }
    pub fn create_node(&self, name: &str) -> NodeBuilder {
        let network_data = self.0.borrow();
        // check if node already exists.
        let existing_node = network_data
            .nodes
            .borrow()
            .iter()
            .find(|n| n.0.borrow().name == name)
            .map(NodeBuilder::to_owned);
        let Some(node) = existing_node else {
            let node_builder = NodeBuilder::new(name, &self);
            network_data.nodes.borrow_mut().push(node_builder.clone());
            return node_builder;
        };
        node
    }
}

impl MessagePriority {
    fn min_id(&self) -> u32 {
        match &self {
            MessagePriority::Default => 800,
            MessagePriority::Realtime => 0,
            MessagePriority::High => 400,
            MessagePriority::Normal => 800,
            MessagePriority::Low => 1200,
            MessagePriority::SuperLow => 1600,
        }
    }
}

impl StreamBuilder {
    pub fn new(name: &str, node_builder: NodeBuilder) -> StreamBuilder {
        let node_data = node_builder.0.borrow();
        let message = node_data
            .network_builder
            .create_message(&format!("{}_stream_{name}", node_builder.0.borrow().name));
        drop(node_data);
        node_builder.add_tx_message(&message);
        message.hide();
        message.set_any_std_id(MessagePriority::Normal);
        let format = message.make_type_format();

        StreamBuilder(make_builder_ref(StreamData {
            name: name.to_owned(),
            description: None,
            message,
            format,
            tx_node: node_builder,
            object_entries: vec![],
            visbility: Visibility::Global,
        }))
    }
    pub fn hide(&self) {
        let mut stream_data = self.0.borrow_mut();
        stream_data.visbility = Visibility::Static;
    }
    pub fn add_description(&self, description: &str) {
        let mut stream_data = self.0.borrow_mut();
        stream_data.description = Some(description.to_owned());
    }
    pub fn add_entry(&self, name: &str) {
        let mut stream_data = self.0.borrow_mut();
        let node = stream_data.tx_node.clone();
        let node_data = node.0.borrow();
        let oe = node_data
            .object_entries
            .iter()
            .find(|oe| oe.0.borrow().name == name)
            .cloned()
            .unwrap_or_else(|| node.create_object_entry(name, "u1"));
        stream_data.object_entries.push(oe.clone());
        let oe_data = oe.0.borrow();
        stream_data.format.add_type(&oe_data.ty, &oe_data.name);
    }
}

impl NodeBuilder {
    pub fn new(name: &str, network_builder: &NetworkBuilder) -> NodeBuilder {
        let get_req_message = network_builder.create_message(&format!("{name}_get_req"));
        get_req_message.hide();
        get_req_message.set_any_std_id(MessagePriority::Low);
        get_req_message.add_description(&format!("get request message for node : {name}"));

        let get_resp_message = network_builder.create_message(&format!("{name}_get_resp"));
        get_resp_message.hide();
        get_resp_message.set_any_std_id(MessagePriority::Low);
        get_resp_message.add_description(&format!("get response message for node : {name}"));

        let set_req_message = network_builder.create_message(&format!("{name}_set_req"));
        set_req_message.hide();
        set_req_message.set_any_std_id(MessagePriority::Low);
        set_req_message.add_description(&format!("set request message for node : {name}"));

        let set_resp_message = network_builder.create_message(&format!("{name}_set_resp"));
        set_resp_message.hide();
        set_resp_message.add_description(&format!("set response message for node : {name}"));
        set_resp_message.set_any_std_id(MessagePriority::Low);

        let node_builder = NodeBuilder(make_builder_ref(NodeData {
            name: name.to_owned(),
            description: None,
            network_builder: network_builder.clone(),
            get_req_message: get_req_message.clone(),
            get_resp_message: get_resp_message.clone(),
            set_req_message: set_req_message.clone(),
            set_resp_message: set_resp_message.clone(),
            commands: vec![],
            extern_commands: vec![],
            tx_messages: vec![],
            rx_messages: vec![],
            object_entries: vec![],
            tx_streams: vec![],
            rx_streams: vec![],
        }));
        node_builder.add_rx_message(&get_req_message);
        node_builder.add_tx_message(&get_resp_message);
        node_builder.add_rx_message(&set_req_message);
        node_builder.add_tx_message(&set_resp_message);

        node_builder
    }
    pub fn add_description(&self, description: &str) {
        let mut node_data = self.0.borrow_mut();
        node_data.description = Some(description.to_owned());
    }
    pub fn add_tx_message(&self, message_builder: &MessageBuilder) {
        let mut node_data = self.0.borrow_mut();
        node_data.tx_messages.push(message_builder.clone());
    }
    pub fn add_rx_message(&self, message_builder: &MessageBuilder) {
        let mut node_data = self.0.borrow_mut();
        node_data.rx_messages.push(message_builder.clone());
    }
    pub fn create_command(&self, name: &str) -> CommandBuilder {
        let command_builder = CommandBuilder::new(name, &self);
        let mut node_data = self.0.borrow_mut();
        node_data.commands.push(command_builder.clone());
        node_data
            .rx_messages
            .push(command_builder.0.borrow().call_message.clone());
        node_data
            .tx_messages
            .push(command_builder.0.borrow().resp_message.clone());
        command_builder
    }
    pub fn add_extern_command(&self, message_builder: &CommandBuilder) {
        let mut node_data = self.0.borrow_mut();
        node_data.extern_commands.push(message_builder.clone());
        node_data
            .rx_messages
            .push(message_builder.0.borrow().resp_message.clone());
        node_data
            .tx_messages
            .push(message_builder.0.borrow().call_message.clone());
    }
    pub fn create_object_entry(&self, name: &str, ty: &str) -> ObjectEntryBuilder {
        let object_entry_builder = ObjectEntryBuilder::new(name, ty);
        let mut node_data = self.0.borrow_mut();
        node_data.object_entries.push(object_entry_builder.clone());
        object_entry_builder
    }
    pub fn create_stream(&self, name: &str) -> StreamBuilder {
        let stream_builder = StreamBuilder::new(name, self.clone());
        let mut node_data = self.0.borrow_mut();
        node_data.tx_streams.push(stream_builder.clone());
        stream_builder
    }

    pub fn receive_stream(&self, tx_node_name: &str, name: &str) -> ReceiveStreamBuilder {
        let node_data = self.0.borrow();
        if tx_node_name == node_data.name {
            panic!("can't receive local stream");
        }
        let network_builder = &node_data.network_builder;
        let tx_node_opt = network_builder
            .0
            .borrow()
            .nodes
            .borrow()
            .iter()
            .find(|n| n.0.borrow().name == tx_node_name)
            .cloned();
        let tx_node = match tx_node_opt {
            Some(tx_node) => tx_node,
            None => network_builder.create_node(tx_node_name),
        };
        let tx_node_data = tx_node.0.borrow();
        let tx_stream_opt = tx_node_data
            .tx_streams
            .iter()
            .find(|s| s.0.borrow().name == name)
            .cloned();
        let tx_stream = match tx_stream_opt {
            Some(tx_stream) => tx_stream,
            None => tx_node.create_stream(name),
        };
        drop(node_data);

        let tx_stream_data = tx_stream.0.borrow();
        self.add_rx_message(&tx_stream_data.message);
        drop(tx_stream_data);


        let mut node_data = self.0.borrow_mut();
        let rx_stream_builder = ReceiveStreamBuilder::new(tx_stream, self.clone());
        node_data.rx_streams.push(rx_stream_builder.clone());


        rx_stream_builder
    }
}

impl ReceiveStreamBuilder {
    pub fn new(stream_builder: StreamBuilder, rx_node: NodeBuilder) -> ReceiveStreamBuilder {
        ReceiveStreamBuilder(make_builder_ref(ReceiveStreamData {
            stream_builder,
            rx_node,
            object_entries: vec![],
            visibility: Visibility::Global,
        }))
    }
    pub fn hide(&self) {
        let mut rx_stream_data = self.0.borrow_mut();
        rx_stream_data.visibility = Visibility::Static;
    }
    pub fn map(&self, from: &str, to: &str) {
        // resolve from
        let mut rx_stream_data = self.0.borrow_mut();
        let tx_stream_builder = rx_stream_data.stream_builder.clone();
        let tx_stream_data = tx_stream_builder.0.borrow();
        let opt_pos = tx_stream_data
            .object_entries
            .iter()
            .position(|oe| oe.0.borrow().name == from);
        let Some(pos) = opt_pos else {
            panic!("invalid rx stream mapping");
        };
        // resolve to
        let oe_opt = rx_stream_data
            .rx_node
            .0
            .borrow()
            .object_entries
            .iter()
            .find(|oe| oe.0.borrow().name == to)
            .cloned();
        let oe = match oe_opt {
            Some(oe) => {
                assert_eq!(
                    oe.0.borrow().ty,
                    tx_stream_data.object_entries[pos].0.borrow().ty
                );
                oe
            }
            None => {
                let tx_oe = tx_stream_data.object_entries[pos].0.borrow();
                rx_stream_data.rx_node.create_object_entry(to, &tx_oe.ty)
            }
        };
        rx_stream_data.object_entries.push((pos, oe));
    }
}

impl ObjectEntryBuilder {
    pub fn new(name: &str, ty: &str) -> ObjectEntryBuilder {
        ObjectEntryBuilder(make_builder_ref(ObjectEntryData {
            name: name.to_owned(),
            ty: ty.to_owned(),
            description: None,
            unit: None,
            access: ObjectEntryAccess::Global,
            visibility: Visibility::Global,
        }))
    }
    pub fn hide(&self) {
        let mut data = self.0.borrow_mut();
        data.visibility = Visibility::Static;
    }
    pub fn add_description(&self, description: &str) {
        let mut data = self.0.borrow_mut();
        data.description = Some(description.to_owned());
    }
    pub fn set_access(&self, access: ObjectEntryAccess) {
        let mut data = self.0.borrow_mut();
        data.access = access;
    }
    pub fn add_unit(&self, unit: &str) {
        let mut data = self.0.borrow_mut();
        data.unit = Some(unit.to_owned());
    }
}

impl CommandBuilder {
    pub fn new(name: &str, tx_node_builder: &NodeBuilder) -> CommandBuilder {
        let node_data = tx_node_builder.0.borrow();
        let network_builder = &node_data.network_builder;
        let tx_message =
            network_builder.create_message(&format!("{}_{}_command_req", node_data.name, name));
        tx_message.hide();
        tx_message.set_any_std_id(MessagePriority::High);
        let tx_message_format = tx_message.make_type_format();
        tx_message_format.add_type("command_req_header", "header");

        let rx_message =
            network_builder.create_message(&format!("{}_{}_command_resp", node_data.name, name));
        rx_message.hide();
        rx_message.set_any_std_id(MessagePriority::Low);
        let rx_message_format = rx_message.make_type_format();
        rx_message_format.add_type("command_resp_header", "header");

        CommandBuilder(make_builder_ref(CommandData {
            name: name.to_owned(),
            description: None,
            call_message: tx_message,
            call_message_format: tx_message_format,
            resp_message: rx_message,
            tx_node: tx_node_builder.clone(),
            visibility: Visibility::Global,
        }))
    }
    pub fn hide(&self) {
        let mut command_data = self.0.borrow_mut();
        command_data.visibility = Visibility::Static;
    }
    pub fn set_priority(&self, priority: MessagePriority) {
        let command_data = self.0.borrow();
        command_data.call_message.set_any_std_id(priority);
    }
    pub fn add_description(&self, name: &str) {
        let mut command_data = self.0.borrow_mut();
        command_data.description = Some(name.to_owned());
    }
    pub fn add_argument(&self, name: &str, ty: &str) {
        let command_data = self.0.borrow();
        command_data.call_message_format.add_type(ty, name);
    }
    pub fn add_callee(&self, name: &str) {
        let network_builder = self.0.borrow().tx_node.0.borrow().network_builder.clone();
        let callee = network_builder.create_node(name);
        callee.add_extern_command(&self);
    }
}

impl MessageBuilder {
    pub fn new(name: &str, network_builder: &NetworkBuilder) -> MessageBuilder {
        MessageBuilder(make_builder_ref(MessageData {
            name: name.to_owned(),
            description: None,
            id: MessageIdTemplate::AnyAny(MessagePriority::Default),
            format: MessageFormat::Empty,
            network_builder: network_builder.clone(),
            visibility: Visibility::Global,
        }))
    }
    pub fn hide(&self) {
        let mut message_data = self.0.borrow_mut();
        message_data.visibility = Visibility::Static;
    }
    pub fn set_std_id(&self, id: u32) {
        let mut message_data = self.0.borrow_mut();
        message_data.id = MessageIdTemplate::StdId(id);
    }
    pub fn set_ext_id(&self, id: u32) {
        let mut message_data = self.0.borrow_mut();
        message_data.id = MessageIdTemplate::ExtId(id);
    }
    pub fn set_any_std_id(&self, priority: MessagePriority) {
        let mut message_data = self.0.borrow_mut();
        message_data.id = MessageIdTemplate::AnyStd(priority);
    }
    pub fn set_any_ext_id(&self, priority: MessagePriority) {
        let mut message_data = self.0.borrow_mut();
        message_data.id = MessageIdTemplate::AnyExt(priority);
    }
    pub fn make_signal_format(&self) -> MessageSignalFormatBuilder {
        let mut message_data = self.0.borrow_mut();
        let signal_format_builder = MessageSignalFormatBuilder::new();
        message_data.format = MessageFormat::Signals(signal_format_builder.clone());
        signal_format_builder
    }
    pub fn make_type_format(&self) -> MessageTypeFormatBuilder {
        let mut message_data = self.0.borrow_mut();
        let type_format_builder = MessageTypeFormatBuilder::new();
        message_data.format = MessageFormat::Types(type_format_builder.clone());
        type_format_builder
    }
    pub fn add_description(&self, name: &str) {
        let mut message_data = self.0.borrow_mut();
        message_data.description = Some(name.to_owned());
    }
    pub fn add_transmitter(&self, name: &str) {
        // check if node with {name} exists.
        let message_data = self.0.borrow();
        let mut node_named: Option<NodeBuilder> = None;
        for node in message_data
            .network_builder
            .0
            .borrow()
            .nodes
            .borrow()
            .iter()
        {
            if node.0.borrow().name == name {
                node_named = Some(node.clone());
            }
        }
        let node = match node_named {
            Some(node) => node,
            None => message_data.network_builder.create_node(name),
        };
        node.add_tx_message(&self);
    }
    pub fn add_receiver(&self, name: &str) {
        // check if node with {name} exists.
        let message_data = self.0.borrow();
        let mut node_named: Option<NodeBuilder> = None;
        for node in message_data
            .network_builder
            .0
            .borrow()
            .nodes
            .borrow()
            .iter()
        {
            if node.0.borrow().name == name {
                node_named = Some(node.clone());
            }
        }
        let node = match node_named {
            Some(node) => node,
            None => message_data.network_builder.create_node(name),
        };
        node.add_rx_message(&self);
    }
}

impl MessageSignalFormatBuilder {
    pub fn new() -> MessageSignalFormatBuilder {
        MessageSignalFormatBuilder(make_builder_ref(MessageSignalFormatData(vec![])))
    }
    pub fn add_signal(&self, signal: Signal) -> errors::Result<()> {
        let mut builder_data = self.0.borrow_mut();
        if builder_data.0.iter().any(|s| s.name == signal.name) {
            return Err(errors::ConfigError::DuplicatedSignal(format!(
                "Dupplicated signal name in message: {}",
                signal.name
            )));
        }
        builder_data.0.push(signal);
        Ok(())
    }
}
impl MessageTypeFormatBuilder {
    pub fn new() -> MessageTypeFormatBuilder {
        MessageTypeFormatBuilder(make_builder_ref(MessageTypeFormatData(vec![])))
    }
    pub fn add_type(&self, type_name: &str, value_name: &str) {
        let mut builder_data = self.0.borrow_mut();
        builder_data
            .0
            .push((type_name.to_owned(), value_name.to_owned()));
    }
}

impl EnumBuilder {
    fn new(name: &str) -> EnumBuilder {
        EnumBuilder(make_builder_ref(EnumData {
            name: name.to_owned(),
            description: None,
            entries: vec![],
            visibility: Visibility::Global,
        }))
    }
    pub fn add_description(&self, description: &str) {
        let mut enum_data = self.0.borrow_mut();
        enum_data.description = Some(description.to_owned());
    }
    pub fn add_entry(&self, name: &str, value: Option<u64>) -> errors::Result<()> {
        let mut enum_data = self.0.borrow_mut();
        if enum_data.entries.iter().any(|a| a.0 == name) {
            return Err(errors::ConfigError::DuplicatedEnumEntry(name.to_owned()));
        }
        enum_data.entries.push((name.to_owned(), value));
        Ok(())
    }
    pub fn hide(&self) {
        let mut enum_data = self.0.borrow_mut();
        enum_data.visibility = Visibility::Static;
    }
}

impl StructBuilder {
    fn new(name: &str) -> StructBuilder {
        StructBuilder(make_builder_ref(StructData {
            name: name.to_owned(),
            description: None,
            attributes: vec![],
            visibility: Visibility::Global,
        }))
    }
    pub fn add_description(&self, description: &str) {
        let mut struct_data = self.0.borrow_mut();
        struct_data.description = Some(description.to_owned());
    }
    pub fn add_attribute(&self, name: &str, ty: &str) -> errors::Result<()> {
        let mut struct_data = self.0.borrow_mut();
        if struct_data.attributes.iter().any(|a| a.0 == name) {
            return Err(errors::ConfigError::DuplicatedStructAttribute(
                name.to_owned(),
            ));
        }
        struct_data
            .attributes
            .push((name.to_owned(), ty.to_owned()));
        Ok(())
    }
    pub fn hide(&self) {
        let mut struct_data = self.0.borrow_mut();
        struct_data.visibility = Visibility::Static;
    }
}

impl TypeBuilder {
    fn name(&self) -> String {
        match &self {
            TypeBuilder::Enum(enum_builder) => enum_builder.0.borrow().name.clone(),
            TypeBuilder::Struct(struct_builder) => struct_builder.0.borrow().name.clone(),
        }
    }
}

impl NetworkBuilder {
    fn resolve_type(
        defined_types: &Vec<TypeRef>,
        type_name: &str,
    ) -> errors::Result<ConfigRef<Type>> {
        let int_regex = regex::Regex::new(r#"^i(?<size>[0-9]{1,2})$"#).unwrap();
        match int_regex.captures(type_name) {
            Some(cap) => {
                let size = &cap["size"];
                let size = size.parse::<u8>().unwrap();
                if size > 0 && size <= 64 {
                    return Ok(make_config_ref(Type::Primitive(SignalType::SignedInt {
                        size,
                    })));
                }
            }
            None => (),
        }
        let uint_regex = regex::Regex::new(r#"^u(?<size>[0-9]{1,2})$"#).unwrap();
        match uint_regex.captures(type_name) {
            Some(cap) => {
                let size = &cap["size"];
                let size = size.parse::<u8>().unwrap();
                if size > 0 && size <= 64 {
                    return Ok(make_config_ref(Type::Primitive(SignalType::UnsignedInt {
                        size,
                    })));
                }
            }
            None => (),
        }
        let dec_regex = regex::Regex::new(r"^d(?<size>[0-9]{1,2})<(?<min>[+-]?([0-9]*[.])?[0-9]+)\.\.(?<max>[+-]?([0-9]*[.])?[0-9]+)>$").unwrap();
        match dec_regex.captures(type_name) {
            Some(cap) => {
                let size = &cap["size"];
                let size = size.parse::<u8>().unwrap();
                let min = &cap["min"];
                let min = min.parse::<f64>().unwrap();
                let max = &cap["max"];
                let max = max.parse::<f64>().unwrap();
                if min >= max {
                    return Err(errors::ConfigError::InvalidRange(
                        "invalid decimal range min has to be less than max".to_owned(),
                    ));
                }
                let range = max - min;
                let scale = range / ((0xFFFFFFFFFFFFFFFF as u64 >> (64 - size)) as f64);
                let offset = min;
                if size <= 64 {
                    return Ok(make_config_ref(Type::Primitive(SignalType::Decimal {
                        size,
                        offset,
                        scale,
                    })));
                }
            }
            None => (),
        }
        let array_regex =
                regex::Regex::new(r#"^(?<type>[a-zA-Z][a-zA-Z0-9]*(<[+-]?([0-9]*[.])?[0-9]+\.\.[+-]?([0-9]*[.])?[0-9]+>)?)\[(?<len>[0-9]+)\]$"#).unwrap();
        match array_regex.captures(type_name) {
            Some(cap) => {
                let len = &cap["len"];
                let len = len.parse::<usize>().unwrap();
                let ty = &cap["type"];
                let inner_type = Self::resolve_type(defined_types, ty)?;
                return Ok(make_config_ref(Type::Array {
                    len,
                    ty: inner_type,
                }));
            }
            None => (),
        }
        for ty in defined_types {
            match ty as &Type {
                Type::Struct {
                    name,
                    description: _,
                    attribs: _,
                    visibility: _,
                } if name == type_name => return Ok(ty.clone()),
                Type::Enum {
                    name,
                    description: _,
                    size: _,
                    entries: _,
                    visibility: _,
                } if name == type_name => return Ok(ty.clone()),
                _ => (),
            }
        }
        return Err(errors::ConfigError::InvalidType(format!(
            "failed to resolve type : {type_name:?}"
        )));
    }

    fn type_to_signals(
        ty: TypeRef,
        message_name: &str,
        value_name: &str,
        type_name: &str,
        offset: &mut usize,
    ) -> Vec<SignalRef> {
        let mut type_signals = vec![];
        match &ty as &Type {
            Type::Primitive(signal_type) => {
                type_signals.push(make_config_ref(Signal {
                    name: format!("{}_{}_field", message_name, value_name),
                    description: Some(format!(
                        "{} of type {} in message {}",
                        value_name, type_name, message_name
                    )),
                    ty: signal_type.clone(),
                    value_table: None,
                    offset: *offset,
                }));
                *offset += signal_type.size() as usize;
            }
            Type::Struct {
                name,
                description: _,
                attribs,
                visibility: _,
            } => {
                for (attrib_name, attrib_type) in attribs {
                    let attrib_signals = Self::type_to_signals(
                        attrib_type.clone(),
                        message_name,
                        value_name,
                        name,
                        &mut 0,
                    );
                    for signal in attrib_signals {
                        type_signals.push(make_config_ref(Signal {
                            name: format!(
                                "{}_{}_{}_{}",
                                message_name, value_name, name, signal.name
                            ),
                            description: Some(format!(
                                "for message {} argument {} attribute {} of struct {}",
                                message_name, value_name, attrib_name, name
                            )),
                            offset: *offset,
                            ty: signal.ty.clone(),
                            value_table: signal.value_table.clone(),
                        }));
                        *offset += signal.ty.size() as usize;
                    }
                }
            }
            Type::Enum {
                name,
                size,
                description: _,
                entries,
                visibility: _,
            } => {
                let value_table = make_config_ref(ValueTable(entries.clone()));
                type_signals.push(make_config_ref(Signal {
                    name: format!("{}_{}value", message_name, value_name),
                    description: Some(format!(
                        "{} of type {} in message {}",
                        value_name, name, message_name
                    )),
                    ty: SignalType::UnsignedInt { size: *size },
                    value_table: Some(value_table),
                    offset: 0,
                }));
            }
            Type::Array { len, ty } => {
                let inner_signals =
                    Self::type_to_signals(ty.clone(), message_name, value_name, type_name, &mut 0);
                for i in 0..*len {
                    for inner_signal in &inner_signals {
                        type_signals.push(make_config_ref(Signal {
                            name: format!("{}_{}_{}_field", message_name, value_name, i),
                            description: Some(format!(
                                "{} of type {} at index {} in message {}",
                                value_name, type_name, i, message_name
                            )),
                            ty: inner_signal.ty.clone(),
                            value_table: inner_signal.value_table.clone(),
                            offset: *offset,
                        }));
                        *offset += inner_signal.ty().size() as usize;
                    }
                }
            }
        }

        type_signals
    }

    fn topo_sort_types(types: &Vec<TypeRef>) -> Vec<TypeRef> {
        let n = types.len();
        struct Node {
            index: usize,
            adj_list: Vec<usize>,
        }
        let mut nodes: Vec<Node> = vec![];
        for i in 0..n {
            let ty = &types[i];
            let mut adj_list = vec![];
            match ty as &Type {
                Type::Struct {
                    name: _,
                    description: _,
                    attribs,
                    visibility: _,
                } => {
                    for (_, attrib_type) in attribs {
                        match types.iter().position(|t| t == attrib_type) {
                            Some(adj) => adj_list.push(adj),
                            None => (),
                        }
                    }
                }
                Type::Array { len: _, ty } => match types.iter().position(|t| t == ty) {
                    Some(adj) => adj_list.push(adj),
                    None => (),
                },
                _ => (),
            }
            nodes.push(Node { index: i, adj_list })
        }
        let mut stack: Vec<usize> = vec![];
        let mut visited = vec![false; nodes.len()];
        fn topo_sort_rec(
            nodes: &Vec<Node>,
            visited: &mut Vec<bool>,
            current: usize,
            stack: &mut Vec<usize>,
        ) {
            visited[current] = true;
            for adj_index in &nodes[current].adj_list {
                if !visited[*adj_index] {
                    topo_sort_rec(nodes, visited, *adj_index, stack);
                }
            }
            stack.push(current);
        }
        for i in 0..n {
            if !visited[i] {
                topo_sort_rec(&nodes, &mut visited, i, &mut stack);
            }
        }

        stack.iter().map(|index| types[*index].clone()).collect()
    }

    fn topo_sort_type_builders(
        type_builders: &Vec<TypeBuilder>,
    ) -> errors::Result<Vec<TypeBuilder>> {
        // TODO check for cycles in the graph
        // number of nodes
        let n = type_builders.len();

        #[derive(Debug)]
        struct Node {
            index: usize,
            adj_list: Vec<usize>,
        }

        let mut nodes: Vec<Node> = vec![];
        for node_index in 0..n {
            let adj_list = match &type_builders[node_index] {
                TypeBuilder::Enum(_) => vec![],
                TypeBuilder::Struct(struct_builder) => {
                    let struct_data = struct_builder.0.borrow();
                    let mut dependencies = vec![];
                    for (_, attrib_type_name) in &struct_data.attributes {
                        //check if type is a inplace definition (u?, i?, d?)
                        let is_inplace = Self::resolve_type(&vec![], attrib_type_name).is_ok();
                        if is_inplace {
                            continue;
                        }
                        let opt = type_builders
                            .iter()
                            .position(|builder| &builder.name() == attrib_type_name);
                        match opt {
                            Some(adj_index) => {
                                dependencies.push(adj_index);
                            }
                            None => {
                                return Err(errors::ConfigError::UndefinedType(format!(
                                    "{attrib_type_name}"
                                )))
                            }
                        }
                    }
                    dependencies
                }
            };
            nodes.push(Node {
                index: node_index,
                adj_list,
            });
        }

        let mut stack: Vec<usize> = vec![];
        let mut visited = vec![false; nodes.len()];
        fn topo_sort_rec(
            nodes: &Vec<Node>,
            visited: &mut Vec<bool>,
            current: usize,
            stack: &mut Vec<usize>,
        ) {
            visited[current] = true;
            for adj_index in &nodes[current].adj_list {
                if !visited[*adj_index] {
                    topo_sort_rec(nodes, visited, *adj_index, stack);
                }
            }
            stack.push(current);
        }
        for i in 0..n {
            if !visited[i] {
                topo_sort_rec(&nodes, &mut visited, i, &mut stack);
            }
        }
        Ok(stack
            .iter()
            .map(|index| type_builders[*index].clone())
            .collect())
    }

    fn resolve_ids(messages: &mut Vec<MessageBuilder>) -> errors::Result<()> {
        for i in 0..messages.len() {
            let mut message_data = messages[i].0.borrow_mut();
            match &message_data.id {
                MessageIdTemplate::StdId(_) => (),
                MessageIdTemplate::ExtId(_) => (),
                MessageIdTemplate::AnyStd(priority) => {
                    let mut id = priority.min_id();
                    loop {
                        for j in 0..messages.len() {
                            if i == j {
                                continue;
                            }
                            let other = messages[j].0.borrow();
                            match other.id {
                                MessageIdTemplate::StdId(other_id) if other_id == id => {
                                    id += 1;
                                    continue;
                                }
                                _ => (),
                            }
                        }
                        if id > 2047 {
                            return Err(errors::ConfigError::FailedToResolveId);
                        }
                        break;
                    }
                    message_data.id = MessageIdTemplate::StdId(id);
                }
                MessageIdTemplate::AnyExt(priority) => {
                    let mut id = priority.min_id();
                    loop {
                        for j in 0..messages.len() {
                            if i == j {
                                continue;
                            }
                            let other = messages[j].0.borrow();
                            match other.id {
                                MessageIdTemplate::ExtId(other_id) if other_id == id => {
                                    id += 1;
                                    continue;
                                }
                                _ => (),
                            }
                        }
                        if id > 536870911 {
                            return Err(errors::ConfigError::FailedToResolveId);
                        }
                        break;
                    }
                    message_data.id = MessageIdTemplate::ExtId(id);
                }
                MessageIdTemplate::AnyAny(priority) => {
                    let mut id = priority.min_id();
                    let m_id: MessageIdTemplate;
                    loop {
                        for j in 0..messages.len() {
                            if i == j {
                                continue;
                            }
                            let other = messages[j].0.borrow();
                            match other.id {
                                MessageIdTemplate::StdId(other_id) if other_id == id => {
                                    id += 1;
                                    continue;
                                }
                                _ => (),
                            }
                        }
                        if id > 2047 {
                            loop {
                                for j in 0..messages.len() {
                                    if i == j {
                                        continue;
                                    }
                                    let other = messages[j].0.borrow();
                                    match other.id {
                                        MessageIdTemplate::ExtId(other_id) if other_id == id => {
                                            id += 1;
                                            continue;
                                        }
                                        _ => (),
                                    }
                                }
                                if id > 536870911 {
                                    return Err(errors::ConfigError::FailedToResolveId);
                                }
                                m_id = MessageIdTemplate::ExtId(id);
                                break;
                            }
                        } else {
                            m_id = MessageIdTemplate::StdId(id);
                        }
                        break;
                    }
                    message_data.id = m_id;
                }
            }
        }

        Ok(())
    }

    pub fn build(self) -> errors::Result<NetworkRef> {
        let builder = self.0.borrow();
        let baudrate = builder.baudrate.unwrap_or(1000000);

        // sort types in topological order!
        let type_builders = Self::topo_sort_type_builders(&builder.types.borrow())?;

        // define types.
        let mut types = vec![];
        for type_builder in type_builders.iter() {
            let type_ref: TypeRef = match type_builder {
                TypeBuilder::Enum(enum_builder) => {
                    let enum_data = enum_builder.0.borrow();

                    let mut entries: Vec<(String, u64)> = vec![];
                    let mut max_entry = 0;
                    for (entry_name, opt_value) in &enum_data.entries {
                        match opt_value {
                            Some(explicit_value) => {
                                entries.push((entry_name.clone(), *explicit_value));
                                max_entry = max_entry.max(*explicit_value);
                            }
                            None => {
                                if !entries.is_empty() {
                                    max_entry += 1;
                                }
                                entries.push((entry_name.clone(), max_entry));
                            }
                        }
                    }

                    let size = ((max_entry + 1) as f64).log2().ceil() as u8;
                    make_config_ref(Type::Enum {
                        name: enum_data.name.clone(),
                        size,
                        description: enum_data.description.clone(),
                        entries,
                        visibility: enum_data.visibility.clone(),
                    })
                }
                TypeBuilder::Struct(struct_builder) => {
                    let struct_data = struct_builder.0.borrow();
                    let mut attribs = vec![];
                    for (name, type_name) in &struct_data.attributes {
                        // this call requires topological sort over dependencies
                        // otherwise a type could not be defined.
                        // This creates the restiction that the types
                        // are not defined recursivly which is probably
                        // a good restriction
                        let ty = Self::resolve_type(&types, type_name)?;
                        attribs.push((name.clone(), ty));
                    }
                    make_config_ref(Type::Struct {
                        name: struct_data.name.clone(),
                        description: struct_data.description.clone(),
                        attribs,
                        visibility: struct_data.visibility.clone(),
                    })
                }
            };
            types.push(type_ref);
        }

        // resolve any ids.
        Self::resolve_ids(&mut builder.messages.borrow_mut())?;

        let mut messages = vec![];
        for message_builder in builder.messages.borrow().iter() {
            let message_data = message_builder.0.borrow();
            let id = match message_data.id {
                MessageIdTemplate::StdId(id) => MessageId::StandardId(id),
                MessageIdTemplate::ExtId(id) => MessageId::ExtendedId(id),
                MessageIdTemplate::AnyStd(_) => panic!("unresolved id"),
                MessageIdTemplate::AnyExt(_) => panic!("unresolve id"),
                MessageIdTemplate::AnyAny(_) => panic!("unresolved id"),
            };
            let (signals, encoding) = match &message_data.format {
                MessageFormat::Signals(signal_format_builder) => {
                    let mut offset : usize = 0;
                    let signal_format_data = signal_format_builder.0.borrow();
                    let mut signals = vec![];
                    for signal_data in signal_format_data.0.iter() {
                        signals.push(make_config_ref(Signal{
                                name : format!("{}_{}", message_data.name, signal_data.name),
                                offset,
                                ..signal_data.clone()
                        }));
                        offset += signal_data.size() as usize;
                    }
                    ( signals, None)
                }
                MessageFormat::Types(type_format_builder) => {
                    let type_format_data = type_format_builder.0.borrow();
                    let mut encodings = vec![];
                    let mut signals = vec![];
                    let mut offset: usize = 0;
                    for (type_name, value_name) in &type_format_data.0 {
                        let type_ref = Self::resolve_type(&types, type_name)?;
                        let type_signals = Self::type_to_signals(
                            type_ref.clone(),
                            &message_data.name,
                            value_name,
                            type_name,
                            &mut offset,
                        );

                        signals.extend_from_slice(&type_signals);

                        encodings.push(TypeSignalEncoding {
                            name: value_name.to_owned(),
                            ty: type_ref,
                            signals: type_signals,
                        });
                    }

                    (signals, Some(encodings))
                }
                MessageFormat::Empty => (vec![], None),
            };

            messages.push(make_config_ref(Message {
                name: message_data.name.clone(),
                description: message_data.description.clone(),
                id,
                encoding,
                signals,
                visbility: message_data.visibility.clone(),
            }));
        }

        // add get and set req,resp to all nodes
        let n_nodes = builder.nodes.borrow().len();
        for i in 0..n_nodes {
            for j in 0..n_nodes {
                if i == j {
                    continue;
                }
                let server_builder = &builder.nodes.borrow()[i];
                let server_data = server_builder.0.borrow();
                let client_builder = &builder.nodes.borrow()[j];
                client_builder.add_tx_message(&server_data.get_req_message);
                client_builder.add_rx_message(&server_data.get_resp_message);
                client_builder.add_tx_message(&server_data.set_req_message);
                client_builder.add_rx_message(&server_data.set_resp_message);
            }
        }

        let mut nodes = vec![];
        // first create messages with tx and rx messages.
        for node_builder in builder.nodes.borrow().iter() {
            let node_data = node_builder.0.borrow();

            let mut node_types = vec![];

            let mut rx_messages = vec![];
            for rx_message_builder in &node_data.rx_messages {
                let message_ref = messages
                    .iter()
                    .find(|m| m.name == rx_message_builder.0.borrow().name)
                    .expect("invalid message_builder was probably not added to the network");
                match &message_ref.encoding {
                    Some(encoding) => {
                        for enc in encoding {
                            let ty: &TypeRef = &enc.ty;
                            if !node_types.contains(ty) {
                                node_types.push(ty.clone());
                            }
                        }
                    }
                    None => (),
                }
                rx_messages.push(message_ref.clone());
            }
            let mut tx_messages = vec![];
            for tx_message_builder in &node_data.tx_messages {
                let message_ref = messages
                    .iter()
                    .find(|m| m.name == tx_message_builder.0.borrow().name)
                    .expect("invalid message_builder was probably not added to the network");
                match &message_ref.encoding {
                    Some(encoding) => {
                        for enc in encoding {
                            let ty: &TypeRef = &enc.ty;
                            if !node_types.contains(ty) {
                                node_types.push(ty.clone());
                            }
                        }
                    }
                    None => (),
                }
                tx_messages.push(message_ref.clone());
            }

            let mut commands: Vec<ConfigRef<Command>> = vec![];
            for tx_command_builder in &node_builder.0.borrow().commands {
                let command_data = tx_command_builder.0.borrow();
                let tx_message = messages
                    .iter()
                    .find(|m| m.name == command_data.call_message.0.borrow().name)
                    .expect("invalid command builder tx_message wasn't added to the network")
                    .clone();
                let rx_message = messages
                    .iter()
                    .find(|m| m.name == command_data.resp_message.0.borrow().name)
                    .expect("invalid command builder rx_message wasn't added to the network")
                    .clone();
                commands.push(make_config_ref(Command {
                    name: command_data.name.clone(),
                    description: command_data.description.clone(),
                    tx_message,
                    rx_message,
                    visibility: command_data.visibility.clone(),
                }));
            }

            let mut object_entries = vec![];
            let mut id_acc = 0;
            for object_entry_builder in &node_builder.0.borrow().object_entries {
                let object_entry_data = object_entry_builder.0.borrow();
                let ty = Self::resolve_type(&mut types, &object_entry_data.ty)?;
                if !node_types.contains(&ty) {
                    node_types.push(ty.clone());
                }
                let id = id_acc;
                id_acc += 1;
                object_entries.push(make_config_ref(ObjectEntry {
                    name: object_entry_data.name.clone(),
                    description: object_entry_data.description.clone(),
                    access: object_entry_data.access.clone(),
                    id,
                    ty,
                    visibility: object_entry_data.visibility.clone(),
                }));
            }

            let mut tx_streams = vec![];
            for tx_stream in &node_builder.0.borrow().tx_streams {
                let stream_data = tx_stream.0.borrow();

                //resolve message
                let message = messages
                    .iter()
                    .find(|m| m.name == stream_data.message.0.borrow().name)
                    .expect("stream message was not added to the network")
                    .clone();
                let mut mappings = vec![];
                for oe_builder in &stream_data.object_entries {
                    let oe_data = oe_builder.0.borrow();
                    let oe = object_entries
                        .iter()
                        .find(|oe| oe.name == oe_data.name)
                        .expect("stream object entry wasn't added to the node")
                        .clone();
                    mappings.push(Some(oe));
                }

                tx_streams.push(make_config_ref(Stream {
                    name: stream_data.name.clone(),
                    description: stream_data.description.clone(),
                    mappings,
                    message,
                    visibility: stream_data.visbility.clone(),
                }));
            }
            let node_types = Self::topo_sort_types(&node_types);

            let get_resp_message = tx_messages
                .iter()
                .find(|m| m.name == node_data.get_resp_message.0.borrow().name)
                .unwrap().clone();
            let get_req_message = rx_messages
                .iter()
                .find(|m| m.name == node_data.get_req_message.0.borrow().name)
                .unwrap().clone();
            let set_resp_message = tx_messages
                .iter()
                .find(|m| m.name == node_data.set_resp_message.0.borrow().name)
                .unwrap().clone();
            let set_req_message = rx_messages
                .iter()
                .find(|m| m.name == node_data.set_req_message.0.borrow().name)
                .unwrap().clone();

            nodes.push(RefCell::new(Node {
                name: node_data.name.clone(),
                description: node_data.description.clone(),
                object_entries,
                extern_commands: vec![],
                commands,
                rx_messages,
                tx_messages,
                rx_streams: vec![],
                tx_streams,
                types: node_types,
                set_resp_message,
                set_req_message,
                get_resp_message,
                get_req_message
            }));
        }

        // add extern commands to nodes
        // requires all nodes to be constructed beforehand.
        for i in 0..n_nodes {
            let node_builder = &builder.nodes.borrow()[i];
            let node_data = node_builder.0.borrow();
            for rx_command in &node_data.extern_commands {
                let rx_command_data = rx_command.0.borrow();
                'outer: for j in 0..n_nodes {
                    if i == j {
                        continue;
                    }
                    let other_node = nodes[j].borrow();
                    for tx_command in &other_node.commands {
                        if tx_command.tx_message.name
                            == rx_command_data.call_message.0.borrow().name
                        {
                            nodes[i]
                                .borrow_mut()
                                .extern_commands
                                .push((other_node.name.clone(), tx_command.clone()));
                            break 'outer;
                        }
                    }
                }
            }
            for rx_stream in &node_data.rx_streams {
                let rx_stream_data = rx_stream.0.borrow();
                let tx_stream_builder = rx_stream_data.stream_builder.clone();
                let tx_stream_data = tx_stream_builder.0.borrow();
                let tx_node_builder = tx_stream_data.tx_node.clone();
                let tx_node_data = tx_node_builder.0.borrow();
                // resolve node.
                let tx_node = nodes
                    .iter()
                    .find(|n| n.borrow().name == tx_node_data.name)
                    .unwrap()
                    .borrow();
                let tx_stream = tx_node
                    .tx_streams
                    .iter()
                    .find(|s| s.name == tx_stream_data.name)
                    .unwrap()
                    .clone();

                let mut builder_mapping = rx_stream_data.object_entries.clone();
                builder_mapping.sort_by(|(i1, _), (i2, _)| {
                    if i1 < i2 {
                        Ordering::Less
                    } else if i1 == i2 {
                        Ordering::Equal
                    } else {
                        Ordering::Greater
                    }
                });
                let oe_count = tx_stream.mappings.len();
                let mut mappings = vec![];
                let mut j = 0;
                let rx_node_data = rx_stream_data.rx_node.0.borrow();
                let rx_node = nodes
                    .iter()
                    .find(|n| n.borrow().name == rx_node_data.name)
                    .unwrap()
                    .borrow();
                for i in 0..oe_count {
                    if builder_mapping[j].0 == i {
                        // search for object entry in rx_node
                        let oe = rx_node
                            .object_entries
                            .iter()
                            .find(|oe| oe.name == builder_mapping[j].1 .0.borrow().name)
                            .unwrap();
                        mappings.push(Some(oe.clone()));
                        j += 1;
                    } else {
                        // insert null mapping
                        mappings.push(None);
                    }
                }

                drop(tx_node);
                drop(rx_node);
                nodes[i]
                    .borrow_mut()
                    .rx_streams
                    .push(make_config_ref(Stream {
                        name: tx_stream.name.clone(),
                        description: tx_stream.description.clone(),
                        message: tx_stream.message.clone(),
                        mappings,
                        visibility: rx_stream_data.visibility.clone(),
                    }));
            }
        }

        let nodes = nodes
            .into_iter()
            .map(|n| make_config_ref(n.into_inner()))
            .collect();

        Ok(make_config_ref(Network {
            baudrate,
            build_time: chrono::Local::now(),
            types,
            messages,
            nodes,
        }))
    }
}
