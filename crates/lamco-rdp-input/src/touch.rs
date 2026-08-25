//! Touch Event Handling
//!
//! Tracks per-contact touch state (MS-RDPEI semantics: down/update/up,
//! in-range, in-contact, canceled) and turns each wire contact update into
//! at most one host-facing touch event, with coordinate transformation.

use crate::coordinates::CoordinateTransformer;
use crate::error::Result;
use tracing::{debug, warn};

/// Maximum simultaneous contacts. MS-RDPEI's `contactId` is a wire `u8`
/// (0-255), so this is a hard protocol ceiling, not a tuning choice.
const MAX_CONTACTS: usize = 256;

/// A touch event ready for host injection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TouchEvent {
    /// A new contact touched down at a stream position.
    Down {
        /// Contact slot (same as the wire `contactId`, widened).
        slot: u32,
        /// Stream X coordinate.
        x: f64,
        /// Stream Y coordinate.
        y: f64,
    },
    /// An engaged contact moved to a stream position.
    Motion {
        /// Contact slot.
        slot: u32,
        /// Stream X coordinate.
        x: f64,
        /// Stream Y coordinate.
        y: f64,
    },
    /// A contact lifted off.
    Up {
        /// Contact slot.
        slot: u32,
    },
}

/// The state a single contact is in, mirroring MS-RDPEI's own model
/// (§ 3.1.1.1): a contact starts out of range, may hover in range without
/// touching, then engages on contact, and may return to hovering after
/// lifting (the digitizer can keep tracking a finger just above the
/// surface) before finally leaving range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContactPhase {
    OutOfRange,
    Hovering,
    Engaged,
}

#[derive(Debug, Clone, Copy)]
struct ContactState {
    phase: ContactPhase,
    /// Set when this contact's position couldn't be mapped to any stream
    /// (e.g. outside all configured monitors). The state machine keeps
    /// running normally so `contact_id` bookkeeping stays correct; only the
    /// host-facing event is suppressed while this is set.
    ignore: bool,
}

impl Default for ContactState {
    fn default() -> Self {
        Self {
            phase: ContactPhase::OutOfRange,
            ignore: false,
        }
    }
}

/// The MS-RDPEI contact flags relevant to a single contact update,
/// decoded from the wire `contactFlags` bit field (MS-RDPEI § 2.2.3.3.1.1).
/// Kept as plain booleans rather than depending on an IronRDP crate type,
/// matching how [`crate::mouse::MouseButton::from_rdp_button`] takes raw
/// wire values rather than a foreign PDU type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TouchContactFlags {
    pub down: bool,
    pub update: bool,
    pub up: bool,
    pub in_range: bool,
    pub in_contact: bool,
    pub canceled: bool,
}

/// Tracks touch contact state and turns wire contact updates into host
/// touch events.
pub struct TouchHandler {
    contacts: Box<[ContactState; MAX_CONTACTS]>,
}

impl Default for TouchHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl TouchHandler {
    pub fn new() -> Self {
        Self {
            contacts: Box::new([ContactState::default(); MAX_CONTACTS]),
        }
    }

