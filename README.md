# ATM interview

## Notes

### Observations:

- We print up to 4 places behind the decimal point, but only if we need to do so
- The example instructions in the formatted output is missing the `locked` column
  - We do print that last column name

### Safety

- Work was done to create a representation of Transaction that represents all the invariants

### Storing money

I opted to use `rust_decimal` as a crate to offer a `Decimal` type which offers exactly what we need with std-like ergonomics.

We don't even have to worry about the precision. The instructions mention that data coming in has a maximum of 4 places past the decimal, and the only operations on money we support are + and -, so we never create more precision. Care is taken to print no trailing zeros.

### Scaling

- Code is written in a way where we have a CSV function parse a CSV line by line, emitting the transactions as we go, avoiding the need to keep it all into memory until completion. This means that we can replace the underlying file reader with a socket listener

### Crates used

- clap: parse cli commands
- csv: to parse csv
- hashbrown: faster hashmap, same api, and we don't need to worry about less HashDos resistance than the one from std
- rust_decimal: perfect crate for money
- serde: to easily serialize csv rows
- tokio: async runtime, channels

### Assumptions on how ATMs work

- You can deposit on locked accounts
- You cannot withdraw from locked accounts
- You can dispute / resolve / chargeback other transactions on locked accounts
- Only deposit-type transactions can be disputed
- There is no protection on chargebacks making the amount go negative, e.g. deposit 100 (tx 1), withdraw 100 (tx 2), dispute tx 2, chargeback tx 2. Now there is -100 money in the account

### Efficiency

- Balance is stored as a value, and is not constantly recalulated based on all the past transactions
- Disputes only store disputed transaction ids, not the amount. We could store them separately at the risk of storing value twice

### What's missing?

- Better history support.
  - Only deposits and disputes are individually recorded
  - Once a dispute has been resolved or charge-backed, there is no history of it
- More complex tests
- Error logging
- Transaction support / locking of entities / enhanced state validation (see `src/ledger.rs#89`)
- Support for different kind of locals, e.g. France where the , is a ; and the . is a ,
- Email client
