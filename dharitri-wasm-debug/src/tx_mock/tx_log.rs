use alloc::vec::Vec;
use dharitri_wasm::types::heap::Address;
use denali::model::Checkable;

#[derive(Clone, Debug)]
pub struct TxLog {
    pub address: Address,
    pub endpoint: Vec<u8>,
    pub topics: Vec<Vec<u8>>,
    pub data: Vec<u8>,
}

impl TxLog {
    pub fn denali_check(&self, check_log: &denali::model::CheckLog) -> bool {
        check_log.address.check(self.address.as_bytes())
            && check_log.endpoint.check(self.endpoint.as_slice())
            && check_log.topics.check(self.topics.as_slice())
            && check_log.data.check(self.data.as_slice())
    }
}
