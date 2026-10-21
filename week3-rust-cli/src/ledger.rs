//! "Блокчейн" в миниатюре: аккаунты, выполнение транзакций, история.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::amount::LAMPORTS_PER_SOL;
use crate::error::{LedgerError, Result};
use crate::instruction::Instruction;
use crate::pack::Pack;
use crate::pubkey::{Pubkey, Signature, SYSTEM_PROGRAM_ID};

/// Базовая комиссия Solana: 5000 лампортов за подпись.
pub const FEE_LAMPORTS: u64 = 5_000;
/// Лимит faucet'а на один запрос, как у devnet.
pub const MAX_AIRDROP_LAMPORTS: u64 = 5 * LAMPORTS_PER_SOL;
/// Магические байты в начале файла (похоже на 8-байтовый discriminator в Anchor).
const MAGIC: [u8; 8] = *b"SOLCLI01";

/// Аккаунт: те же поля, что у `solana_sdk::account::Account`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub executable: bool,
}

impl Account {
    /// Обычный кошелёк: данных нет, владелец — System Program.
    pub fn new_system(lamports: u64) -> Self {
        Self { lamports, data: Vec::new(), owner: SYSTEM_PROGRAM_ID, executable: false }
    }
}

impl Pack for Account {
    fn pack(&self, out: &mut Vec<u8>) {
        self.lamports.pack(out);
        self.data.pack(out);
        self.owner.pack(out);
        self.executable.pack(out);
    }

