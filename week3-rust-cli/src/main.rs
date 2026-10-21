//! solcli: мини-симулятор блокчейна Solana в терминале.
//! Неделя 3: Rust для Solana-разработчика.

#![forbid(unsafe_code)]

mod amount;
mod base58;
mod error;
mod instruction;
mod ledger;
mod pack;
mod pubkey;

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use amount::{format_sol, parse_sol};
use error::{LedgerError, Result};
use instruction::Instruction;
use ledger::{Ledger, TxStatus, FEE_LAMPORTS, MAX_AIRDROP_LAMPORTS};
use pack::Pack;
use pubkey::{Pubkey, SYSTEM_PROGRAM_ID};

const DEFAULT_LEDGER_FILE: &str = "ledger.bin";
const DEFAULT_HISTORY_LIMIT: usize = 20;
const KNOWN_COMMANDS: [&str; 10] = [
    "keygen", "airdrop", "transfer", "balance", "account", "accounts", "history", "tx", "reset", "help",
];

/// Команды CLI: каждый вариант enum несёт уже разобранные аргументы.
#[derive(Debug, PartialEq, Eq)]
enum Command {
    Keygen { label: String },
    Airdrop { amount: u64, to: String },
    Transfer { from: String, to: String, amount: u64, skip_preflight: bool },
    Balance { account: String },
    Account { account: String },
    Accounts,
    History { limit: usize },
    Tx { signature: String },
    Reset,
    Help,
}

impl Command {
    /// Разбор аргументов через pattern matching по срезу:
    /// форма каждой команды описана декларативно, в одном `match`.
    fn parse(args: &[String]) -> Result<Self> {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();

        let command = match args.as_slice() {
            [] | ["help" | "-h" | "--help"] => Self::Help,
            ["keygen", label] => Self::Keygen { label: label.to_string() },
            ["airdrop", amount, to] => Self::Airdrop { amount: parse_sol(amount)?, to: to.to_string() },
            ["transfer", from, to, amount, rest @ ..] => Self::Transfer {
                from: from.to_string(),
                to: to.to_string(),
                amount: parse_sol(amount)?,
                skip_preflight: match rest {
                    [] => false,
                    ["--skip-preflight"] => true,
                    _ => return Err(LedgerError::Usage(format!("unexpected arguments: {}", rest.join(" ")))),
                },
            },
            ["balance", account] => Self::Balance { account: account.to_string() },
            ["account", account] => Self::Account { account: account.to_string() },
            ["accounts"] => Self::Accounts,
            ["history"] => Self::History { limit: DEFAULT_HISTORY_LIMIT },
            ["history", limit] => Self::History {
                limit: limit
                    .parse()
                    .map_err(|_| LedgerError::Usage(format!("`{limit}` is not a valid number")))?,
            },
            ["tx", signature] => Self::Tx { signature: signature.to_string() },
            ["reset"] => Self::Reset,
            [command, ..] => {
                let message = if KNOWN_COMMANDS.contains(command) {
                    format!("wrong arguments for `{command}`")
                } else {
                    format!("unknown command `{command}`")
                };
                return Err(LedgerError::Usage(message));
            }
        };
        Ok(command)
    }

