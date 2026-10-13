//! Smart paste (007) boundary tests driven through the public module API
//! with scripted native ports and a virtual clock.

mod smart_paste {
    pub mod direct;
    pub mod fakes;
    pub mod keyboard;
    pub mod lifecycle;
    pub mod snapshot;
}
