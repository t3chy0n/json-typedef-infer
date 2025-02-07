# jtd-infer: Generate JSON Typedef schemas from examples [![Crates.io](https://img.shields.io/crates/v/jtd_infer)](https://crates.io/crates/jtd_infer) [![Docs.rs](https://docs.rs/jtd-infer/badge.svg)](https://docs.rs/jtd_infer)

[JSON Type Definition](https://jsontypedef.com), aka
[RFC8927](https://tools.ietf.org/html/rfc8927), is an easy-to-learn,
standardized way to define a schema for JSON data. You can use JSON Typedef to
portably validate data across programming languages, create dummy data, generate
code, and more.

## Building using wasm pack

```
cargo install wasm-pack
```

Node
```
wasm-pack build --target nodejs
```
Web
```
wasm-pack build --target web
```

# About

`jtd-infer` is a tool that generates ("infers") a JSON Typedef schema from
example data. This fork focuses on exposing jtd-infer webassembly bindings, which
can be used from Javascript code as follows:

```bash
echo '{ "name": "Joe", "age": 42 }' | jtd-infer | jq
```

```json
{
  "properties": {
    "age": {
      "type": "uint8"
    },
    "name": {
      "type": "string"
    }
  }
}
```

## Usage

For high-level guidance on how to use `jtd-infer`, see ["Inferring a JSON
Typedef Schema from Real Data"][jtd-jtd-infer] in the JSON Typedef website docs.

### Basic Usage

```typescript
import * as jtdInfer from 'jtd-infer';

// This is an async action since WebAssembly modules are promise-based in Node.js.
async function useLibrary() {
//    await wasmLib.default(); // This initializes the WASM module.

    // Sample data to pass to the function
    const input = '{"a": 2}';
    const enumHints = [];
    const valuesHints = [];
    const discriminatorHints = [];
    const defaultNumberType = "int8";

    try {
        const result = jtdInfer.generate_schema({input, enumHints, valuesHints, discriminatorHints, defaultNumberType});
        console.log(result);
    } catch (error) {
        console.error("Error:", error);
    }
}

useLibrary();

```
To invoke `jtd-infer`, you can either:

1. Add a jtd-infer dependency.
2. Import it in your node js app.

`jtd-infer` reads a _sequence_ of JSON messages. So for example, if you have a
file like this in `data.json`:

```json
{ "name": "john doe", "age": 42 }
{ "name": "jane doe", "age": 45 }
```

In both cases, you'd get this output:

```json
{"properties":{"name":{"type":"string"},"age":{"type":"uint8"}}}
```

### Changing the default number type

> ⚠️ This section is often important if you are retrofitting JSON Typedef to a
> JavaScript-based application.

By default, JSON Typedef will infer the most specific possible type for inputs.
So, for example, it will guess `uint8` if it sees a `12` in your input:

generate_schema accepts `defaultNumberType` inside parameters object.

```json
{"type":"int8"}
```

However, if you're giving JSON Typedef a small sample set, or if you in practice
have data that is far smaller than the actual numerical datatypes your
application supports, then this behavior may be undesirable. For example, it's
common for JavaScript-based applications to actually support `float64` for all
numeric inputs, because JavaScript numbers are IEEE double-precision floats.

To tell JSON Typedef to prefer a different type than the one it would normally
guess, you can use `defaultNumberType` to change its behavior. For example:

```bash
# JavaScript numbers are all float64s, and so it's pretty common for JavaScript
# applications to not check if inputs are integers or within a particular range.
#
# If you don't want to make your JSON Typedef schema strict about decimal,
# negative, or out of int range numbers, you can pass float64 as the default
# number type.
 const result = jtdInfer.generate_schema({
   input:"12",
   enumHints,
   valuesHints,
   discriminatorHints, 
   defaultNumberType:"float64"
 });
 console.log(result);
```

```json
{"type":"float64"}
```

Another use-case is if you're writing an application that uses signed 32-bit
ints everywhere, and your example data simply never in practice has examples of
negative numbers or numbers too big for 8- or 16-bit numbers. You can achieve
that by using `int32` as your default number type:

```bash
 const result = jtdInfer.generate_schema({
    input: "12",
    enumHints, 
    valuesHints,
    discriminatorHints,
    defaultNumberType:"int32"
 });
 console.log(result);
```

```json
{"type":"int32"}
```

Note that `jtd-infer` will ignore your default if it doesn't match with the
data. For example, `int32` only works with whole numbers, so if a decimal number
or a number too big for 32-bit signed integers comes in, it will fall back to
`float64`:

```bash
# both of these output {"type":"float64"}
 const result = jtdInfer.generate_schema({
   input: "3.14",
    enumHints,
    valuesHints,
    discriminatorHints, 
    defaultNumberType: "int32"
 });
 
 const result2 = jtdInfer.generate_schema({
   input: "9999999999",
    enumHints,
    valuesHints,
    discriminatorHints, 
    defaultNumberType: "int32"
 });
 
 console.log(result);
 console.log(result2);

```

### Advanced Usage: Providing Hints

By default, `jtd-infer` will never output `enum`, `values`, or `discriminator`
schemas. This is by design: by always being consistent with what it outputs,
`jtd-infer` is more predictable and reliable.

If you want `jtd-infer` to output an `enum`, `values`, or `discriminator`, you
can use the `enumHints`, `valueHints`, and `discriminatorHint` flags.
You can pass each of these flags multiple times.

All of the hint flags accept [JSON
Pointers](https://tools.ietf.org/html/rfc6901) as values. If you're used to the
JavaScript-y syntax of referring to things as `$.foo.bar`, the equivalent JSON
Pointer is `/foo/bar`. `jtd-infer` treats `-` as a "wildcard". `/foo/-/bar` is
equivalent to the JavaScript-y `$.foo.*.bar`.

As a corner-case, if you want to point to the *root* / top-level of your input,
then use the empty string as the path. See ["Using
`--values-hint`"](##using---values-hint) for an example of this.

#### Using `enumHints` option

By default, strings are always inferred to be `{ "type": "string" }`:

```bash

 const result = jtdInfer.generate_schema({
   input: '["foo", "bar", "baz"]',
    enumHints, 
    valuesHints,
    discriminatorHints,
    defaultNumberType: "int32"
 });
 console.log(result);
```

```json
{"elements":{"type":"string"}}
```

But you can instead have `jtd-infer` output an enum by providing a path to the
string you consider to be an enum. In this case, it's any element of the root of
the array -- the JSON Pointer for that is `/-`:

```bash

 const result = jtdInfer.generate_schema({
   input: '["foo", "bar", "baz"]',
   enumHints: ["/-"],
   valuesHints,
   discriminatorHints,
   defaultNumberType: "int32"
 });
 
 console.log(result);
```

```json
{"elements":{"enum":["bar","baz","foo"]}}
```

#### Using `valuesHint`

By default, objects are always assumed to be "structs", and `jtd-infer` will
generate `properties` / `optionalProperties`. For example:

```bash
 const result = jtdInfer.generate_schema({
   input: '{"x": [1, 2, 3], "y": [4, 5, 6], "z": [7, 8, 9]}',
   enumHints,
   valuesHints,
   discriminatorHints,
   defaultNumberType:"int32"
 });
 
 console.log(result);
```

```json
{"properties":{"y":{"elements":{"type":"uint8"}},"z":{"elements":{"type":"uint8"}},"x":{"elements":{"type":"uint8"}}}}
```

If your data is more like a map / dictionary, pass a `values-hint` that points
to the object that you want a `values` schema from. In this case, that's the
root-level object, which in JSON Pointer is just an empty string:

```bash
  const result = jtdInfer.generate_schema({
   input: '{"x": [1, 2, 3], "y": [4, 5, 6], "z": [7, 8, 9]}',
   enumHints,
   valuesHints: [""],
   discriminatorHints,
   defaultNumberType:"int32"
 });
 console.log(result);
```

```json
{"values":{"elements":{"type":"uint8"}}}
```

#### Using `--discriminator-hint`

By default, objects are always assumed to be "structs", and `jtd-infer` will
generate `properties` / `optionalProperties`. For example:

```bash
 const result = jtdInfer.generate_schema({
   input: '[{"type": "s", "value": "foo"},{"type": "n", "value": 3.14}]',
   enumHints,
   valuesHints,
   discriminatorHints,
   defaultNumberType:"int32"
 });
 console.log(result);
```

```json
{"elements":{"properties":{"value":{},"type":{"type":"string"}}}}
```

If your data has a special "type" property that tells you what's in the rest of
the object, then use `--discriminator-hint` to point to that property.
`jtd-infer` will output an appropriate `discriminator` schema instead:

```bash
 const result = jtdInfer.generate_schema({
   input: '[{"type": "s", "value": "foo"},{"type": "n", "value": 3.14}]',
   enumHints,
   valuesHints,
   discriminatorHints: ['/-/type'], 
   defaultNumberType: "int32"
  });
 console.log(result);
```

```json
{
  "elements": {
    "discriminator": "type",
    "mapping": {
      "s": {
        "properties": {
          "value": {
            "type": "string"
          }
        }
      },
      "n": {
        "properties": {
          "value": {
            "type": "float64"
          }
        }
      }
    }
  }
}
```

[jtd-jtd-infer]: https://jsontypedef.com/docs/tools/jtd-infer
[latest]: https://github.com/jsontypedef/json-typedef-infer/releases/latest
