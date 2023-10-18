#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageType {
    NonType,
    AkdgAvssFin,
    AdkgProp,
    AdkgSig,
    VabaStart,
    VabaAvssFin,
    VabaAttach,
    VabaSig,
    VabaIndice,
    VabaEval,
    VabaFin,
    GatherStart,
    Gather1,
    Gather2,
    Gather3,
    GatherFin,
    SumAndRec,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub sender_id: usize,
    pub receiver_id: Vec<usize>,
    pub msg_type: MessageType,
    pub msg_content: Vec<usize>,
    pub additional: String,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::NonType => write!(f, "NON_TYPE"),
            MessageType::AkdgAvssFin => write!(f, "AVSS_SEND_FIN"),
            MessageType::AdkgProp => write!(f, "ADKG_PROP"),
            MessageType::AdkgSig => write!(f, "ADKG_SIG"),
            MessageType::VabaStart => write!(f, "VABA_START"),
            MessageType::VabaAvssFin => write!(f, "VABA_SEND_FIN"),
            MessageType::VabaAttach => write!(f, "VABA_ATTACH"),
            MessageType::VabaSig => write!(f, "VABA_SIG"),
            MessageType::VabaIndice => write!(f, "VABA_INDICE"),
            MessageType::VabaEval => write!(f, "VABA_EVAL"),
            MessageType::VabaFin => write!(f, "VABA_FIN"),
            MessageType::GatherStart => write!(f, "GATHER_START"),
            MessageType::Gather1 => write!(f, "GATHER_1"),
            MessageType::Gather2 => write!(f, "GATHER_2"),
            MessageType::Gather3 => write!(f, "GATHER_3"),
            MessageType::GatherFin => write!(f, "GATHER_FIN"),
            MessageType::SumAndRec => write!(f, "SUM_AND_REC"),
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

impl Message {
    pub fn new(id: usize) -> Message {
        Message {
            sender_id: id,
            receiver_id: Vec::new(),
            msg_type: MessageType::NonType,
            msg_content: Vec::new(),
            additional: String::new(),
        }
    }

    pub fn send_message_with_addi(id: usize, recv: Vec<usize>, msg_type: MessageType, msg_content: Vec<usize>, addi: String) -> Message {
        Message {
            sender_id: id,
            receiver_id: recv,
            msg_type,
            msg_content,
            additional: addi,
        }
    }

    pub fn send_message(id: usize, recv: Vec<usize>, msg_type: MessageType, msg_content: Vec<usize>) -> Option<Message>{
        Some(Message { 
            sender_id: id,
            receiver_id: recv,
            msg_type,
            msg_content,
            additional: String::new(),
        })
    }

    pub fn send_message2all(id: usize, msg_type: MessageType, msg_content: Vec<usize>) -> Option<Message>{
        Some(Message { 
            sender_id: id,
            receiver_id: vec![],
            msg_type,
            msg_content,
            additional: String::new(),
        })
    }
}