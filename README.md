# Telemetry message generation library.

Models generate streams of telemetry messages by implementing
[model::TelemStream](crates/telem-gen/src/model.rs). Protocols are used to
generate specific message formats and
encodings, by implementing [protocol::TelemMsg](crates/telem-gen/src/protocol/mod.rs).

Initial support for CoT (cursor on target) XML message generation based on a
"random walk" movement model within a bounding box.

Still under initial development. To generate docs, run `cargo doc`.
