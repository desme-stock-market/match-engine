use std::cmp::Reverse;

use smallvec::{SmallVec, smallvec};

use crate::{
    matcher::Matcher,
    model::{
        BookOrder, CancelReason, EngineEvent, IncomingOrder, OrderSide, OrderType, Price,
        ProcessOrder, TimeInForce,
    },
    policies::PolicyChecker,
    storage::BookSide,
};

pub struct MatchEngine {
    asks: BookSide<Price>,
    bids: BookSide<Reverse<Price>>,
    next_trade_id: TradeId,
}

impl MatchEngine {
    pub fn new() -> Self {
        MatchEngine {
            asks: BookSide::new(),
            bids: BookSide::new(),
            next_trade_id: 0,
        }
    }
    pub fn process(&mut self, order: IncomingOrder) -> SmallVec<[EngineEvent; 16]> {
        match order.order_type {
            OrderType::Market { .. } => self.handle_market(&mut ProcessOrder::from(order)),
            OrderType::Limit { .. } => self.handle_limit(&mut ProcessOrder::from(order)),
        }
    }

    fn handle_market(&mut self, mut order: &mut ProcessOrder) -> SmallVec<[EngineEvent; 16]> {
        match PolicyChecker::check_post_only(order) {
            Ok(_) => (),
            Err(event) => return smallvec![event],
        }
        match order.side {
            OrderSide::Buy => Matcher::hard_match(&mut order, &mut self.asks, &mut self.next_trade_id),
            OrderSide::Sell => Matcher::hard_match(&mut order, &mut self.bids, &mut self.next_trade_id),
        }
        .iter()
        .map(|trade| EngineEvent::TradeExecuted(*trade))
        .collect()
    }

    fn handle_limit(&mut self, mut order: &mut ProcessOrder) -> SmallVec<[EngineEvent; 16]> {
        match PolicyChecker::check_post_only(order) {
            Ok(_) => (),
            Err(event) => return smallvec![event],
        };
        let liquidity_check_result = match order.side {
            OrderSide::Buy => PolicyChecker::check_liquidity(order, &mut self.asks),
            OrderSide::Sell => PolicyChecker::check_liquidity(order, &mut self.bids),
        };
        match liquidity_check_result {
            Ok(_) => (),
            Err(event) => return smallvec![event],
        };
        let mut executed_events: SmallVec<[EngineEvent; 16]> = match order.side {
            OrderSide::Buy => Matcher::hard_match(&mut order, &mut self.asks, &mut self.next_trade_id),
            OrderSide::Sell => Matcher::hard_match(&mut order, &mut self.bids, &mut self.next_trade_id),
        }
        .iter()
        .map(|trade| EngineEvent::TradeExecuted(*trade))
        .collect();
        if order.amount > 0 {
            match order.tif {
                TimeInForce::GTC => {
                    let book_order = BookOrder {
                        user_id: order.user_id,
                        order_id: order.order_id,
                        price: order.price,
                        amount: order.amount,
                    };
                    match order.side {
                        OrderSide::Buy => self.bids.insert(book_order.clone()),
                        OrderSide::Sell => self.asks.insert(book_order.clone()),
                    };
                    executed_events.push(EngineEvent::OrderPlaced {
                        order: book_order,
                        side: order.side,
                    });
                }
                TimeInForce::IOC => {
                    executed_events.push(EngineEvent::OrderCancelled {
                        order_id: order.order_id,
                        remaining_amount: order.amount,
                        reason: CancelReason::IocExpired,
                    });
                }
                TimeInForce::FOK => (),
            }
        }
        executed_events
    }
}