    /// Process one contact update from an MS-RDPEI touch frame.
    ///
    /// Returns `Ok(None)` for updates that don't produce a host event
    /// (hover-only motion, an illegal flag combination, or a position that
    /// couldn't be mapped to any stream) — none of these are propagated as
    /// errors, since a single bad contact in a multi-touch frame must not
    /// abort processing the rest of the frame's contacts.
    pub fn handle_contact(
        &mut self,
        contact_id: u8,
        x: i32,
        y: i32,
        flags: TouchContactFlags,
        transformer: &mut CoordinateTransformer,
    ) -> Result<Option<TouchEvent>> {
        let slot = u32::from(contact_id);
        let state = &mut self.contacts[contact_id as usize];

        match (flags.down, flags.update, flags.up, flags.in_range, flags.in_contact) {
            // DOWN|INRANGE|INCONTACT: new contact engaging.
            (true, false, false, true, true) => {
                state.phase = ContactPhase::Engaged;
                state.ignore = false;
                Self::transform(state, transformer, x, y).map(|pos| pos.map(|(x, y)| TouchEvent::Down { slot, x, y }))
            }

            // UPDATE|INRANGE|INCONTACT: engaged contact moved.
            (false, true, false, true, true) => {
                if state.phase != ContactPhase::Engaged {
                    warn!(
                        contact_id,
                        "touch UPDATE|INCONTACT for a non-engaged contact, treating as down"
                    );
                    state.phase = ContactPhase::Engaged;
                }
                Self::transform(state, transformer, x, y).map(|pos| pos.map(|(x, y)| TouchEvent::Motion { slot, x, y }))
            }

            // UPDATE|INRANGE (not in contact): hovering, no host event —
            // ei::Touchscreen has no hover primitive.
            (false, true, false, true, false) => {
                if state.phase == ContactPhase::OutOfRange {
                    state.phase = ContactPhase::Hovering;
                }
                Ok(None)
            }

            // UP|INRANGE: lifted but still hovering (digitizer keeps
            // tracking just above the surface) — emit Up, demote rather
            // than fully release.
            (false, false, true, true, false) => {
                let was_engaged = state.phase == ContactPhase::Engaged;
                state.phase = ContactPhase::Hovering;
                Ok(was_engaged.then_some(TouchEvent::Up { slot }))
            }

            // UP, or UP|CANCELED: fully released.
            (false, false, true, false, false) => {
                let was_engaged = state.phase == ContactPhase::Engaged;
                *state = ContactState::default();
                Ok(was_engaged.then_some(TouchEvent::Up { slot }))
            }

            _ => {
                warn!(
                    contact_id,
                    ?flags,
                    "illegal MS-RDPEI touch contact flag combination, ignoring"
                );
                Ok(None)
            }
        }
    }

    /// Reset all contact state (e.g. on client reconnection).
    pub fn reset(&mut self) {
        *self.contacts = [ContactState::default(); MAX_CONTACTS];
    }

