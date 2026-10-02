use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use stellar_xdr::{Limits, ReadXdr, ScVal};
use tokio::sync::watch;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SorobanTopicFilter {
    #[serde(rename = "type")]
    pub event_type: String,
    pub contract_ids: Vec<String>,
    pub topics: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SorobanRawEventData {
    #[serde(rename = "type")]
    pub event_type: String,
    pub ledger: u32,
    pub ledger_closed_at: String,
    pub contract_id: String,
    pub id: String,
    pub paging_token: String,
    pub topic: Vec<String>,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SorobanDomainEvent {
    PlanCreated { plan_id: u64, owner: String },
    PlanPinged { plan_id: u64, timestamp: u64 },
    PlanTriggered { plan_id: u64, caller: String },
    PlanClaimed { plan_id: u64, beneficiary: String },
    Unknown { topic_name: String },
}

pub struct SorobanEventParser;

impl SorobanEventParser {
    pub fn parse_raw_event(raw: &SorobanRawEventData) -> Option<SorobanDomainEvent> {
        let first_topic = raw.topic.first()?;
        let scval = ScVal::from_xdr_base64(first_topic, Limits::none()).ok()?;

        let topic_name = match scval {
            ScVal::Symbol(sym) => String::from_utf8_lossy(sym.0.as_vec()).to_string(),
            _ => return None,
        };

        match topic_name.as_str() {
            "plan_created" => {
                let plan_id = raw
                    .topic
                    .get(1)
                    .and_then(|t| ScVal::from_xdr_base64(t, Limits::none()).ok())
                    .and_then(|v| match v {
                        ScVal::U64(id) => Some(id),
                        _ => None,
                    })
                    .unwrap_or(0);
                Some(SorobanDomainEvent::PlanCreated {
                    plan_id,
                    owner: raw.contract_id.clone(),
                })
            }
            "plan_ping" => {
                let plan_id = raw
                    .topic
                    .get(1)
                    .and_then(|t| ScVal::from_xdr_base64(t, Limits::none()).ok())
                    .and_then(|v| match v {
                        ScVal::U64(id) => Some(id),
                        _ => None,
                    })
                    .unwrap_or(0);
                Some(SorobanDomainEvent::PlanPinged {
                    plan_id,
                    timestamp: raw.ledger as u64,
                })
            }
            "plan_triggered" => {
                let plan_id = raw
                    .topic
                    .get(1)
                    .and_then(|t| ScVal::from_xdr_base64(t, Limits::none()).ok())
                    .and_then(|v| match v {
                        ScVal::U64(id) => Some(id),
                        _ => None,
                    })
                    .unwrap_or(0);
                Some(SorobanDomainEvent::PlanTriggered {
                    plan_id,
                    caller: raw.contract_id.clone(),
                })
            }
            "plan_claimed" => {
                let plan_id = raw
                    .topic
                    .get(1)
                    .and_then(|t| ScVal::from_xdr_base64(t, Limits::none()).ok())
                    .and_then(|v| match v {
                        ScVal::U64(id) => Some(id),
                        _ => None,
                    })
                    .unwrap_or(0);
                Some(SorobanDomainEvent::PlanClaimed {
                    plan_id,
                    beneficiary: raw.contract_id.clone(),
                })
            }
            other => Some(SorobanDomainEvent::Unknown {
                topic_name: other.to_string(),
            }),
        }
    }
}

pub struct SorobanEventIndexerService {
    rpc_url: String,
    contract_id: String,
    poll_interval: Duration,
}

impl SorobanEventIndexerService {
    pub fn new(rpc_url: String, contract_id: String, poll_interval: Duration) -> Self {
        Self {
            rpc_url,
            contract_id,
            poll_interval,
        }
    }

    pub fn start(self: Arc<Self>, mut shutdown_rx: watch::Receiver<bool>) {
        tokio::spawn(async move {
            info!(
                "Starting real-time Soroban contract event indexer service for {}",
                self.contract_id
            );
            let mut interval = tokio::time::interval(self.poll_interval);
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Poll Soroban getEvents and parse contract topics for state sync
                    }
                    _ = shutdown_rx.changed() => {
                        info!("Soroban event indexer service shutting down");
                        break;
                    }
                }
            }
        });
    }
}
