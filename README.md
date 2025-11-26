# Postoffice

Postoffice is a message broker that can receive, filter, transform and publish messages from and to different protocols.

This is done by defining a `pipeline` in a json config file. The config file is divided into two parts, the `connectors` and `blocks`.

```json
{
  "connectors": [],
  "blocks": []
}
```

A `connector` is something that can receive (`Source`) or publish (`Sink`) messages. A single `connector` may also be able to do both (`Source`, `Sink`).

A `block` is something that can filter or transform a message.

The `pipeline` is build by defining links to a `block` or `sink`. This is done by defining a `to` section on a `block` or `source`.

```jsonc
{
  "connectors": [
    {
      "MQTT": {
        // Received messages will be forwarded to the block with index 0
        "to": [{ "Block": 0 }],

        "config": // Additional config
      }
    }
  ],
  "blocks": [
    {
      "Wait": {
        // Messages will be forwarded to the connector with index 0
        "to": [{ "Sink": 0 }],

        "config": // Additional config
      }
    }
  ]
}
```

Take a look at the examples for a complete config.

## Connectors

### MQTT

#### Type:

`Source`, `Sink`

The MQTT connector can connect to a running MQTT broker and can receive and publish messages. It can also subscribe to specific, all or no topics.

#### Config

```ts
{
  "host": string,
  "port": u16,
  "client_id"?: string,
  "topics"?: string[]
}
```

`host`, `port` are used to connect to an MQTT broker running at this host and port.

`client_id` is the client id that this connector uses to connect to the MQTT broker. Keep in mind that every client id has to be unique. It defaults to `postoffice`.

`topics` is an optional array with topics that the connector subscribes to. If no array is given, it will subscribe to `#`. If an empty array is given, it will not subscribe to any topic. This can be useful if you only want to publish messages.

### OSCRecv

#### Type:

`Source`

The OSCRecv connector can receive OSC messages.

#### Config

```ts
{
  "interface": string,
  "port": u16
}
```

`interface`, `port` are used to listen for messages. Use `0.0.0.0` as interface to listen on all interfaces.

### OSCSend

#### Type:

`Sink`

The OSCSend connector can send OSC messages to a specified host and port.

#### Config

```ts
{
  "host": string,
  "port": u16
}
```

`host`, `port` are used to send the OSC message to.

### UDPSend

#### Type:

`Sink`
The UDPSend connector can send the **message topic** to a host over UDP.

#### Config

```ts
{
  "host": string,
  "port": u16
}
```

`host`, `port` are used to send the UDP message to.

## Blocks

### AddLeadingSlash

The AddLeadingSlash block adds a leading slash to the topic if it doesnt already exists.

#### Examples

```
a/b/c -> /a/b/c
/a/b/c -> /a/b/c
```

### RemoveLeadingSlash

The RemoveLeadingSlash block removes all leading slashes of the topic.

#### Examples

```
a/b/c -> a/b/c
/a/b/c -> a/b/c
//a/b/c -> a/b/c
```

### RemoveBody

The RemoveBody block converts the body to `InternalMessageData::Empty`. This can be useful if the body should be ignored and not be sent.

### ReplaceBody

The ReplaceBody block replaces the body with json data from the config file. The body will be of type `InternalMessageData::Json` after this block.

#### Config

```ts
{
  "config": {
    "some": [
      "json",
      "data"
    ]
  }
}
```

### MatchTopic

The MatchTopic block filters messages based on a filter type and message.

#### Config

```ts
{
  "config": {
    "Exact" | "StartsWith" | "EndsWith" | "Regex": string
  }
}
```

`Exact` checks if the topic is exactly the same as the given pattern

`StartsWith` checks if the topic starts with the given pattern

`EndsWith` checks if the topic ends with the given pattern

`Regex` checks if the topic matches the given regex pattern

#### Examples

```
{ "Exact": "/a/b/c" }
/a/b/c -> true
/a/b -> false
a/b/c -> false

{ "StartsWith": "/a" }
/a/b/c -> true
/a -> true
a/b/c -> false

{ "EndsWith": "/c" }
/a/b/c -> true
/c -> true
/a/b -> false

{ "Regex": "\\/b" } (\ has to be escaped)
/a/b/c -> true
/b -> true
/a/b -> true
```

### ReplaceTopic

The ReplaceTopic block replaces the topic with a new topic from the config file.

#### Config

```ts
{ "config": string }
```

### LuaFilter

The LuaFilter block runs a `lua` script and filters the message based on the outputs. The topic is available under the global `topic` variable and the body is available under the global `data` variable. To filter messages you can use the global `finish` function and pass a boolean. If the boolean is true the message will be forwarded in the pipeline. If the boolean is false or `finish` is never called the message will be dropped.

#### Config

```ts
{
  "config": {
    "Inline" | "Script": string
  }
}
```

`Inline` uses the given string as a lua script.

`Script` uses the given string as a file path to a lua script.

### ConvertBody

The ConvertBody block tries to convert the body to the given type from the config file. Keep in mind however, that not all conversions will suceed.

#### Config

```ts
{
  "config": "Empty" | "JSON" | "OSC" | "Binary" | "String"
}
```

### Wait

The Wait block waits for the given amount of milliseconds before fowarding the message in the pipeline.

#### Config

```ts
{
  "config": u64
}
```