    fn transform(
        state: &mut ContactState,
        transformer: &mut CoordinateTransformer,
        x: i32,
        y: i32,
    ) -> Result<Option<(f64, f64)>> {
        match transformer.rdp_to_stream(x, y) {
            Ok((stream_x, stream_y)) => {
                state.ignore = false;
                let (stream_x, stream_y) = transformer.clamp_to_bounds(stream_x, stream_y);
                Ok(Some((stream_x, stream_y)))
            }
            Err(e) => {
                debug!(x, y, error = %e, "touch contact position outside all monitors, suppressing host event");
                state.ignore = true;
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinates::MonitorInfo;

    fn create_test_transformer() -> CoordinateTransformer {
        let monitor = MonitorInfo {
            id: 1,
            name: "Primary".to_string(),
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            dpi: 96.0,
            scale_factor: 1.0,
            stream_x: 0,
            stream_y: 0,
            stream_width: 1920,
            stream_height: 1080,
            is_primary: true,
        };
        CoordinateTransformer::new(vec![monitor]).unwrap()
    }

    fn down_flags() -> TouchContactFlags {
        TouchContactFlags {
            down: true,
            in_range: true,
            in_contact: true,
            ..Default::default()
        }
    }

    fn update_flags() -> TouchContactFlags {
        TouchContactFlags {
            update: true,
            in_range: true,
            in_contact: true,
            ..Default::default()
        }
    }

    fn up_flags() -> TouchContactFlags {
        TouchContactFlags {
            up: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_down_motion_up_sequence() {
        let mut handler = TouchHandler::new();
        let mut transformer = create_test_transformer();

        let event = handler
            .handle_contact(0, 960, 540, down_flags(), &mut transformer)
            .unwrap();
        assert!(matches!(event, Some(TouchEvent::Down { slot: 0, .. })));

        let event = handler
            .handle_contact(0, 970, 540, update_flags(), &mut transformer)
            .unwrap();
        assert!(matches!(event, Some(TouchEvent::Motion { slot: 0, .. })));

        let event = handler
            .handle_contact(0, 970, 540, up_flags(), &mut transformer)
            .unwrap();
        assert_eq!(event, Some(TouchEvent::Up { slot: 0 }));
    }

    #[test]
    fn test_multiple_contacts_independent_slots() {
        let mut handler = TouchHandler::new();
        let mut transformer = create_test_transformer();

        let a = handler
            .handle_contact(0, 100, 100, down_flags(), &mut transformer)
            .unwrap();
        let b = handler
            .handle_contact(1, 200, 200, down_flags(), &mut transformer)
            .unwrap();

        assert!(matches!(a, Some(TouchEvent::Down { slot: 0, .. })));
        assert!(matches!(b, Some(TouchEvent::Down { slot: 1, .. })));
    }

    #[test]
    fn test_up_with_inrange_demotes_to_hovering_not_full_release() {
        let mut handler = TouchHandler::new();
        let mut transformer = create_test_transformer();

        handler
            .handle_contact(0, 100, 100, down_flags(), &mut transformer)
            .unwrap();

        let hover_up = TouchContactFlags {
            up: true,
            in_range: true,
            ..Default::default()
        };
        let event = handler.handle_contact(0, 100, 100, hover_up, &mut transformer).unwrap();
        assert_eq!(event, Some(TouchEvent::Up { slot: 0 }));

        // A second UP for the same (now-hovering) contact must not emit
        // another Up — it was never re-engaged.
        let event = handler
            .handle_contact(0, 100, 100, up_flags(), &mut transformer)
            .unwrap();
        assert_eq!(event, None);
    }

    #[test]
    fn test_hover_only_produces_no_event() {
        let mut handler = TouchHandler::new();
        let mut transformer = create_test_transformer();

        let hover = TouchContactFlags {
            update: true,
            in_range: true,
            ..Default::default()
        };
        let event = handler.handle_contact(0, 100, 100, hover, &mut transformer).unwrap();
        assert_eq!(event, None);
    }

    #[test]
    fn test_illegal_flag_combination_is_ignored_not_erroring() {
        let mut handler = TouchHandler::new();
        let mut transformer = create_test_transformer();

        // DOWN without INRANGE/INCONTACT is not one of the 8 legal
        // combinations per MS-RDPEI 2.2.3.3.1.1.
        let illegal = TouchContactFlags {
            down: true,
            ..Default::default()
        };
        let event = handler.handle_contact(0, 100, 100, illegal, &mut transformer).unwrap();
        assert_eq!(event, None);
    }

    #[test]
    fn test_out_of_bounds_position_clamps_rather_than_erroring() {
        let mut handler = TouchHandler::new();
        let mut transformer = create_test_transformer();

        // Far outside the single 1920x1080 monitor configured above.
        // CoordinateTransformer falls back to the primary monitor and
        // clamps rather than failing (matching mouse's own behavior), so
        // this still produces a Down event, just clamped to the edge.
        let event = handler
            .handle_contact(0, 50_000, 50_000, down_flags(), &mut transformer)
            .unwrap();
        match event {
            Some(TouchEvent::Down { x, y, .. }) => {
                assert!(x <= 1920.0);
                assert!(y <= 1080.0);
            }
            other => panic!("expected a clamped Down event, got {other:?}"),
        }
    }

    #[test]
    fn test_reset_clears_all_contacts() {
        let mut handler = TouchHandler::new();
        let mut transformer = create_test_transformer();

        handler
            .handle_contact(0, 100, 100, down_flags(), &mut transformer)
            .unwrap();
        handler.reset();

        // After reset, UP for contact 0 should not report it as
        // previously-engaged (no Up event since it was never re-downed).
        let event = handler
            .handle_contact(0, 100, 100, up_flags(), &mut transformer)
            .unwrap();
        assert_eq!(event, None);
    }
}
