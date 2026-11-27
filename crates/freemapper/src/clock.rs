// SPDX-License-Identifier: AGPL-3.0-or-later

//! Moved to `freemapper-adapter-api` as `clock` (ADR 0082), so that an adapter outside the engine
//! asks the same clock; this module keeps the old paths working.

pub use freemapper_adapter_api::clock::*;
