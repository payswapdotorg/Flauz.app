//! The versioned session/event transport (kernel §5).
//!
//! Carries session event streams as bounded, newline-delimited JSON frames.
//! The transport is **generic** over the envelope value:
//!
//! ```text
//! E: Serialize + DeserializeOwned + Clone + Debug + Send + Sync + 'static
//! ```
//!
//! Envelopes are opaque values: the transport has **its own framing
//! version field** (the frame's `"v"`, currently
//! [`TransportFrameVersion`] = 1) and never inspects envelope internals —
//! it does not read event IDs, streams, sequences or payload fields. That
//! is what lets the integration harness feed the world crate's
//! `EventEnvelope` through this transport without any dependency between
//! the crates, and no exec-crate trait is ever implemented on a foreign
//! type (orphan-rule safety).
//!
//! Wire format (one frame per line, JSON):
//!
//! ```json
//! {"v":1,"kind":"flauz.transport.frame","envelope":{...opaque...}}
//! ```
//!
//! Decoding is two-phase: the transport first reads **its own** frame
//! header (the framing version and kind tag — the only fields it is
//! allowed to inspect), rejecting unsupported framing versions, then
//! parses the full frame strictly (`deny_unknown_fields`). The envelope
//! value is handed to `E` untouched.
//!
//! Frames are bounded ([`DEFAULT_MAX_TRANSPORT_FRAME_BYTES`]); every read
//! and write is bounded, with no unbounded allocations (house rule).

use std::io::{self, BufRead, Write};
use std::marker::PhantomData;
use std::num::NonZeroUsize;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// The default maximum size of one transport frame, matching the house
/// protocol default.
pub const DEFAULT_MAX_TRANSPORT_FRAME_BYTES: usize = 16 * 1024 * 1024;

/// The current framing version of the transport. This is the transport's
/// OWN version field, independent of any envelope's schema version: the
/// transport never reads the envelope's version.
pub const TRANSPORT_FRAME_VERSION: u32 = 1;

/// A bounded frame-size violation (house protocol shape).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameTooLarge {
    /// The configured frame size limit.
    pub limit: usize,
    /// The size already observed when the limit was hit.
    pub observed_at_least: usize,
}

impl std::fmt::Display for FrameTooLarge {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "transport frame exceeded {} bytes (observed at least {})",
            self.limit, self.observed_at_least
        )
    }
}

impl std::error::Error for FrameTooLarge {}

/// Errors returned by the transport.
#[derive(Debug)]
pub enum TransportError {
    /// The underlying stream failed.
    Io(io::Error),
    /// A frame exceeded the configured size limit.
    FrameTooLarge(FrameTooLarge),
    /// The frame bytes are not valid JSON.
    InvalidJson,
    /// The frame structure violates the framing contract (header shape,
    /// kind tag, empty frame).
    InvalidFrame(&'static str),
    /// The frame carries an unsupported framing version.
    UnsupportedFrameVersion {
        /// The framing version the frame carried.
        found: u32,
        /// The framing version this build supports.
        supported: u32,
    },
    /// The opaque envelope value failed to deserialize into `E`.
    Envelope(String),
    /// An envelope could not be serialized.
    Serialization,
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "transport I/O failed: {error}"),
            Self::FrameTooLarge(error) => write!(formatter, "{error}"),
            Self::InvalidJson => formatter.write_str("transport frame is not valid JSON"),
            Self::InvalidFrame(reason) => write!(formatter, "invalid transport frame: {reason}"),
            Self::UnsupportedFrameVersion { found, supported } => write!(
                formatter,
                "unsupported transport framing version {found}; this build frames v{supported}"
            ),
            Self::Envelope(reason) => {
                write!(formatter, "framed envelope failed to deserialize: {reason}")
            }
            Self::Serialization => formatter.write_str("envelope serialization failed"),
        }
    }
}

impl std::error::Error for TransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::FrameTooLarge(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for TransportError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<FrameTooLarge> for TransportError {
    fn from(error: FrameTooLarge) -> Self {
        Self::FrameTooLarge(error)
    }
}

/// The transport framing schema version marker. Serializes as the frame's
/// `"v": 1` and rejects any other value, so a frame written by a different
/// framing version is rejected by canonical reads instead of being
/// silently misread. Independent of every envelope's own schema version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransportFrameVersion;

impl Serialize for TransportFrameVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(TRANSPORT_FRAME_VERSION)
    }
}

impl<'de> Deserialize<'de> for TransportFrameVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let version = u32::deserialize(deserializer)?;
        if version == TRANSPORT_FRAME_VERSION {
            Ok(Self)
        } else {
            Err(serde::de::Error::custom(format!(
                "unsupported transport framing version {version}; this build frames v{TRANSPORT_FRAME_VERSION}"
            )))
        }
    }
}

