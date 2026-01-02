use anyhow::Result;

use crate::{
    model::{CancelReason, EngineEvent, OrderSide, Price, ProcessOrder, RejectReason, TimeInForce},
    storage::{BookSide, PriceKey},
};

pub struct PolicyChecker;

impl PolicyChecker {
    pub fn check_price_match(
        side: OrderSide,
        maker_price: Price,
        aggressor_price: Price,
        is_market: bool,
    ) -> bool {
        if is_market {
            return true;
        }
        match side {
            OrderSide::Buy => maker_price <= aggressor_price,
            OrderSide::Sell => maker_price >= aggressor_price,
        }
    }

    pub fn check_liquidity<K: PriceKey>(
        order: &ProcessOrder,
        book: &BookSide<K>,
    ) -> Result<(), EngineEvent> {
        if match order.tif {
            TimeInForce::FOK => false,
            TimeInForce::GTC => true,
            TimeInForce::IOC => true,
        } {
            return Ok(());
        }

        if order.amount > book.get_liquidity(order.price) {
            return Err(EngineEvent::OrderCancelled {
                order_id: order.order_id,
                remaining_amount: order.amount,
                reason: CancelReason::FokLiquidityShortage,
            });
        }
        Ok(())
    }

    pub fn check_post_only(order: &ProcessOrder) -> Result<(), EngineEvent> {
        if !order.post_only {
            return Ok(());
        }
        if order.is_market {
            return Err(EngineEvent::OrderRejected {
                order_id: order.order_id,
                reason: RejectReason::PostOnlyViolation,
            });
        }
        if match order.tif {
            TimeInForce::FOK => true,
            TimeInForce::GTC => false,
            TimeInForce::IOC => true,
        } {
            return Err(EngineEvent::OrderRejected {
                order_id: order.order_id,
                reason: RejectReason::PostOnlyViolation,
            });
        }
        Ok(())
    }
}
