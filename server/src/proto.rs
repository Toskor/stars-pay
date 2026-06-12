//! Generated protobuf types + helpers for the WebSocket wire protocol.
//!
//! See `proto/events.proto` for the schema. Servers encode a `ServerMessage`
//! and push it to overlays as a binary WS frame.

#![allow(clippy::all)]
include!(concat!(env!("OUT_DIR"), "/tg_stars.v1.rs"));

use prost::Message;

use crate::json;

/// Serialize a `ServerMessage` to a byte vector ready for `Frame::binary`.
pub fn encode(msg: &ServerMessage) -> Vec<u8> {
    let mut buf = Vec::with_capacity(msg.encoded_len());
    // encode() to a Vec<u8> only fails if it can't grow the buffer, which
    // can't happen for an owned Vec we just reserved capacity for.
    msg.encode(&mut buf).expect("Vec write is infallible");
    buf
}

impl From<&json::WSEvent> for ServerMessage {
    fn from(event: &json::WSEvent) -> Self {
        use server_message::Payload;
        let payload = match event {
            json::WSEvent::Success(success) => match &success.data {
                json::WSEventData::Donation {
                    from,
                    total_amount,
                    invoice_payload,
                    message,
                } => Payload::Donation(DonationEvent {
                    from: from.clone(),
                    total_amount: *total_amount,
                    invoice_payload: invoice_payload.clone(),
                    message: message.clone(),
                }),
                json::WSEventData::GoalProps { props } => Payload::Goal(GoalEvent {
                    props_json: serde_json::to_string(props.as_ref()).unwrap_or_default(),
                }),
            },
            json::WSEvent::Error { error, .. } => Payload::Error(ErrorEvent {
                message: error.clone(),
            }),
        };
        ServerMessage {
            payload: Some(payload),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn donation_event_round_trips() {
        let event = json::WSEvent::Success(Box::new(json::WSEventSuccess {
            ok: true,
            data: json::WSEventData::Donation {
                from: "alice".into(),
                total_amount: 42,
                invoice_payload: "paymentFor:bot_x".into(),
                message: "ty".into(),
            },
        }));
        let msg = ServerMessage::from(&event);
        let bytes = encode(&msg);
        let decoded = ServerMessage::decode(bytes.as_slice()).unwrap();
        match decoded.payload.unwrap() {
            server_message::Payload::Donation(d) => {
                assert_eq!(d.from, "alice");
                assert_eq!(d.total_amount, 42);
                assert_eq!(d.invoice_payload, "paymentFor:bot_x");
                assert_eq!(d.message, "ty");
            }
            other => panic!("expected Donation, got {other:?}"),
        }
    }

    #[test]
    fn error_event_round_trips() {
        let event = json::WSEvent::Error {
            ok: false,
            error: "boom".into(),
        };
        let bytes = encode(&ServerMessage::from(&event));
        let decoded = ServerMessage::decode(bytes.as_slice()).unwrap();
        match decoded.payload.unwrap() {
            server_message::Payload::Error(e) => assert_eq!(e.message, "boom"),
            other => panic!("expected Error, got {other:?}"),
        }
    }
}
