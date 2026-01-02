use std::cmp::min;

use smallvec::SmallVec;

use crate::{
    model::{OrderSide, ProcessOrder, Trade, TradeId},
    policies::PolicyChecker,
    storage::{BookSide, PriceKey},
};

pub struct Matcher;

impl Matcher {
    pub fn hard_match<K: PriceKey>(
        aggressor: &mut ProcessOrder,
        book: &mut BookSide<K>,
        next_trade_id: &mut TradeId,
    ) -> SmallVec<[Trade; 16]> {
        let mut executed_trades: SmallVec<[Trade; 16]> = SmallVec::new();
        while aggressor.amount > 0 {
            let maker_order_ref = match book.peek_best() {
                Some(o) => o,
                None => break,
            };

            let is_match = PolicyChecker::check_price_match(
                aggressor.side,
                maker_order_ref.price,
                aggressor.price,
                aggressor.is_market,
            );

            if !is_match {
                break;
            }

            let mut maker_order = match book.pop_best() {
                Some(o) => o,
                None => break,
            };
            let trade_amount = min(aggressor.amount, maker_order.amount);

            let trade = Trade {
                trade_id: *next_trade_id,
                maker_order_id: maker_order.order_id,
                taker_order_id: aggressor.order_id,
                amount: trade_amount,
                buyer_id: match aggressor.side {
                    OrderSide::Buy => aggressor.user_id,
                    OrderSide::Sell => maker_order.user_id,
                },
                seller_id: match aggressor.side {
                    OrderSide::Buy => maker_order.user_id,
                    OrderSide::Sell => aggressor.user_id,
                },
                price: maker_order.price,
            };
            *next_trade_id += 1;

            executed_trades.push(trade);

            aggressor.amount -= trade_amount;
            maker_order.amount -= trade_amount;

            if maker_order.amount > 0 {
                book.insert(maker_order);
            }
        }
        executed_trades
    }
}
