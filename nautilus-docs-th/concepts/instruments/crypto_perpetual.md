# Crypto Perpetual

`CryptoPerpetual` represents a crypto perpetual futures contract, also known as a
perpetual swap. It has no expiry, tracks a crypto base asset, and settles in a crypto,
stablecoin, or other venue-defined settlement currency.

Examples include `ETHUSDT-PERP.BINANCE`, `BTCUSD.BYBIT`, and `BTC-USD-SWAP.OKX`.

## Fields

| Field                 | Rust type          | Required/default | Notes                                    |
| --------------------- | ------------------ | ---------------- | ---------------------------------------- |
| `instrument_id`       | `InstrumentId`     | Required         | Stored as `id` in Rust.                  |
| `raw_symbol`          | `Symbol`           | Required         | Native venue symbol.                     |
| `base_currency`       | `Currency`         | Required         | Base crypto asset.                       |
| `quote_currency`      | `Currency`         | Required         | Price quote currency.                    |
| `settlement_currency` | `Currency`         | Required         | Currency used to settle PnL and fees.    |
| `is_inverse`          | `bool`             | Required         | True when sizing/costing is inverse.     |
| `price_precision`     | `u8`               | Required         | Decimal places allowed for prices.       |
| `size_precision`      | `u8`               | Required         | Decimal places allowed for order sizes.  |
| `price_increment`     | `Price`            | Required         | Smallest valid price step.               |
| `size_increment`      | `Quantity`         | Required         | Smallest valid size step.                |
| `ts_event`            | `UnixNanos`        | Required         | Event timestamp in nanoseconds.          |
| `ts_init`             | `UnixNanos`        | Required         | Initialization timestamp in nanoseconds. |
| `multiplier`          | `Quantity`         | `1`              | Contract multiplier.                     |
| `lot_size`            | `Quantity`         | `1`              | Rounded lot or board size.               |
| `max_quantity`        | `Option<Quantity>` | `None`           | Maximum order quantity.                  |
| `min_quantity`        | `Option<Quantity>` | `None`           | Minimum order quantity.                  |
| `max_notional`        | `Option<Money>`    | `None`           | Maximum order notional value.            |
| `min_notional`        | `Option<Money>`    | `None`           | Minimum order notional value.            |
| `max_price`           | `Option<Price>`    | `None`           | Maximum valid quote or order price.      |
| `min_price`           | `Option<Price>`    | `None`           | Minimum valid quote or order price.      |
| `margin_init`         | `Option<Decimal>`  | `0`              | Initial margin rate.                     |
| `margin_maint`        | `Option<Decimal>`  | `0`              | Maintenance margin rate.                 |
| `tick_scheme`         | `Option<Ustr>`     | `None`           | Registered variable tick scheme name.    |
| `info`                | `Option<Params>`   | `None`           | Adapter metadata.                        |

**Note**: Python constructors use `instrument_id`. Rust stores the same value as `id`.

## Behavior

- `CryptoPerpetual` has asset class `Cryptocurrency` and instrument class `Swap`.
- It has no activation or expiration timestamp.

**Settlement style**: the currency set determines the settlement style.

- **Linear**: typically sets `is_inverse=False` and settles in the quote currency.
- **Inverse**: sets `is_inverse=True` and typically settles in the base currency.
- **Quanto**: settles in a third currency that differs from both base and quote.

**Cost currency**: base for inverse contracts, settlement for quanto contracts, and quote otherwise.

:::note
Funding payments are not fields on the instrument. They arrive as data, such as
`FundingRateUpdate`, and reference the instrument ID.
:::

## Example

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    identifiers::{InstrumentId, Symbol},
    instruments::{CryptoPerpetual, InstrumentAny},
    types::{Currency, Money, Price, Quantity},
};
use rust_decimal_macros::dec;

let ethusdt_perp = CryptoPerpetual::builder()
    .instrument_id(InstrumentId::from("ETHUSDT-PERP.BINANCE"))
    .raw_symbol(Symbol::from("ETHUSDT"))
    .base_currency(Currency::from("ETH"))
    .quote_currency(Currency::from("USDT"))
    .settlement_currency(Currency::from("USDT"))
    .is_inverse(false)
    .price_precision(2)
    .size_precision(3)
    .price_increment(Price::from("0.01"))
    .size_increment(Quantity::from("0.001"))
    .max_quantity(Quantity::from("10000.000"))
    .min_quantity(Quantity::from("0.001"))
    .min_notional(Money::from("10.00 USDT"))
    .max_price(Price::from("15000.00"))
    .min_price(Price::from("1.00"))
    .margin_init(dec!(1.0))
    .margin_maint(dec!(0.35))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();

let instrument = InstrumentAny::CryptoPerpetual(ethusdt_perp);
```


## Adapters

**Adapters that create or consume `CryptoPerpetual` instruments include**:

- [Binance](../../integrations/binance.md) for USD-M and COIN-M perpetual futures.
- [Bybit](../../integrations/bybit.md) for linear and inverse perpetual products.
- [dYdX](../../integrations/dydx.md) for perpetual markets.
- [Hyperliquid](../../integrations/hyperliquid.md) for perpetual markets.
- [Kraken](../../integrations/kraken.md) for futures venue perpetual markets.
- [OKX](../../integrations/okx.md) for swap markets.
- [Tardis](../../integrations/tardis.md) for crypto perpetual metadata.

## Related guides

- [Data](../data/) covers mark prices, index prices, and funding rate updates.
- [Options](../options.md) covers option-specific instrument types.
- [Execution](../execution/) explains precision and notional checks before orders reach a venue.
