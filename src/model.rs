use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type Price = Decimal;
pub type OrderId = u64;
pub type TradeId = u64;
pub type Amount = u64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC, // Good Till Cancelled
    IOC, // Immediate Or Cancel
    FOK, // Fill Or Kill
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit {
        post_only: bool,
        price: Price,
        tif: TimeInForce,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CancelReason {
    UserRequest,
    IocExpired,
    FokLiquidityShortage,
}

impl std::fmt::Display for CancelReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CancelReason::UserRequest => write!(f, "UserRequest"),
            CancelReason::IocExpired => write!(f, "IocExpired"),
            CancelReason::FokLiquidityShortage => write!(f, "FokLiquidityShortage"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RejectReason {
    PostOnlyViolation,
    InvalidPrice,
    InvalidAmount,
    SymbolNotFound,
}

impl std::fmt::Display for RejectReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RejectReason::PostOnlyViolation => write!(f, "PostOnlyViolation"),
            RejectReason::InvalidPrice => write!(f, "InvalidPrice"),
            RejectReason::InvalidAmount => write!(f, "InvalidAmount"),
            RejectReason::SymbolNotFound => write!(f, "SymbolNotFound"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingOrder {
    pub order_id: OrderId,
    pub user_id: Uuid,
    pub side: OrderSide,
    pub amount: Amount,

    pub order_type: OrderType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessOrder {
    pub order_id: OrderId,
    pub user_id: Uuid,
    pub side: OrderSide,
    pub amount: Amount,

    pub price: Price,
    pub post_only: bool,
    pub is_market: bool,
    pub tif: TimeInForce,
}

impl From<IncomingOrder> for ProcessOrder {
    fn from(order: IncomingOrder) -> Self {
        ProcessOrder {
            order_id: order.order_id,
            user_id: order.user_id,
            side: order.side,
            amount: order.amount,
            price: match order.order_type {
                OrderType::Market { .. } => Price::from(0),
                OrderType::Limit { price, .. } => price,
            },
            is_market: match order.order_type {
                OrderType::Market { .. } => true,
                OrderType::Limit { .. } => false,
            },
            post_only: match order.order_type {
                OrderType::Market { .. } => false,
                OrderType::Limit { post_only, .. } => post_only,
            },
            tif: match order.order_type {
                OrderType::Market { .. } => TimeInForce::GTC,
                OrderType::Limit { tif, .. } => tif,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookOrder {
    pub order_id: OrderId,
    pub user_id: Uuid,
    pub price: Price,
    pub amount: Amount,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Trade {
    pub amount: Amount,
    pub buyer_id: Uuid,
    pub price: Price,
    pub seller_id: Uuid,
    pub trade_id: TradeId,
    pub maker_order_id: OrderId,
    pub taker_order_id: OrderId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineEvent {
    OrderPlaced {
        order: BookOrder,
        side: OrderSide,
    },
    TradeExecuted(Trade),
    OrderCancelled {
        order_id: OrderId,
        remaining_amount: Amount,
        reason: CancelReason,
    },
    OrderRejected {
        order_id: OrderId,
        reason: RejectReason,
    },
}
