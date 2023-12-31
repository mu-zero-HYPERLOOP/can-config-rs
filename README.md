
#### Network
- **baudrate** : baudrate of the network
- **nodes** : all nodes in the network
- **messages** : all messages in the network

#### Node
- **name** : name of the node
- **description** : description of the node
- **tx_messages** : messages transmitted by the node
- **rx_messages** : messages received by the node
- **types**       : types that are used by the node
- **rx_commands** : commands that other nodes can call
- **tx_commands** : commands that this node can call
- **tx_streams** : streams transmitted by this node
- **rx_streams** : streams received by this node
- **object_entries** : values defined by this node
- **get_resp_message** : message used to respond to get requests.
- **set_resp_message** : message used to respond to set requests.
- **get_req_message** : message received on a get request.
- **set_req_message** : message received on a set request.

#### Message
- **name** : name of the message
- **description**: description of the node
- **signals** : signals that compose this message
- **encoding** : defines how named types are mapped to signals.
- **dlc** : defined the length of the message.
- **id** : id of the message can be standard or extended identifier.

#### Signal
signals can only belong to one message.
- **name**: name of the signal
- **description**: description of the signal
- **type** : type of the signal
- **value_table** : value tables map values to enums
- **byte_offset** : byte_offset of the signal in the owning message

#### SignalType
A enum that can be a Integer or a Decimal Type.
Decimals are basically fix point values.
- UnsignedInt{ size : u8 }
- SignedInt{ size : u8 }
- Decimal{ size : u8, offset : f64, scale : f64 }

#### ObjectEntry
A object entry describes a value that a node owns.
ObjectEntries can be modified over the get and set protocol.
They can also be mapped to streams for realtime data transfer.
- **name** : name of the object entry
- **description**: description of the object entry
- **id** : the id of the object entry
- **ty** : the type of the value stored in the object entry
- **access** : 
    - Const : no write, no read
    - Local : local write, global read
    - Global : global write, global read
#### Stream
A stream defines a single producer multiple consumer
communication model, without any data overhead.
- **name** : name of the stream
- **description** : description of the stream
- **mappings** : defines how the data of the stream is mapped to object entries (for rx or tx).
- **message** : the message that the stream uses.

#### Commands
- **name** : name of the command
- **description** : description of the command
- **tx_message** : message used to invoke the command
- **rx_message** : message used to respond to the callee

****

##### Visibility
Another concept is visibility some config objects
define visibility. Visibility can be Global or 
Static if a object is Statically visible it 
should not be exposed to the user of the autogenerated
code.
