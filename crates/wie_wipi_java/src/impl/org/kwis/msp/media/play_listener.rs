use alloc::vec;

use crate::base::JavaClassProto;

// interface org.kwis.msp.media.PlayListener
pub struct PlayListener {}

impl PlayListener {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: "",
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
