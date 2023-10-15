pub const ADKG_PROP: usize = 11;
pub const ADKG_SIG: usize = 12;
pub const VABA_ATTACH: usize = 21;
pub const VABA_SIG: usize = 22;
pub const VABA_INDICE: usize = 23;
pub const VABA_EVAL: usize = 24;
pub const VABA_FIN: usize = 25;
pub const GATHER_1: usize = 31;
pub const GATHER_2: usize = 32;
pub const GATHER_3: usize = 33;
pub const GATHER_FIN: usize = 34;

#[derive(Clone, Debug)]
pub struct Message {
    pub sender_id: usize,
    pub receiver_id: Vec<usize>,
    pub msg_type: usize,
    pub msg_content: Vec<usize>,
    pub additional: String,
}

impl Message {
    pub fn new(id: usize) -> Message {
        Message {
            sender_id: id,
            receiver_id: Vec::new(),
            msg_type: 0,
            msg_content: Vec::new(),
            additional: String::new(),
        }
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut receiver_id = String::new();
        for i in &self.receiver_id {
            receiver_id.push_str(&i.to_string());
            receiver_id.push_str(" ");
        }
        let mut msg_content = String::new();
        for i in &self.msg_content {
            msg_content.push_str(&i.to_string());
            msg_content.push_str(" ");
        }
        write!(f, "sender_id: {}, receiver_id: {}, msg_type: {} \n  >>> msg_content: {} \n  >>> additional: {}",
               self.sender_id, receiver_id, self.msg_type, msg_content, self.additional)
    }
}
