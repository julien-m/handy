//! Smart paste (007) boundary tests driven through the public module API
//! with scripted native ports and a virtual clock.

mod smart_paste {
    pub mod actor;
    pub mod chain;
    pub mod coordinator;
    pub mod direct;
    pub mod fake_pasteboard;
    pub mod fakes;
    pub mod keyboard;
    pub mod lifecycle;
    pub mod pipeline;
    pub mod privacy;
    pub mod restoration_policy;
    pub mod rig;
    pub mod snapshot;
}
