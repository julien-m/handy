//! Facade for private Unix resources, sockets and the settings startup lease.

#![cfg(unix)]

#[path = "platform_unix/resources.rs"]
mod resources;
#[path = "platform_unix/socket.rs"]
mod socket;
#[path = "platform_unix/startup_fence.rs"]
mod startup_fence;

pub(crate) use resources::{current_uid, PrivateDirectory};
pub(crate) use socket::{verify_peer, verify_peer_uid, PrivateSocket};
pub(crate) use startup_fence::StartupFence;

#[cfg(test)]
#[path = "platform_unix/tests.rs"]
mod tests;