/// The frozen kind tag of a transport frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportFrameKind {
    /// The `flauz.transport.frame` frame kind.
    #[serde(rename = "flauz.transport.frame")]
    FlauzTransportFrame,
}

/// One versioned transport frame carrying an opaque envelope value. The
/// frame's `v` is the **framing** version (this transport's own version
/// field); the envelope inside is never inspected by the transport.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransportFrame<E> {
    /// The transport framing version (`"v": 1`).
    pub v: TransportFrameVersion,
    /// The frozen frame kind tag.
    pub kind: TransportFrameKind,
    /// The opaque envelope value. Its internals are invisible to the
    /// transport; it round-trips as a whole through `E`'s serde impls.
    pub envelope: E,
}

impl<E> TransportFrame<E> {
    /// Builds a frame around an envelope value.
    #[must_use]
    pub fn new(envelope: E) -> Self {
        Self {
            v: TransportFrameVersion,
            kind: TransportFrameKind::FlauzTransportFrame,
            envelope,
        }
    }
}

/// The non-strict header probe: the only part of a frame the transport
/// inspects — its own version field and kind tag. Extra fields (the
/// envelope) are ignored at this stage.
#[derive(Deserialize)]
struct FrameHeader {
    v: u32,
    kind: String,
}

/// The versioned session/event transport: encodes envelopes as bounded,
/// newline-delimited versioned frames and decodes them back, generic over
/// the envelope type.
///
/// The generic bound is exactly the kernel §5 bound:
/// `E: Serialize + DeserializeOwned + Clone + Debug + Send + Sync +
/// 'static`. Any envelope satisfying it — the world crate's
/// `EventEnvelope` at integration, or plain `serde_json::Value` — flows
/// through unchanged, with value equality preserved by round-trips.
pub struct SessionEventTransport<E> {
    max_frame_bytes: NonZeroUsize,
    marker: PhantomData<fn() -> E>,
}

impl<E> SessionEventTransport<E> {
    /// A transport with the default frame size limit.
    #[must_use]
    pub fn new() -> Self {
        Self::with_max_frame_bytes(
            NonZeroUsize::new(DEFAULT_MAX_TRANSPORT_FRAME_BYTES).unwrap_or(NonZeroUsize::MIN),
        )
    }

    /// A transport with an explicit frame size limit.
    #[must_use]
    pub fn with_max_frame_bytes(max_frame_bytes: NonZeroUsize) -> Self {
        Self {
            max_frame_bytes,
            marker: PhantomData,
        }
    }

    /// The configured maximum size of one frame.
    #[must_use]
    pub fn max_frame_bytes(&self) -> usize {
        self.max_frame_bytes.get()
    }
}

