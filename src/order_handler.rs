//! A set of functions to handle placing market or limit orders,
//! trigger orders and canceling orders



/// Create a market order on FTX
pub(crate) async fn place_market_order(
    api: &ftx::rest::Rest,
    market_name: &str,
    order_side: ftx::rest::Side,
    order_size: rust_decimal::Decimal) -> bool {

    let order= api.request(ftx::rest::PlaceOrder {
        market: std::string::String::from(market_name),
        side: order_side,
        price: None,
        r#type: ftx::rest::OrderType::Market,
        size: order_size,
        reduce_only: false,
        ioc: false,
        post_only: false,
        client_id: None,
        reject_on_price_band: false,
    }).await;

    let order_success;
    match order {
        Err(e) => {
            log::error!("Unable to place order, Err: {:?}", e);
            order_success = false;
        }
        Ok(o) => {
            log::info!("Order placed successfully: {:?}", o);
            order_success = true;
        }
    };

    return order_success

}


/// Cancel all open orders and trigger orders on FTX
pub(crate) async fn cancel_all_orders(api: &ftx::rest::Rest, market_name: &str) -> ftx::rest::Result<String> {
    return api.request(ftx::rest::CancelAllOrder {
        market: Option::from(String::from(market_name)),
        side: None,
        conditional_orders_only: Option::from(false),
        limit_orders_only: Option::from(false),
    }).await;
}

/// Place take profit and stop loss orders
pub(crate) async fn place_trigger_orders(
    api: &ftx::rest::Rest,
    market_name: &str,
    order_side: ftx::rest::Side,
    order_size: rust_decimal::Decimal,
    tp_price: rust_decimal::Decimal,
    sl_price: rust_decimal::Decimal) -> bool {
    let trigger_side = match order_side {
        ftx::rest::Side::Buy => ftx::rest::Side::Sell,
        ftx::rest::Side::Sell => ftx::rest::Side::Buy,
    };

    let take_profit_success;
    let stop_loss_success;

    let take_profit = api.request(ftx::rest::PlaceTriggerOrder {
        market: String::from(market_name),
        side: trigger_side,
        size: order_size,
        r#type: ftx::rest::OrderType::TakeProfit,
        trigger_price: tp_price,
        reduce_only: Option::from(true),
        retry_until_filled: None,
        order_price: None,
        trail_value: None,
    }).await;

    match take_profit {
        Err(e) => {
            log::error!("Unable to place take profit, Err: {:?}", e);
            take_profit_success = false
        }
        Ok(o) => {
            log::info!("Take profit placed successfully: {:?}", o);
            take_profit_success = true
        }
    };

    let stop_loss = api.request(ftx::rest::PlaceTriggerOrder {
        market: String::from(market_name),
        side: trigger_side,
        size: order_size,
        r#type: ftx::rest::OrderType::Stop,
        trigger_price: sl_price,
        reduce_only: Option::from(true),
        retry_until_filled: None,
        order_price: None,
        trail_value: None,
    }).await;

    match stop_loss {
        Err(e) => {
            log::error!("Unable to place stop loss, Err: {:?}", e);
            stop_loss_success = false
        }
        Ok(o) => {
            log::info!("Stop loss placed successfully: {:?}", o);
            stop_loss_success = true
        }
    };

    return take_profit_success && stop_loss_success;
}


/// Calculate static TP and SL values
pub(crate) fn calculate_tp_and_sl(
    price: rust_decimal::Decimal,
    side: ftx::rest::Side,
    tp_percent: rust_decimal::Decimal,
    sl_percent: rust_decimal::Decimal,
    price_precision: u32) -> (rust_decimal::Decimal, rust_decimal::Decimal) {

    let div = rust_decimal::Decimal::from(100);

    let (tp_price, sl_price) = match side {
        ftx::rest::Side::Buy => {
            (price + price * tp_percent / div, price - price * sl_percent / div)
        }
        ftx::rest::Side::Sell => {
            (price - price * tp_percent / div, price + price * sl_percent / div)
        }
    };
    return (tp_price.round_dp(price_precision), sl_price.round_dp(price_precision));
}
