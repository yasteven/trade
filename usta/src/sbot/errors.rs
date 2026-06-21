
// trade/usta/src/sbot/errors.rs

use std::fmt;

/// Core variants shared by all bots.
/// Add bot-specific variants later if truly needed (rare for most bots).
#[derive(Debug, Clone)]
pub enum BotError 
{
  ChannelClosed,
  Timeout(String),
  OrderSendFailed(String),
  InvalidStateTransition(String),
  MissingData(String),
  LogicError(String),
  IoError(String),
  JsonError(String),
  Other(String),
}

impl fmt::Display for BotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BotError::ChannelClosed => write!(f, "channel closed"),
            BotError::Timeout(msg) => write!(f, "operation timed out: {}", msg),
            BotError::OrderSendFailed(msg) => write!(f, "failed to send order: {}", msg),
            BotError::InvalidStateTransition(msg) => write!(f, "invalid state transition: {}", msg),
            BotError::MissingData(msg) => write!(f, "missing required data: {}", msg),
            BotError::LogicError(msg) => write!(f, "logic error: {}", msg),
            BotError::IoError(e) => write!(f, "I/O error: {}", e),
            BotError::JsonError(e) => write!(f, "JSON error: {}", e),
            BotError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for BotError {}

/// Convenience alias used by almost all bots
pub type BotResult<T = ()> = Result<T, BotError>;