impl<E> Default for SessionEventTransport<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E> SessionEventTransport<E>
where
    E: Serialize + DeserializeOwned + Clone + Debug + Send + Sync + 'static,
{
    /// Serializes and frames one envelope: the full frame bytes including
    /// the trailing newline. Fails when the frame would exceed the size
    /// limit.
    pub fn encode(&self, envelope: &E) -> Result<Vec<u8>, TransportError> {
        let frame = TransportFrame::new(envelope.clone());
        let mut bytes = serde_json::to_vec(&frame).map_err(|_| TransportError::Serialization)?;
        // +1 accounts for the trailing newline.
        if bytes.len().saturating_add(1) > self.max_frame_bytes.get() {
            return Err(TransportError::FrameTooLarge(FrameTooLarge {
                limit: self.max_frame_bytes.get(),
                observed_at_least: bytes.len(),
            }));
        }
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// Decodes one frame's bytes (without the trailing newline) back into
    /// the envelope value.
    ///
    /// Two-phase: the transport reads its own framing version and kind tag
    /// first (the only fields it may inspect), then parses the full frame
    /// strictly. The envelope value is deserialized by `E` untouched.
    pub fn decode(&self, frame: &[u8]) -> Result<E, TransportError> {
        if frame.is_empty() {
            return Err(TransportError::InvalidFrame("frame must not be empty"));
        }
        let header: FrameHeader = serde_json::from_slice(frame).map_err(classify_header)?;
        if header.v != TRANSPORT_FRAME_VERSION {
            return Err(TransportError::UnsupportedFrameVersion {
                found: header.v,
                supported: TRANSPORT_FRAME_VERSION,
            });
        }
        if header.kind != "flauz.transport.frame" {
            return Err(TransportError::InvalidFrame(
                "frame kind must be \"flauz.transport.frame\"",
            ));
        }
        let parsed: TransportFrame<E> = serde_json::from_slice(frame).map_err(classify_envelope)?;
        Ok(parsed.envelope)
    }

    /// Writes one framed envelope to a stream (bounded per frame).
    pub fn write(&self, envelope: &E, writer: &mut impl Write) -> Result<(), TransportError> {
        let bytes = self.encode(envelope)?;
        writer.write_all(&bytes)?;
        Ok(())
    }

    /// Reads one framed envelope from a buffered stream. Returns `Ok(None)`
    /// at a clean end of stream. Blank lines (after carriage-return
    /// trimming) are skipped, so trailing newlines in a stream are not
    /// framing errors.
    pub fn read(&self, reader: &mut impl BufRead) -> Result<Option<E>, TransportError> {
        loop {
            let frame = match read_bounded_frame(reader, self.max_frame_bytes)? {
                Some(frame) => frame,
                None => return Ok(None),
            };
            if frame.is_empty() {
                continue;
            }
            return self.decode(&frame).map(Some);
        }
    }

    /// Reads at most `limit` framed envelopes from a buffered stream,
    /// stopping at a clean end of stream. Bounded by both the frame size
    /// limit and the count (house rule: no unbounded history queries).
    pub fn read_up_to(
        &self,
        reader: &mut impl BufRead,
        limit: usize,
    ) -> Result<Vec<E>, TransportError> {
        let mut envelopes = Vec::new();
        for _ in 0..limit {
            match self.read(reader)? {
                Some(envelope) => envelopes.push(envelope),
                None => break,
            }
        }
        Ok(envelopes)
    }
}

fn classify_header(error: serde_json::Error) -> TransportError {
    if error.is_syntax() {
        TransportError::InvalidJson
    } else {
        TransportError::InvalidFrame("frame header must be an object carrying v and kind")
    }
}

fn classify_envelope(error: serde_json::Error) -> TransportError {
    if error.is_syntax() {
        TransportError::InvalidJson
    } else {
        TransportError::Envelope(error.to_string())
    }
}

/// Reads one newline-delimited frame while keeping both the allocation and
/// oversize recovery bounded (house protocol pattern). An oversized line
/// is consumed through its delimiter so a caller can decide whether to
/// continue. Carriage returns immediately before a newline are trimmed.
fn read_bounded_frame(
    reader: &mut impl BufRead,
    max_frame_bytes: NonZeroUsize,
) -> Result<Option<Vec<u8>>, TransportError> {
    let limit = max_frame_bytes.get();
    let mut frame = Vec::with_capacity(limit.min(8 * 1024));

    loop {
        let (available_len, newline_index) = {
            let available = reader.fill_buf()?;
            if available.is_empty() {
                if frame.is_empty() {
                    return Ok(None);
                }
                trim_carriage_return(&mut frame);
                return Ok(Some(frame));
            }
            (
                available.len(),
                available.iter().position(|byte| *byte == b'\n'),
            )
        };

        let content_len = newline_index.unwrap_or(available_len);
        if frame.len().saturating_add(content_len) > limit {
            let consume_len = newline_index.map_or(available_len, |index| index + 1);
            reader.consume(consume_len);
            if newline_index.is_none() {
                discard_through_newline(reader)?;
            }
            return Err(TransportError::FrameTooLarge(FrameTooLarge {
                limit,
                observed_at_least: frame.len().saturating_add(content_len),
            }));
        }

        {
            let available = reader.fill_buf()?;
            frame.extend_from_slice(&available[..content_len]);
        }
        let consume_len = newline_index.map_or(available_len, |index| index + 1);
        reader.consume(consume_len);

        if newline_index.is_some() {
            trim_carriage_return(&mut frame);
            return Ok(Some(frame));
        }
    }
}

fn trim_carriage_return(frame: &mut Vec<u8>) {
    if frame.last() == Some(&b'\r') {
        frame.pop();
    }
}

fn discard_through_newline(reader: &mut impl BufRead) -> Result<(), io::Error> {
    loop {
        let (available_len, newline_index) = {
            let available = reader.fill_buf()?;
            if available.is_empty() {
                return Ok(());
            }
            (
                available.len(),
                available.iter().position(|byte| *byte == b'\n'),
            )
        };
        reader.consume(newline_index.map_or(available_len, |index| index + 1));
        if newline_index.is_some() {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }

    fn err<T>(result: Result<T, TransportError>) -> TransportError {
        match result {
            Ok(_) => panic!("expected a transport error"),
            Err(error) => error,
        }
    }

    /// A minimal stand-in for the world crate's envelope: the transport
    /// must not care what `E` is.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct TestEnvelope {
        event_id: String,
        seq: u64,
        event_type: String,
        ts: String,
    }

    fn test_envelope(seq: u64) -> TestEnvelope {
        TestEnvelope {
            event_id: format!("ev_01J8ZQ5V8K3T2B7N6X4R9DQPD{seq:01}"),
            seq,
            event_type: "task.model_changed".to_owned(),
            ts: "2026-09-21T13:45:00Z".to_owned(),
        }
    }

    #[test]
    fn frames_carry_their_own_version_and_kind() {
        let transport = SessionEventTransport::<TestEnvelope>::new();
        let bytes = ok(transport.encode(&test_envelope(1)));
        let line = String::from_utf8_lossy(&bytes).trim_end().to_owned();
        assert!(line.starts_with("{\"v\":1,\"kind\":\"flauz.transport.frame\",\"envelope\":"));
        let parsed: TransportFrame<TestEnvelope> = ok(serde_json::from_str(&line));
        assert_eq!(parsed.envelope, test_envelope(1));
        assert_eq!(bytes.last(), Some(&b'\n'));
    }

    #[test]
    fn stream_round_trip_preserves_values_in_order() {
        let transport = SessionEventTransport::<TestEnvelope>::new();
        let envelopes: Vec<TestEnvelope> = (1..=3).map(test_envelope).collect();
        let mut stream: Vec<u8> = Vec::new();
        for envelope in &envelopes {
            ok(transport.write(envelope, &mut stream));
        }
        // A trailing blank line is tolerated, not a framing error.
        stream.push(b'\n');
        let mut reader = std::io::Cursor::new(&stream);
        let restored = ok(transport.read_up_to(&mut reader, 16));
        assert_eq!(restored, envelopes);
    }

    #[test]
    fn decode_rejects_unsupported_framing_versions() {
        let transport = SessionEventTransport::<TestEnvelope>::new();
        let future = "{\"v\":2,\"kind\":\"flauz.transport.frame\",\"envelope\":{}}";
        let error = err(transport.decode(future.as_bytes()));
        assert!(matches!(
            error,
            TransportError::UnsupportedFrameVersion {
                found: 2,
                supported: 1
            }
        ));
    }

    #[test]
    fn decode_rejects_wrong_kind_tags_and_bad_json() {
        let transport = SessionEventTransport::<TestEnvelope>::new();
        let wrong_kind = "{\"v\":1,\"kind\":\"flauz.event\",\"envelope\":{}}";
        assert!(matches!(
            err(transport.decode(wrong_kind.as_bytes())),
            TransportError::InvalidFrame("frame kind must be \"flauz.transport.frame\"")
        ));
        let not_json = "{not json";
        assert!(matches!(
            err(transport.decode(not_json.as_bytes())),
            TransportError::InvalidJson
        ));
        let not_object = "[1,2,3]";
        assert!(transport.decode(not_object.as_bytes()).is_err());
        assert!(matches!(
            err(transport.decode(&[])),
            TransportError::InvalidFrame("frame must not be empty")
        ));
    }

    #[test]
    fn oversized_frames_are_rejected_and_streaming_stays_bounded() {
        let transport = SessionEventTransport::<TestEnvelope>::with_max_frame_bytes(
            NonZeroUsize::new(64).unwrap_or(NonZeroUsize::MIN),
        );
        let error = err(transport.encode(&test_envelope(1)));
        assert!(matches!(error, TransportError::FrameTooLarge(_)));

        // A frame larger than the limit is consumed through its delimiter:
        // reading a stream where a huge frame precedes a valid one fails on
        // the huge frame without unbounded allocation.
        let small = ok(SessionEventTransport::<TestEnvelope>::new().encode(&test_envelope(2)));
        let mut stream: Vec<u8> = vec![b'x'; 128];
        stream.push(b'\n');
        stream.extend_from_slice(&small);
        let mut reader = std::io::Cursor::new(&stream);
        let error = err(transport.read(&mut reader));
        assert!(matches!(error, TransportError::FrameTooLarge(_)));
    }

    #[test]
    fn transport_is_generic_over_arbitrary_json_values() {
        let transport = SessionEventTransport::<serde_json::Value>::new();
        let envelope = serde_json::json!({
            "v": 1,
            "kind": "flauz.event",
            "event_id": "ev_01J8ZQ5V8K3T2B7N6X4R9DQPD3",
            "seq": 42,
            "nested": {"deep": [true, 7, "x"]}
        });
        let bytes = ok(transport.encode(&envelope));
        let frame_bytes: &[u8] = bytes.strip_suffix(b"\n").unwrap_or(&bytes);
        let restored = ok(transport.decode(frame_bytes));
        assert_eq!(restored, envelope);
    }
}
