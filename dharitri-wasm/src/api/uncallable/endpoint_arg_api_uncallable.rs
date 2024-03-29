use crate::api::{endpoint_arg_api::EndpointArgumentApiImpl, EndpointArgumentApi, Handle};

use super::UncallableApi;

impl EndpointArgumentApi for UncallableApi {
    type EndpointArgumentApiImpl = UncallableApi;

    fn argument_api_impl() -> Self::EndpointArgumentApiImpl {
        unreachable!()
    }
}

impl EndpointArgumentApiImpl for UncallableApi {
    fn get_num_arguments(&self) -> i32 {
        unreachable!()
    }

    fn load_argument_managed_buffer(&self, _arg_id: i32, _dest: Handle) {
        unreachable!()
    }
}
