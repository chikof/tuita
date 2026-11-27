//! Direct messages and end-to-end encryption.
//!
//! Every page is plain server-rendered markup. Nothing in here is hydrated by Topcoat: the lists
//! and threads change after load (new messages, older pages, opened ciphertexts), so `messages.js`
//! and `e2ee.js` drive them through `data-*` hooks, as the post islands do.

pub(crate) mod bubbles;
pub(crate) mod chat_footer;
pub(crate) mod chat_note;
pub(crate) mod chat_thread;
pub(crate) mod conversation_row;
pub(crate) mod e2ee_notice;
pub(crate) mod e2ee_prompt;
pub(crate) mod encryption;
pub(crate) mod fragments;
pub(crate) mod frame;
pub(crate) mod globals;
pub(crate) mod people;
pub(crate) mod safety_dialog;
pub(crate) mod thread;