    fn unpack(input: &mut &[u8]) -> Result<Self> {
        Ok(Self {
            lamports: u64::unpack(input)?,
            data: Vec::unpack(input)?,
            owner: Pubkey::unpack(input)?,
            executable: bool::unpack(input)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxStatus {
    Success,
    Failed(String),
}

impl Pack for TxStatus {
    fn pack(&self, out: &mut Vec<u8>) {
        match self {
            Self::Success => 0u8.pack(out),
            Self::Failed(reason) => {
                1u8.pack(out);
                reason.pack(out);
            }
        }
    }

    fn unpack(input: &mut &[u8]) -> Result<Self> {
        match u8::unpack(input)? {
            0 => Ok(Self::Success),
            1 => Ok(Self::Failed(String::unpack(input)?)),
            tag => Err(LedgerError::CorruptedData(format!("unknown tx status {tag}"))),
        }
    }
}

/// Запись о транзакции, попавшей в "блокчейн".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxRecord {
    pub signature: Signature,
    pub slot: u64,
    pub fee: u64,
    pub instruction: Instruction,
    pub status: TxStatus,
}

impl Pack for TxRecord {
    fn pack(&self, out: &mut Vec<u8>) {
        self.signature.pack(out);
        self.slot.pack(out);
        self.fee.pack(out);
        self.instruction.pack(out);
        self.status.pack(out);
    }

    fn unpack(input: &mut &[u8]) -> Result<Self> {
        Ok(Self {
            signature: Signature::unpack(input)?,
            slot: u64::unpack(input)?,
            fee: u64::unpack(input)?,
            instruction: Instruction::unpack(input)?,
            status: TxStatus::unpack(input)?,
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Ledger {
    pub slot: u64,
    /// On-chain состояние: адрес -> аккаунт (у валидатора это AccountsDB).
    pub accounts: HashMap<Pubkey, Account>,
    /// Локальный кошелёк: метка -> адрес (как файлы в ~/.config/solana/).
    /// Это НЕ часть блокчейна: адрес существует и без аккаунта on-chain.
    pub wallet: BTreeMap<String, Pubkey>,
    pub history: Vec<TxRecord>,
}

impl Pack for Ledger {
    fn pack(&self, out: &mut Vec<u8>) {
        MAGIC.pack(out);
        self.slot.pack(out);
        self.accounts.pack(out);
        self.wallet.pack(out);
        self.history.pack(out);
    }

    fn unpack(input: &mut &[u8]) -> Result<Self> {
        if <[u8; 8]>::unpack(input)? != MAGIC {
            return Err(LedgerError::CorruptedData("not a solcli ledger file".into()));
        }
        Ok(Self {
            slot: u64::unpack(input)?,
            accounts: HashMap::unpack(input)?,
            wallet: BTreeMap::unpack(input)?,
            history: Vec::unpack(input)?,
        })
    }
}

impl Ledger {
    pub fn load(path: &Path) -> Result<Self> {
        match fs::read(path) {
            Ok(bytes) => Self::from_bytes(&bytes),
            // Файла ещё нет: начинаем с пустого "генезиса"
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(err.into()),
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        fs::write(path, self.to_bytes())?;
        Ok(())
    }

    /// Баланс несуществующего аккаунта = 0 (как `solana balance`).
    pub fn balance(&self, pubkey: &Pubkey) -> u64 {
        self.accounts.get(pubkey).map_or(0, |account| account.lamports)
    }

    /// Сумма всех балансов. try_fold + checked_add: итератор остановится
    /// с ошибкой при переполнении, а не выдаст неверную сумму.
    pub fn total_supply(&self) -> Result<u64> {
        self.accounts
            .values()
            .try_fold(0u64, |sum, account| sum.checked_add(account.lamports))
            .ok_or(LedgerError::ArithmeticOverflow)
    }

    /// Комиссии сжигаются (в Solana сжигается часть, остальное — валидатору).
    pub fn fees_burned(&self) -> Result<u64> {
        self.history
            .iter()
            .try_fold(0u64, |sum, tx| sum.checked_add(tx.fee))
            .ok_or(LedgerError::ArithmeticOverflow)
    }

    pub fn label_of(&self, pubkey: &Pubkey) -> Option<&str> {
        self.wallet
            .iter()
            .find(|(_, key)| *key == pubkey)
            .map(|(label, _)| label.as_str())
    }

    /// Создаёт новый адрес под меткой. On-chain аккаунт при этом НЕ появляется.
    pub fn keygen(&mut self, label: &str, seed: u64) -> Result<Pubkey> {
        validate_label(label)?;
        if self.wallet.contains_key(label) {
            return Err(LedgerError::LabelAlreadyExists(label.to_string()));
        }
        let pubkey = Pubkey::new_unique(seed);
        self.wallet.insert(label.to_string(), pubkey);
        Ok(pubkey)
    }

    /// Метка из кошелька или base58-адрес -> Pubkey.
    pub fn resolve(&self, name: &str) -> Result<Pubkey> {
        match self.wallet.get(name) {
            Some(pubkey) => Ok(*pubkey),
            None => name.parse().map_err(|_| LedgerError::UnknownAccount(name.to_string())),
        }
    }

    /// Ищет транзакцию по подписи или её началу.
    pub fn find_transaction(&self, prefix: &str) -> Result<&TxRecord> {
        let matches: Vec<&TxRecord> = self
            .history
            .iter()
            .filter(|tx| !prefix.is_empty() && tx.signature.to_string().starts_with(prefix))
            .collect();

        match matches.as_slice() {
            [tx] => Ok(*tx),
            [] => Err(LedgerError::TransactionNotFound(prefix.to_string())),
            _ => Err(LedgerError::Usage(format!(
                "signature prefix `{prefix}` is ambiguous ({} matches)",
                matches.len()
            ))),
        }
    }

    /// Жизненный цикл транзакции, как в Solana:
    /// 1. декодируем instruction data из байтов;
    /// 2. списываем комиссию с fee payer'а (не хватает — транзакция отклонена целиком);
    /// 3. выполняем инструкцию на черновике состояния;
    /// 4. успех -> применяем черновик; ошибка -> откатываем всё, КРОМЕ комиссии.
    ///
    /// Preflight (симуляция) по умолчанию не пускает в сеть заведомо падающую
    /// транзакцию — тогда комиссия не списывается вовсе.
    pub fn process_transaction(&mut self, data: &[u8], skip_preflight: bool) -> Result<TxRecord> {
        let instruction =
            Instruction::from_bytes(data).map_err(|_| LedgerError::InvalidInstructionData)?;
        let slot = self.slot.checked_add(1).ok_or(LedgerError::ArithmeticOverflow)?;

        let mut state = self.accounts.clone();
        let fee = match instruction.fee_payer() {
            Some(payer) => {
                charge_fee(&mut state, payer, FEE_LAMPORTS)?;
                FEE_LAMPORTS
            }
            None => 0,
        };

        let mut draft = state.clone();
        let status = match execute_instruction(&mut draft, &instruction) {
            Ok(()) => {
                state = draft;
                TxStatus::Success
            }
            Err(err) if !skip_preflight => return Err(LedgerError::PreflightFailed(Box::new(err))),
            Err(err) => TxStatus::Failed(err.to_string()),
        };

        // Транзакция попала в блок: фиксируем состояние и пишем в историю
        self.accounts = state;
        self.slot = slot;
        let record = TxRecord {
            signature: Signature::derive(data, slot),
            slot,
            fee,
            instruction,
            status,
        };
        self.history.push(record.clone());
        Ok(record)
    }
}

/// Аналог `process_instruction` в Solana-программе: получает изменяемый доступ
/// к аккаунтам и либо применяет инструкцию, либо возвращает ошибку.
pub fn execute_instruction(
    accounts: &mut HashMap<Pubkey, Account>,
    instruction: &Instruction,
) -> Result<()> {
    match *instruction {
        Instruction::Airdrop { to, lamports } => {
            if lamports == 0 {
                return Err(LedgerError::InvalidAmount("0: amount must be greater than zero".into()));
            }
            if lamports > MAX_AIRDROP_LAMPORTS {
                return Err(LedgerError::AirdropLimitExceeded {
                    requested: lamports,
                    max: MAX_AIRDROP_LAMPORTS,
                });
            }
            credit(accounts, to, lamports)
        }
        Instruction::Transfer { from, to, lamports } => {
            if lamports == 0 {
                return Err(LedgerError::InvalidAmount("0: amount must be greater than zero".into()));
            }
            if from == to {
                return Err(LedgerError::SelfTransfer);
            }

            // Изменяемая ссылка на аккаунт внутри HashMap. Borrow checker
            // не даст одновременно держать вторую &mut на ту же карту,
            // поэтому сначала списываем, и только потом зачисляем.
            let source = accounts
                .get_mut(&from)
                .ok_or_else(|| LedgerError::AccountNotFound(from.to_string()))?;
            if source.owner != SYSTEM_PROGRAM_ID {
                return Err(LedgerError::InvalidAccountOwner);
            }
            let available = source.lamports;
            source.lamports = available
                .checked_sub(lamports)
                .ok_or(LedgerError::InsufficientFunds { needed: lamports, available })?;

            credit(accounts, to, lamports)
        }
    }
}

fn charge_fee(accounts: &mut HashMap<Pubkey, Account>, payer: Pubkey, fee: u64) -> Result<()> {
    let account = accounts
        .get_mut(&payer)
        .ok_or_else(|| LedgerError::AccountNotFound(payer.to_string()))?;
    let available = account.lamports;
    account.lamports = available
        .checked_sub(fee)
        .ok_or(LedgerError::InsufficientFundsForFee { fee, available })?;
    Ok(())
}

fn credit(accounts: &mut HashMap<Pubkey, Account>, to: Pubkey, lamports: u64) -> Result<()> {
    // Entry API: если адреса ещё нет on-chain, аккаунт создаётся при первом
    // поступлении лампортов — ровно так ведёт себя System Program.
    let account = accounts.entry(to).or_insert_with(|| Account::new_system(0));
    account.lamports = account
        .lamports
        .checked_add(lamports)
        .ok_or(LedgerError::ArithmeticOverflow)?;
    Ok(())
}

fn validate_label(label: &str) -> Result<()> {
    let valid_length = (1..=32).contains(&label.len());
    let valid_chars = label.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if valid_length && valid_chars && !label.starts_with('-') {
        Ok(())
    } else {
        Err(LedgerError::InvalidLabel(label.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOL: u64 = LAMPORTS_PER_SOL;

    /// Пустой ledger с двумя адресами в кошельке.
    fn setup() -> (Ledger, Pubkey, Pubkey) {
        let mut ledger = Ledger::default();
        let alice = ledger.keygen("alice", 1).unwrap();
        let bob = ledger.keygen("bob", 2).unwrap();
        (ledger, alice, bob)
    }

    fn send(ledger: &mut Ledger, instruction: Instruction, skip_preflight: bool) -> Result<TxRecord> {
        ledger.process_transaction(&instruction.to_bytes(), skip_preflight)
    }

    fn airdrop(ledger: &mut Ledger, to: Pubkey, lamports: u64) {
        let tx = send(ledger, Instruction::Airdrop { to, lamports }, false).unwrap();
        assert_eq!(tx.status, TxStatus::Success);
    }

    #[test]
    fn keygen_creates_address_but_not_onchain_account() {
        let (ledger, alice, _) = setup();
        assert_eq!(ledger.resolve("alice"), Ok(alice));
        assert!(!ledger.accounts.contains_key(&alice));
        assert_eq!(ledger.balance(&alice), 0);
    }

    #[test]
    fn keygen_rejects_bad_and_duplicate_labels() {
        let (mut ledger, _, _) = setup();
        assert_eq!(ledger.keygen("alice", 3), Err(LedgerError::LabelAlreadyExists("alice".into())));
        let too_long = "x".repeat(33);
        for bad in ["", "has space", "-flag", "emoji🙂", too_long.as_str()] {
            assert_eq!(ledger.keygen(bad, 3), Err(LedgerError::InvalidLabel(bad.into())));
        }
    }

    #[test]
    fn resolve_accepts_label_or_base58_address() {
        let (ledger, alice, _) = setup();
        assert_eq!(ledger.resolve(&alice.to_string()), Ok(alice));
        assert_eq!(ledger.resolve("carol"), Err(LedgerError::UnknownAccount("carol".into())));
    }

    #[test]
    fn airdrop_creates_account_and_is_free() {
        let (mut ledger, alice, _) = setup();
        let tx = send(&mut ledger, Instruction::Airdrop { to: alice, lamports: 2 * SOL }, false).unwrap();
        assert_eq!((tx.status, tx.fee, tx.slot), (TxStatus::Success, 0, 1));
        assert_eq!(ledger.balance(&alice), 2 * SOL);
        assert_eq!(ledger.accounts[&alice].owner, SYSTEM_PROGRAM_ID);
    }

    #[test]
    fn airdrop_is_limited() {
        let (mut ledger, alice, _) = setup();
        let requested = MAX_AIRDROP_LAMPORTS + 1;
        let err = send(&mut ledger, Instruction::Airdrop { to: alice, lamports: requested }, false);
        assert_eq!(
            err,
            Err(LedgerError::PreflightFailed(Box::new(LedgerError::AirdropLimitExceeded {
                requested,
                max: MAX_AIRDROP_LAMPORTS
            })))
        );
        assert!(ledger.history.is_empty());
    }

    #[test]
    fn transfer_moves_lamports_and_charges_fee() {
        let (mut ledger, alice, bob) = setup();
        airdrop(&mut ledger, alice, 2 * SOL);
        let tx = send(&mut ledger, Instruction::Transfer { from: alice, to: bob, lamports: SOL / 2 }, false)
            .unwrap();
        assert_eq!((tx.status, tx.fee), (TxStatus::Success, FEE_LAMPORTS));
        assert_eq!(ledger.balance(&alice), 2 * SOL - SOL / 2 - FEE_LAMPORTS);
        assert_eq!(ledger.balance(&bob), SOL / 2);
        assert_eq!(ledger.slot, 2);
    }

    #[test]
    fn transfer_to_new_address_creates_account() {
        let (mut ledger, alice, _) = setup();
        airdrop(&mut ledger, alice, SOL);
        let stranger = Pubkey::new_unique(42);
        send(&mut ledger, Instruction::Transfer { from: alice, to: stranger, lamports: 1_000 }, false).unwrap();
        assert_eq!(ledger.balance(&stranger), 1_000);
        assert_eq!(ledger.label_of(&stranger), None);
    }

    #[test]
    fn preflight_blocks_failing_transfer_without_side_effects() {
        let (mut ledger, alice, bob) = setup();
        airdrop(&mut ledger, alice, SOL);
        let before = ledger.clone();

        let result = send(&mut ledger, Instruction::Transfer { from: alice, to: bob, lamports: SOL }, false);
        // Ровно 1 SOL перевести нельзя: 5000 лампортов уже ушли на комиссию
        assert_eq!(
            result,
            Err(LedgerError::PreflightFailed(Box::new(LedgerError::InsufficientFunds {
                needed: SOL,
                available: SOL - FEE_LAMPORTS
            })))
        );
        assert_eq!(ledger, before);
    }

    #[test]
    fn failed_transaction_still_pays_fee_with_skip_preflight() {
        let (mut ledger, alice, bob) = setup();
        airdrop(&mut ledger, alice, SOL);

        let tx = send(&mut ledger, Instruction::Transfer { from: alice, to: bob, lamports: 2 * SOL }, true)
            .unwrap();
        assert!(matches!(tx.status, TxStatus::Failed(_)));
        assert_eq!(ledger.balance(&alice), SOL - FEE_LAMPORTS); // списана только комиссия
        assert_eq!(ledger.balance(&bob), 0); // сам перевод откатился
        assert_eq!(ledger.history.len(), 2);
    }

    #[test]
    fn fee_payer_must_exist_and_afford_the_fee() {
        let (mut ledger, alice, bob) = setup();
        let transfer = Instruction::Transfer { from: alice, to: bob, lamports: 1 };
        assert_eq!(
            send(&mut ledger, transfer.clone(), true),
            Err(LedgerError::AccountNotFound(alice.to_string()))
        );

        airdrop(&mut ledger, alice, 100); // 100 лампортов меньше комиссии в 5000
        assert_eq!(
            send(&mut ledger, transfer, true),
            Err(LedgerError::InsufficientFundsForFee { fee: FEE_LAMPORTS, available: 100 })
        );
        assert_eq!(ledger.slot, 1); // отклонённые транзакции в блок не попадают
    }

    #[test]
    fn rejects_self_transfer_and_zero_amount() {
        let (mut ledger, alice, bob) = setup();
        airdrop(&mut ledger, alice, SOL);
        let mut accounts = ledger.accounts.clone();
        assert_eq!(
            execute_instruction(&mut accounts, &Instruction::Transfer { from: alice, to: alice, lamports: 1 }),
            Err(LedgerError::SelfTransfer)
        );
        assert!(matches!(
            execute_instruction(&mut accounts, &Instruction::Transfer { from: alice, to: bob, lamports: 0 }),
            Err(LedgerError::InvalidAmount(_))
        ));
    }

    #[test]
    fn checked_math_catches_overflow() {
        let whale = Pubkey::new_unique(7);
        let mut accounts = HashMap::from([(whale, Account::new_system(u64::MAX - 10))]);
        let result = execute_instruction(&mut accounts, &Instruction::Airdrop { to: whale, lamports: 11 });
        assert_eq!(result, Err(LedgerError::ArithmeticOverflow));
        assert_eq!(accounts[&whale].lamports, u64::MAX - 10);
    }

    #[test]
    fn only_system_owned_accounts_can_send() {
        let program_owned = Pubkey::new_unique(8);
        let mut account = Account::new_system(SOL);
        account.owner = Pubkey::new_unique(9);
        let mut accounts = HashMap::from([(program_owned, account)]);
        let transfer = Instruction::Transfer { from: program_owned, to: Pubkey::new_unique(10), lamports: 1 };
        assert_eq!(execute_instruction(&mut accounts, &transfer), Err(LedgerError::InvalidAccountOwner));
    }

    #[test]
    fn supply_equals_airdrops_minus_burned_fees() {
        let (mut ledger, alice, bob) = setup();
        airdrop(&mut ledger, alice, 3 * SOL);
        airdrop(&mut ledger, bob, SOL);
        send(&mut ledger, Instruction::Transfer { from: alice, to: bob, lamports: SOL }, false).unwrap();
        // упадёт, но комиссию всё равно заплатит
        send(&mut ledger, Instruction::Transfer { from: bob, to: alice, lamports: 10 * SOL }, true).unwrap();

        assert_eq!(ledger.fees_burned(), Ok(2 * FEE_LAMPORTS));
        assert_eq!(ledger.total_supply(), Ok(4 * SOL - 2 * FEE_LAMPORTS));
    }

    #[test]
    fn find_transaction_by_signature_prefix() {
        let (mut ledger, alice, _) = setup();
        airdrop(&mut ledger, alice, SOL);
        let signature = ledger.history[0].signature.to_string();
        assert_eq!(ledger.find_transaction(&signature[..10]).unwrap().slot, 1);
        assert!(matches!(ledger.find_transaction(""), Err(LedgerError::TransactionNotFound(_))));
    }

    #[test]
    fn ledger_roundtrips_through_bytes_and_file() {
        let (mut ledger, alice, bob) = setup();
        airdrop(&mut ledger, alice, SOL);
        send(&mut ledger, Instruction::Transfer { from: alice, to: bob, lamports: 1_234 }, false).unwrap();

        let bytes = ledger.to_bytes();
        assert_eq!(&bytes[..8], b"SOLCLI01");
        let decoded = Ledger::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, ledger);
        // У восстановленной HashMap другой порядок обхода, но байты те же
        assert_eq!(decoded.to_bytes(), bytes);

        let path = std::env::temp_dir().join(format!("solcli-test-{}.bin", std::process::id()));
        ledger.save(&path).unwrap();
        let loaded = Ledger::load(&path);
        fs::remove_file(&path).unwrap();
        assert_eq!(loaded, Ok(ledger));
    }

    #[test]
    fn missing_file_is_fresh_ledger_and_garbage_is_rejected() {
        let missing = std::env::temp_dir().join("solcli-definitely-missing-file.bin");
        assert_eq!(Ledger::load(&missing), Ok(Ledger::default()));

        let mut bytes = setup().0.to_bytes();
        bytes.truncate(bytes.len() - 1);
        assert!(matches!(Ledger::from_bytes(&bytes), Err(LedgerError::CorruptedData(_))));
        assert!(matches!(Ledger::from_bytes(b"NOT_A_LEDGER"), Err(LedgerError::CorruptedData(_))));
    }
}
