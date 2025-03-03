use hashbrown::{HashMap, HashSet};
use rust_decimal::Decimal;

use crate::transaction::{Chargeback, Deposit, Dispute, Resolve, Transaction, Withdrawal};
use crate::{ClientId, TransactionId};

pub struct Account {
    pub balance: Decimal,
    pub deposits: HashMap<TransactionId, Decimal>,
    pub disputes: HashSet<TransactionId>,
    pub locked: bool,
}

pub struct Ledger {
    pub accounts: HashMap<ClientId, Account>,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    pub fn process(&mut self, transaction: &Transaction) {
        match transaction {
            Transaction::Deposit(Deposit { client, tx, amount }) => {
                // deposit, which also serves as a create-client-if-it-doesn't-exist
                // deposits always work, even if an account is locked

                let Some(account) = self.accounts.get_mut(client) else {
                    self.accounts.insert(
                        *client,
                        Account {
                            balance: *amount,
                            disputes: HashSet::new(),
                            deposits: [(*tx, *amount)].into(),
                            locked: false,
                        },
                    );

                    return;
                };

                account.balance += amount;
                account.deposits.insert(*tx, *amount);
            },
            Transaction::Withdrawal(Withdrawal {
                client,
                tx: _tx,
                amount,
            }) => {
                let Some(account) = self.accounts.get_mut(client) else {
                    return;
                };

                // can't withdraw if we dont have enough money, or if the account is locked
                if account.balance < *amount || account.locked {
                    return;
                }

                account.balance -= amount;

                // this is a withdrawal, which isn't disputable
            },
            Transaction::Dispute(Dispute { client, tx }) => {
                let Some(account) = self.accounts.get_mut(client) else {
                    return;
                };

                let Some(&amount) = account.deposits.get(tx) else {
                    return;
                };

                account.disputes.insert(*tx);

                account.balance -= amount;
            },
            Transaction::Resolve(Resolve { client, tx }) => {
                let Some(account) = self.accounts.get_mut(client) else {
                    return;
                };

                let true = account.disputes.remove(tx) else {
                    return;
                };

                let Some(&amount) = account.deposits.get(tx) else {
                    panic!("Tried to resolve a dispute for a non-existing deposit");
                };

                // restore amount
                account.balance += amount;
            },
            Transaction::Chargeback(Chargeback { client, tx }) => {
                let Some(account) = self.accounts.get_mut(client) else {
                    return;
                };

                // we don't care about the amount, we merely check that this transaction is disputed,
                // and if so, this is a valid subject for a chargeback
                let true = account.disputes.remove(tx) else {
                    return;
                };

                account.locked = true;
            },
        }
    }
}

#[cfg(test)]
mod tests {

    use super::Ledger;
    use crate::transaction::{Chargeback, Deposit, Dispute, Resolve, Transaction, Withdrawal};

    #[test]
    fn deposit_goes_up() {
        let mut ledger = Ledger::new();

        ledger.process(&Transaction::Deposit(Deposit {
            client: 1,
            tx: 1,
            amount: 1.into(),
        }));

        ledger.process(&Transaction::Deposit(Deposit {
            client: 1,
            tx: 2,
            amount: 1.into(),
        }));

        let account = ledger.accounts.get(&1).unwrap();

        assert_eq!(account.balance, 2.into());
    }

    #[test]
    fn withdraw_goes_down() {
        let mut ledger = Ledger::new();

        ledger.process(&Transaction::Deposit(Deposit {
            client: 1,
            tx: 1,
            amount: 10.into(),
        }));

        ledger.process(&Transaction::Withdrawal(Withdrawal {
            client: 1,
            tx: 2,
            amount: 5.into(),
        }));

        let account = ledger.accounts.get(&1).unwrap();

        assert_eq!(account.balance, 5.into());
    }

    #[test]
    fn dispute_hides_money() {
        let mut ledger = Ledger::new();

        ledger.process(&Transaction::Deposit(Deposit {
            client: 1,
            tx: 1,
            amount: 5.into(),
        }));

        ledger.process(&Transaction::Deposit(Deposit {
            client: 1,
            tx: 2,
            amount: 5.into(),
        }));

        {
            let account = ledger.accounts.get(&1).unwrap();

            assert_eq!(account.balance, 10.into());
        }

        ledger.process(&Transaction::Dispute(Dispute { client: 1, tx: 2 }));

        {
            let account = ledger.accounts.get(&1).unwrap();

            assert_eq!(account.balance, 5.into());
        }
    }

    #[test]
    fn resolve_restores_disputed_money() {
        let mut ledger = Ledger::new();

        ledger.process(&Transaction::Deposit(Deposit {
            client: 1,
            tx: 1,
            amount: 5.into(),
        }));

        ledger.process(&Transaction::Deposit(Deposit {
            client: 1,
            tx: 2,
            amount: 5.into(),
        }));

        {
            let account = ledger.accounts.get(&1).unwrap();

            assert_eq!(account.balance, 10.into());
        }

        ledger.process(&Transaction::Dispute(Dispute { client: 1, tx: 2 }));

        {
            let account = ledger.accounts.get(&1).unwrap();

            assert_eq!(account.balance, 5.into());
        }

        ledger.process(&Transaction::Resolve(Resolve { client: 1, tx: 2 }));

        {
            let account = ledger.accounts.get(&1).unwrap();

            assert_eq!(account.balance, 10.into());
        }
    }

    #[test]
    fn chargeback_locks() {
        let mut ledger = Ledger::new();

        ledger.process(&Transaction::Deposit(Deposit {
            client: 1,
            tx: 1,
            amount: 5.into(),
        }));

        ledger.process(&Transaction::Deposit(Deposit {
            client: 1,
            tx: 2,
            amount: 5.into(),
        }));

        {
            let account = ledger.accounts.get(&1).unwrap();

            assert_eq!(account.balance, 10.into());
        }

        ledger.process(&Transaction::Dispute(Dispute { client: 1, tx: 2 }));

        {
            let account = ledger.accounts.get(&1).unwrap();

            assert_eq!(account.balance, 5.into());
        }

        ledger.process(&Transaction::Chargeback(Chargeback { client: 1, tx: 2 }));

        {
            let account = ledger.accounts.get(&1).unwrap();

            assert_eq!(account.balance, 5.into());
            assert!(account.locked);
        }
    }
}
