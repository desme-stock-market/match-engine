use std::{cmp::Reverse, collections::BTreeMap};

use rustc_hash::FxHashMap;

use crate::model::{Amount, BookOrder, OrderId, Price};

pub trait PriceKey: Ord + Copy + Clone {
    fn from_price(price: Price) -> Self;
    fn as_price(&self) -> Price;
}

impl PriceKey for Price {
    fn as_price(&self) -> Price {
        *self
    }
    fn from_price(price: Price) -> Self {
        price
    }
}

impl PriceKey for Reverse<Price> {
    fn as_price(&self) -> Price {
        self.0
    }
    fn from_price(price: Price) -> Self {
        Reverse(price)
    }
}

pub struct BookSide<K: PriceKey> {
    orders: BTreeMap<(K, OrderId), BookOrder>,
    liquidity_index: LiquidityIndex<K>,
    index: OrderIndex,
}

impl<K: PriceKey> BookSide<K> {
    pub fn new() -> Self {
        BookSide {
            orders: BTreeMap::new(),
            liquidity_index: LiquidityIndex::new(),
            index: OrderIndex::new(),
        }
    }

    pub fn insert(&mut self, order: BookOrder) {
        let key: (K, OrderId) = (K::from_price(order.price), order.order_id);
        self.index.insert(order.order_id, order.price);
        self.liquidity_index
            .add_liquidity(K::from_price(order.price), order.amount);
        self.orders.insert(key, order);
    }

    pub fn remove(&mut self, order_id: OrderId) -> Option<BookOrder> {
        let key = (K::from_price(*self.index.get(order_id)), order_id);
        let order = self.orders.remove(&key)?;
        self.index.remove(order_id);
        self.liquidity_index
            .remove_liquidity(K::from_price(order.price), order.amount);
        Some(order)
    }

    pub fn best_price(&self) -> Option<Price> {
        self.orders
            .keys()
            .next()
            .map(|(price_key, _)| price_key.as_price())
    }

    pub fn peek_best(&self) -> Option<&BookOrder> {
        self.orders.values().next()
    }

    pub fn pop_best(&mut self) -> Option<BookOrder> {
        let (_, order) = self.orders.pop_first()?;
        self.liquidity_index
            .remove_liquidity(K::from_price(order.price), order.amount);
        self.index.remove(order.order_id);
        Some(order)
    }

    pub fn get_liquidity(&self, price: Price) -> Amount {
        self.liquidity_index.get_liquidity(K::from_price(price))
    }

    pub fn iter(&self) -> impl Iterator<Item = &BookOrder> {
        self.orders.values()
    }
}

struct LiquidityIndex<K: PriceKey> {
    index: BTreeMap<K, Amount>,
}

impl<K: PriceKey> LiquidityIndex<K> {
    pub fn new() -> Self {
        LiquidityIndex {
            index: BTreeMap::new(),
        }
    }

    pub fn add_liquidity(&mut self, price: K, amount: Amount) {
        self.index
            .entry(price)
            .and_modify(|a| *a += amount)
            .or_insert(amount);
    }

    pub fn remove_liquidity(&mut self, price: K, amount: Amount) {
        if let Some(level) = self.index.get_mut(&price) {
            *level -= amount;
            if *level == 0 {
                self.index.remove(&price);
            }
        }
    }

    pub fn get_liquidity(&self, price: K) -> Amount {
        self.index.range(..=price).map(|(_, amount)| amount).sum()
    }
}

struct OrderIndex {
    index: FxHashMap<OrderId, Price>,
}

impl OrderIndex {
    pub fn new() -> Self {
        OrderIndex {
            index: FxHashMap::default(),
        }
    }
    pub fn insert(&mut self, order_id: OrderId, price: Price) {
        self.index.insert(order_id, price);
    }
    pub fn remove(&mut self, order_id: OrderId) {
        self.index.remove(&order_id);
    }
    pub fn get(&self, order_id: OrderId) -> &Price {
        self.index.get(&order_id).unwrap()
    }
}