    fn mutates_state(&self) -> bool {
        matches!(self, Self::Keygen { .. } | Self::Airdrop { .. } | Self::Transfer { .. } | Self::Reset)
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<()> {
    let command = Command::parse(args)?;
    if command == Command::Help {
        print_help();
        return Ok(());
    }

    let path = ledger_path();
    // reset не читает старый файл: так можно начать заново даже с повреждённым ledger
    let mut ledger = if command == Command::Reset { Ledger::default() } else { Ledger::load(&path)? };
    let mutates = command.mutates_state();

    // Сохраняем даже при ошибке: транзакция с --skip-preflight могла упасть,
    // но комиссия уже списана и запись должна остаться в истории.
    let result = dispatch(&mut ledger, command);
    if mutates {
        ledger.save(&path)?;
    }
    result
}

fn ledger_path() -> PathBuf {
    env::var_os("SOLCLI_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LEDGER_FILE))
}

/// `command` передаётся по значению (move): после dispatch он больше не нужен,
/// и строки из него можно забрать без клонирования.
fn dispatch(ledger: &mut Ledger, command: Command) -> Result<()> {
    match command {
        Command::Keygen { label } => cmd_keygen(ledger, &label),
        Command::Airdrop { amount, to } => {
            let instruction = Instruction::Airdrop { to: ledger.resolve(&to)?, lamports: amount };
            send(ledger, instruction, false)
        }
        Command::Transfer { from, to, amount, skip_preflight } => {
            let instruction = Instruction::Transfer {
                from: ledger.resolve(&from)?,
                to: ledger.resolve(&to)?,
                lamports: amount,
            };
            send(ledger, instruction, skip_preflight)
        }
        Command::Balance { account } => {
            let pubkey = ledger.resolve(&account)?;
            println!("{} SOL", format_sol(ledger.balance(&pubkey)));
            Ok(())
        }
        Command::Account { account } => cmd_account(ledger, &account),
        Command::Accounts => cmd_accounts(ledger),
        Command::History { limit } => cmd_history(ledger, limit),
        Command::Tx { signature } => cmd_tx(ledger, &signature),
        Command::Reset => {
            *ledger = Ledger::default();
            println!("Ledger reset: all accounts, keys and history removed.");
            Ok(())
        }
        Command::Help => {
            print_help();
            Ok(())
        }
    }
}

fn cmd_keygen(ledger: &mut Ledger, label: &str) -> Result<()> {
    let pubkey = ledger.keygen(label, pubkey::random_seed(label))?;
    println!("Generated new address `{label}`");
    println!("pubkey: {pubkey}");
    println!("The account does not exist on-chain until it receives lamports.");
    println!("Try: solcli airdrop 2 {label}");
    Ok(())
}

/// Клиент сериализует инструкцию в байты: "программа" получает только `&[u8]`,
/// ровно как instruction_data в настоящей Solana.
fn send(ledger: &mut Ledger, instruction: Instruction, skip_preflight: bool) -> Result<()> {
    let data = instruction.to_bytes();
    let record = ledger.process_transaction(&data, skip_preflight)?;

    println!("{}", describe(ledger, &record.instruction));
    println!("Signature: {}", record.signature);
    println!("Slot: {} | Fee: {} SOL", record.slot, format_sol(record.fee));
    let balances: Vec<String> = record
        .instruction
        .accounts()
        .iter()
        .map(|pubkey| format!("{} = {} SOL", label_or_short(ledger, pubkey), format_sol(ledger.balance(pubkey))))
        .collect();
    println!("Balances: {}", balances.join(", "));

    match record.status {
        TxStatus::Success => {
            println!("Status: Ok");
            Ok(())
        }
        TxStatus::Failed(reason) => Err(LedgerError::TransactionFailed(reason)),
    }
}

/// Вывод в стиле `solana account <ADDRESS>`.
fn cmd_account(ledger: &Ledger, name: &str) -> Result<()> {
    let pubkey = ledger.resolve(name)?;
    let account = ledger
        .accounts
        .get(&pubkey)
        .ok_or_else(|| LedgerError::AccountNotFound(pubkey.to_string()))?;
    let owner_note = if account.owner == SYSTEM_PROGRAM_ID { " (System Program)" } else { "" };
    let bytes = account.to_bytes();

    println!("Public Key: {}", full_name(ledger, &pubkey));
    println!("Balance:    {} SOL ({} lamports)", format_sol(account.lamports), account.lamports);
    println!("Owner:      {}{owner_note}", account.owner);
    println!("Executable: {}", account.executable);
    println!("Length:     {len} (0x{len:x}) bytes", len = account.data.len());
    println!();
    println!("Serialized account, {} bytes (lamports u64 | data Vec<u8> | owner [u8; 32] | executable bool):", bytes.len());
    println!("{}", hex_dump(&bytes));
    Ok(())
}

fn cmd_accounts(ledger: &Ledger) -> Result<()> {
    if ledger.wallet.is_empty() && ledger.accounts.is_empty() {
        println!("No accounts yet. Try: solcli keygen alice");
        return Ok(());
    }

    println!("{:<12} {:<44} {:>22}", "LABEL", "ADDRESS", "BALANCE (SOL)");
    for (label, pubkey) in &ledger.wallet {
        let balance = match ledger.accounts.get(pubkey) {
            Some(account) => format_sol(account.lamports),
            None => "- (not on-chain)".to_string(),
        };
        println!("{label:<12} {pubkey:<44} {balance:>22}");
    }

    // Аккаунты, созданные переводом на адрес, которого нет в нашем кошельке
    let mut external: Vec<(&Pubkey, u64)> = ledger
        .accounts
        .iter()
        .filter(|(pubkey, _)| ledger.label_of(pubkey).is_none())
        .map(|(pubkey, account)| (pubkey, account.lamports))
        .collect();
    external.sort();
    for (pubkey, lamports) in external {
        println!("{:<12} {pubkey:<44} {:>22}", "(external)", format_sol(lamports));
    }

    println!();
    println!(
        "On-chain accounts: {} | Total supply: {} SOL | Fees burned: {} SOL | Slot: {}",
        ledger.accounts.len(),
        format_sol(ledger.total_supply()?),
        format_sol(ledger.fees_burned()?),
        ledger.slot
    );
    Ok(())
}

fn cmd_history(ledger: &Ledger, limit: usize) -> Result<()> {
    if ledger.history.is_empty() {
        println!("No transactions yet.");
        return Ok(());
    }

    println!("{:>4}  {:<15}  {:<6}  {:>9}  INSTRUCTION", "SLOT", "SIGNATURE", "STATUS", "FEE (SOL)");
    // Итераторы: новые сверху, не больше limit штук
    for tx in ledger.history.iter().rev().take(limit) {
        let status = match tx.status {
            TxStatus::Success => "Ok",
            TxStatus::Failed(_) => "Failed",
        };
        println!(
            "{:>4}  {:<15}  {status:<6}  {:>9}  {}",
            tx.slot,
            tx.signature.short(),
            format_sol(tx.fee),
            describe(ledger, &tx.instruction)
        );
        if let TxStatus::Failed(reason) = &tx.status {
            println!("{:>4}  -> {reason}", "");
        }
    }
    Ok(())
}

/// Разбор транзакции "как в Explorer": поля + сырые байты instruction data.
fn cmd_tx(ledger: &Ledger, signature: &str) -> Result<()> {
    let tx = ledger.find_transaction(signature)?;
    let data = tx.instruction.to_bytes();

    println!("Signature:   {}", tx.signature);
    println!("Slot:        {}", tx.slot);
    match &tx.status {
        TxStatus::Success => println!("Status:      Ok"),
        TxStatus::Failed(reason) => println!("Status:      Failed ({reason})"),
    }
    println!("Fee:         {} SOL", format_sol(tx.fee));
    println!("Program:     {} (System Program)", SYSTEM_PROGRAM_ID);
    println!("Instruction: {} (tag byte = {})", tx.instruction.name(), data[0]);
    match &tx.instruction {
        Instruction::Airdrop { to, lamports } => {
            println!("  to:        {}", full_name(ledger, to));
            println!("  lamports:  {lamports} ({} SOL)", format_sol(*lamports));
        }
        Instruction::Transfer { from, to, lamports } => {
            println!("  from:      {}", full_name(ledger, from));
            println!("  to:        {}", full_name(ledger, to));
            println!("  lamports:  {lamports} ({} SOL)", format_sol(*lamports));
        }
    }
    println!();
    println!("Instruction data, {} bytes:", data.len());
    println!("{}", hex_dump(&data));
    Ok(())
}

fn describe(ledger: &Ledger, instruction: &Instruction) -> String {
    match instruction {
        Instruction::Airdrop { to, lamports } => {
            format!("Airdrop {} SOL -> {}", format_sol(*lamports), label_or_short(ledger, to))
        }
        Instruction::Transfer { from, to, lamports } => format!(
            "Transfer {} SOL {} -> {}",
            format_sol(*lamports),
            label_or_short(ledger, from),
            label_or_short(ledger, to)
        ),
    }
}

fn label_or_short(ledger: &Ledger, pubkey: &Pubkey) -> String {
    match ledger.label_of(pubkey) {
        Some(label) => label.to_string(),
        None => pubkey.short(),
    }
}

fn full_name(ledger: &Ledger, pubkey: &Pubkey) -> String {
    match ledger.label_of(pubkey) {
        Some(label) => format!("{pubkey} ({label})"),
        None => pubkey.to_string(),
    }
}

fn hex_dump(bytes: &[u8]) -> String {
    bytes
        .chunks(16)
        .enumerate()
        .map(|(row, chunk)| {
            let hex: Vec<String> = chunk.iter().map(|byte| format!("{byte:02x}")).collect();
            format!("  {:04x}: {}", row * 16, hex.join(" "))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn print_help() {
    println!(
        "solcli - mini Solana ledger simulator (week 3: Rust for Solana developers)

USAGE:
    solcli <COMMAND> [ARGS]

COMMANDS:
    keygen <label>                  Generate a new address and save it under a label
    airdrop <SOL> <account>         Request test SOL (max {max_airdrop} SOL, no fee)
    transfer <from> <to> <SOL> [--skip-preflight]
                                    Transfer SOL (fee {fee} SOL). With --skip-preflight
                                    a failing transaction still lands and pays the fee
    balance <account>               Show balance
    account <account>               Inspect an account (like `solana account`)
    accounts                        List all accounts, total supply and burned fees
    history [N]                     Show the last N transactions (default {history})
    tx <signature>                  Inspect a transaction and its raw instruction data
                                    (a unique signature prefix is enough)
    reset                           Remove all accounts, keys and history
    help                            Show this help

<account> is a wallet label (e.g. alice) or a base58 public key.
State is stored in ./{file} (override with the SOLCLI_LEDGER env variable).",
        max_airdrop = format_sol(MAX_AIRDROP_LAMPORTS),
        fee = format_sol(FEE_LAMPORTS),
        history = DEFAULT_HISTORY_LIMIT,
        file = DEFAULT_LEDGER_FILE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Command> {
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        Command::parse(&args)
    }

    #[test]
    fn parses_commands() {
        assert_eq!(parse(&[]), Ok(Command::Help));
        assert_eq!(parse(&["keygen", "alice"]), Ok(Command::Keygen { label: "alice".into() }));
        assert_eq!(
            parse(&["airdrop", "2", "alice"]),
            Ok(Command::Airdrop { amount: 2_000_000_000, to: "alice".into() })
        );
        assert_eq!(
            parse(&["transfer", "alice", "bob", "0.5", "--skip-preflight"]),
            Ok(Command::Transfer {
                from: "alice".into(),
                to: "bob".into(),
                amount: 500_000_000,
                skip_preflight: true
            })
        );
        assert_eq!(parse(&["history", "5"]), Ok(Command::History { limit: 5 }));
    }

    #[test]
    fn reports_usage_errors() {
        assert!(matches!(parse(&["fly"]), Err(LedgerError::Usage(msg)) if msg.contains("unknown command")));
        assert!(matches!(parse(&["airdrop", "1"]), Err(LedgerError::Usage(msg)) if msg.contains("wrong arguments")));
        assert!(matches!(parse(&["transfer", "a", "b", "1", "--force"]), Err(LedgerError::Usage(_))));
        assert!(matches!(parse(&["history", "ten"]), Err(LedgerError::Usage(_))));
        assert!(matches!(parse(&["airdrop", "lots", "alice"]), Err(LedgerError::InvalidAmount(_))));
    }

    #[test]
    fn hex_dump_formats_rows_of_16_bytes() {
        let bytes: Vec<u8> = (0..18).collect();
        assert_eq!(
            hex_dump(&bytes),
            "  0000: 00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f\n  0010: 10 11"
        );
    }
}
