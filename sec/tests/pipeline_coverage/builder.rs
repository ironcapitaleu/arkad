use std::pin::pin;

use futures_util::StreamExt;
use sec::implementations::states::extract::ExtractSuperState;
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use sec::prelude::*;
use sec::shared::http_client::implementations::sec_client::SecClient;
use uuid::Uuid;

/// Marker type indicating no CIK has been provided yet.
pub struct NoCik;
/// Marker type indicating no [`SecClient`] has been provided yet.
pub struct NoClient;

/// Typestate builder for constructing a [`Pipeline`].
pub struct PipelineBuilder<C, S> {
    cik: C,
    sec_client: S,
}

impl PipelineBuilder<NoCik, NoClient> {
    pub(super) const fn new() -> Self {
        Self {
            cik: NoCik,
            sec_client: NoClient,
        }
    }
}

impl<S> PipelineBuilder<NoCik, S> {
    pub fn cik(self, cik: impl Into<String>) -> PipelineBuilder<String, S> {
        PipelineBuilder {
            cik: cik.into(),
            sec_client: self.sec_client,
        }
    }
}

impl<C> PipelineBuilder<C, NoClient> {
    pub fn sec_client(self, sec_client: SecClient) -> PipelineBuilder<C, SecClient> {
        PipelineBuilder {
            cik: self.cik,
            sec_client,
        }
    }
}

impl PipelineBuilder<String, SecClient> {
    pub fn build(self) -> Pipeline {
        Pipeline {
            raw_cik: self.cik,
            sec_client: self.sec_client,
        }
    }
}

/// A full Extract + Transform pipeline for a single CIK.
pub struct Pipeline {
    raw_cik: String,
    sec_client: SecClient,
}

impl Pipeline {
    pub const fn builder() -> PipelineBuilder<NoCik, NoClient> {
        PipelineBuilder::new()
    }

    pub async fn run(self) -> Result<(), StreamError> {
        let execution_id = Uuid::new_v4();
        let state = ExtractSuperState::<ValidateCikFormat>::new(self.raw_cik, self.sec_client);
        let mut stream = pin!(state.into_stream(execution_id));

        while let Some(result) = stream.next().await {
            if let Err(e) = result {
                return Err(e);
            }
        }

        Ok(())
    }
}
