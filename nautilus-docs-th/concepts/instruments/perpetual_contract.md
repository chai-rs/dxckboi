# Perpetual Contract

`PerpetualContract` represents a generic perpetual futures contract across asset classes.
Use it when a venue exposes a perpetual swap that is not specifically modeled as
`CryptoPerpetual`.

Examples include non-crypto perpetual contracts and venue-specific synthetic swaps.

## Fields

| Field                 | Rust type          | Required/default | Notes                                    |
| --------------------- | ------------------ | ---------------- | ---------------------------------------- |
| `instrument_id`       | `InstrumentId`     | Required         | Stored as `id` in Rust.                  |
| `raw_symbol`          | `Symbol`           | Required         | Native venue symbol.                     |
| `underlying`          | `Ustr`             | Required         | Underlying asset or reference market.    |
| `asset_class`         | `AssetClass`       | Required         | Asset class of the underlying.           |
| `base_currency`       | `Option<Currency>` | `None`           | Base currency, required for inverse.     |
| `quote_currency`      | `Currency`         | Required         | Currency used to quote the price.        |
| `settlement_currency` | `Currency`         | Required         | Currency used to settle PnL and fees.    |
| `is_inverse`          | `bool`             | Required         | True when sizing/costing is inverse.     |
| `price_precision`     | `u8`               | Required         | Decimal places allowed for prices.       |
| `size_precision`      | `u8`               | Required         | Decimal places allowed for order sizes.  |
| `price_increment`     | `Price`            | Required         | Smallest valid price step.               |
| `size_increment`      | `Quantity`         | Required         | Smallest valid size step.                |
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
| `ts_event`            | `UnixNanos`        | Required         | Event timestamp in nanoseconds.          |
| `ts_init`             | `UnixNanos`        | Required         | Initialization timestamp in nanoseconds. |

**Note**: Python constructors use `instrument_id`. Rust stores the same value as `id`.

## Behavior

- `PerpetualContract` has instrument class `Swap`.
- It has no activation timestamp or expiration timestamp.
- Inverse contracts require a base currency.
- Linear contracts typically settle in the quote currency.
- Use `CryptoPerpetual` for crypto perpetuals where the base asset is a currency.

## Example

```rust tab="Rust"
use nautilus_core::UnixNanos;
use nautilus_model::{
    enums::AssetClass,
    identifiers::{InstrumentId, Symbol},
    instruments::PerpetualContract,
    types::{Currency, Price, Quantity},
};
use rust_decimal_macros::dec;
use ustr::Ustr;

let eurusd_perp = PerpetualContract::builder()
    .instrument_id(InstrumentId::from("EURUSD-PERP.AX"))
    .raw_symbol(Symbol::from("EURUSD-PERP"))
    .underlying(Ustr::from("EURUSD"))
    .asset_class(AssetClass::FX)
    .base_currency(Currency::from("EUR"))
    .quote_currency(Currency::from("USD"))
    .settlement_currency(Currency::from("USD"))
    .is_inverse(false)
    .price_precision(5)
    .size_precision(0)
    .price_increment(Price::from("0.00001"))
    .size_increment(Quantity::from("1"))
    .margin_init(dec!(0.03))
    .margin_maint(dec!(0.03))
    .ts_event(UnixNanos::default())
    .ts_init(UnixNanos::default())
    .build()
    .unwrap();
```


## Adapters

**Adapters that create or consume `PerpetualContract` instruments include**:

- [Architect AX](../../integrations/architect_ax.md) for venue-defined perpetual contracts.
- [Binance](../../integrations/binance.md) for USD-M TradFi perpetual contracts.

## Related guides

- [Crypto Perpetual](crypto_perpetual.md) covers crypto perpetual futures.
- [Data](../data/) covers mark prices, index prices, and funding rate updates.
