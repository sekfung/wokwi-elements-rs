# Compatibility Oracle

`wokwi-elements-1.9.2/` is the immutable published `@wokwi/elements@1.9.2`
package used only by the Rust port's differential tests and fixture generators.
It is distributed under the MIT license included in that directory. Runtime
code does not import or execute the upstream JavaScript.

`contracts/` contains the captured property/attribute/pin/default metadata and
the rendered default DOM used to generate Rust fixtures.
