// Copyright (c) 2018-2022 The MobileCoin Foundation

mod autogenerated_code {
    // Expose proto data types from included third-party/external proto files.
    pub use mc_api::{blockchain, external};
    pub use protobuf::well_known_types::Empty;

    // Needed due to how to the auto-generated code references the Empty message.
    pub mod empty {
        pub use protobuf::well_known_types::Empty;
    }

    // Include the auto-generated code.
    include!(concat!(env!("OUT_DIR"), "/protos-auto-gen/mod.rs"));
}

pub use autogenerated_code::*;

use mc_api::ConversionError;
use mc_fog_report_types::{Report, ReportResponse};

// These are needed for tests
impl Eq for report::Report {}
impl Eq for report::ReportResponse {}

impl TryFrom<report::Report> for Report {
    type Error = ConversionError;

    fn try_from(src: report::Report) -> Result<Self, Self::Error> {
        Ok(Self {
            fog_report_id: src.fog_report_id,
            pubkey_expiry: src.pubkey_expiry,
            report: src
                .report
                .as_ref()
                .ok_or(ConversionError::ObjectMissing)?
                .try_into()?,
        })
    }
}

impl From<Report> for report::Report {
    fn from(src: Report) -> Self {
        Self {
            fog_report_id: src.fog_report_id,
            pubkey_expiry: src.pubkey_expiry,
            report: Some((&src.report).into()),
        }
    }
}

impl TryFrom<report::ReportResponse> for ReportResponse {
    type Error = ConversionError;

    fn try_from(src: report::ReportResponse) -> Result<Self, Self::Error> {
        let reports = src
            .reports
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;
        Ok(Self {
            reports,
            chain: src.chain,
            signature: src.signature,
        })
    }
}

impl From<ReportResponse> for report::ReportResponse {
    fn from(src: ReportResponse) -> Self {
        Self {
            reports: src.reports.into_iter().map(Into::into).collect(),
            chain: src.chain,
            signature: src.signature,
        }
    }
}
