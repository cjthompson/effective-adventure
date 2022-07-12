## Solution Explanation
The rust type system was used to verify the input data was in the correct format,
allowing the code to skip CSV rows that don't conform to the expected data structure.

Rust enums were used to differentiate between the different types of transactions.
Deposits and Withdrawals are treated differently than Disputes, Resolves, and Chargebacks.

A custom serializer was made to avoid serializing data that was used for internal processing
and to add the `total` field as a calculated value not stored in the struct.

### Error handling
1. Exit if there isn't just 1 CLI argument
2. If the file cannot be read, exit cleanly
3. If a line in the CSV file cannot be parsed, ignore it and continue
4. If a dispute, resolve, or chargeback refers to a transaction that doesn't belong to the account, ignore it
5. If a d/r/c refers to a transaction that's already disputed, ignore it
6. If a transaction has a duplicate id, ignore it
7. If a withdrawal is attempted with an amount greater than available, ignore it
8. If a with/depo has a negative amount, ignore it
