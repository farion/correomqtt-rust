use crate::runtime::WasmSandboxLimits;
use crate::{
    DetailByteTransformRequest, DetailByteTransformResponse, DetailFormatterRequest,
    DetailFormatterResponse, HookDispatchError, HookKind, IncomingMessageTransformRequest,
    IncomingMessageTransformResponse, MessageValidatorRequest, MessageValidatorResponse,
    OutgoingMessageTransformRequest, OutgoingMessageTransformResponse, VersionedDto, ABI_VERSION,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub enum HookInvocation {
    OutgoingMessageTransform(OutgoingMessageTransformRequest),
    IncomingMessageTransform(IncomingMessageTransformRequest),
    MessageValidator(MessageValidatorRequest),
    DetailByteTransform(DetailByteTransformRequest),
    DetailFormatter(DetailFormatterRequest),
}

impl HookInvocation {
    pub fn hook(&self) -> HookKind {
        match self {
            Self::OutgoingMessageTransform(_) => HookKind::OutgoingMessageTransform,
            Self::IncomingMessageTransform(_) => HookKind::IncomingMessageTransform,
            Self::MessageValidator(_) => HookKind::MessageValidator,
            Self::DetailByteTransform(_) => HookKind::DetailByteTransform,
            Self::DetailFormatter(_) => HookKind::DetailFormatter,
        }
    }

    pub(crate) fn to_request_bytes(
        &self,
        limits: &WasmSandboxLimits,
    ) -> Result<Vec<u8>, HookDispatchError> {
        let bytes = match self {
            Self::OutgoingMessageTransform(request) => serialize_request(request, self.hook())?,
            Self::IncomingMessageTransform(request) => serialize_request(request, self.hook())?,
            Self::MessageValidator(request) => serialize_request(request, self.hook())?,
            Self::DetailByteTransform(request) => serialize_request(request, self.hook())?,
            Self::DetailFormatter(request) => serialize_request(request, self.hook())?,
        };
        ensure_payload_size(bytes.len(), limits.max_payload_bytes, self.hook())?;
        Ok(bytes)
    }

    pub(crate) fn parse_output(
        self,
        bytes: Vec<u8>,
        limits: &WasmSandboxLimits,
    ) -> Result<HookOutput, HookDispatchError> {
        ensure_payload_size(bytes.len(), limits.max_payload_bytes, self.hook())?;
        match self {
            Self::OutgoingMessageTransform(_) => {
                parse_response::<OutgoingMessageTransformResponse>(&bytes, self.hook())
                    .map(HookOutput::OutgoingMessageTransform)
            }
            Self::IncomingMessageTransform(_) => {
                parse_response::<IncomingMessageTransformResponse>(&bytes, self.hook())
                    .map(HookOutput::IncomingMessageTransform)
            }
            Self::MessageValidator(_) => {
                parse_response::<MessageValidatorResponse>(&bytes, self.hook())
                    .map(HookOutput::MessageValidator)
            }
            Self::DetailByteTransform(_) => {
                parse_response::<DetailByteTransformResponse>(&bytes, self.hook())
                    .map(HookOutput::DetailByteTransform)
            }
            Self::DetailFormatter(_) => {
                parse_response::<DetailFormatterResponse>(&bytes, self.hook())
                    .map(HookOutput::DetailFormatter)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HookOutput {
    OutgoingMessageTransform(OutgoingMessageTransformResponse),
    IncomingMessageTransform(IncomingMessageTransformResponse),
    MessageValidator(MessageValidatorResponse),
    DetailByteTransform(DetailByteTransformResponse),
    DetailFormatter(DetailFormatterResponse),
}

impl HookOutput {
    pub fn hook(&self) -> HookKind {
        match self {
            Self::OutgoingMessageTransform(_) => HookKind::OutgoingMessageTransform,
            Self::IncomingMessageTransform(_) => HookKind::IncomingMessageTransform,
            Self::MessageValidator(_) => HookKind::MessageValidator,
            Self::DetailByteTransform(_) => HookKind::DetailByteTransform,
            Self::DetailFormatter(_) => HookKind::DetailFormatter,
        }
    }
}

fn serialize_request<T: Serialize>(
    value: &T,
    hook: HookKind,
) -> Result<Vec<u8>, HookDispatchError> {
    serde_json::to_vec(value).map_err(|source| HookDispatchError::SerializeRequest { hook, source })
}

fn parse_response<T>(bytes: &[u8], hook: HookKind) -> Result<T, HookDispatchError>
where
    T: DeserializeOwned + VersionedDto,
{
    let text = String::from_utf8(bytes.to_vec())
        .map_err(|source| HookDispatchError::ResponseUtf8 { hook, source })?;
    let response = serde_json::from_str::<T>(&text)
        .map_err(|source| HookDispatchError::DecodeResponse { hook, source })?;
    if response.abi_version() == ABI_VERSION {
        Ok(response)
    } else {
        Err(HookDispatchError::AbiVersionMismatch {
            hook,
            found: response.abi_version(),
            expected: ABI_VERSION,
        })
    }
}

fn ensure_payload_size(
    actual: usize,
    limit: usize,
    hook: HookKind,
) -> Result<(), HookDispatchError> {
    if actual > limit {
        Err(HookDispatchError::PayloadTooLarge {
            hook,
            actual,
            limit,
        })
    } else {
        Ok(())
    }
}
