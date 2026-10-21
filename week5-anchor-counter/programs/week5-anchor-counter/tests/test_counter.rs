//! Тесты программы-счётчика на LiteSVM (быстрый in-process рантайм Solana).
use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    week5_anchor_counter::{accounts, instruction, state::Counter, COUNTER_SEED},
};

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/week5_anchor_counter.so"));
    svm.add_program(week5_anchor_counter::id(), bytes).unwrap();
    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();
    (svm, user)
}

fn counter_pda(authority: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[COUNTER_SEED, authority.as_ref()], &week5_anchor_counter::id()).0
}

fn send(svm: &mut LiteSVM, ix: Instruction, signer: &Keypair) -> bool {
    let msg = Message::new_with_blockhash(&[ix], Some(&signer.pubkey()), &svm.latest_blockhash());
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[signer]).unwrap();
    let ok = svm.send_transaction(tx).is_ok();
    svm.expire_blockhash();
    ok
}

fn init_ix(user: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        week5_anchor_counter::id(),
        &instruction::Initialize {}.data(),
        accounts::Initialize { authority: *user, counter: counter_pda(user), system_program: system_program::ID }
            .to_account_metas(None),
    )
}

fn update_ix(authority: &Pubkey, counter: Pubkey, data: Vec<u8>) -> Instruction {
    Instruction::new_with_bytes(
        week5_anchor_counter::id(),
        &data,
        accounts::Update { counter, authority: *authority }.to_account_metas(None),
    )
}

fn read(svm: &LiteSVM, user: &Pubkey) -> Counter {
    let account = svm.get_account(&counter_pda(user)).unwrap();
    Counter::try_deserialize(&mut account.data.as_slice()).unwrap()
}

#[test]
fn full_counter_lifecycle() {
    let (mut svm, user) = setup();
    let me = user.pubkey();
    let pda = counter_pda(&me);

    assert!(send(&mut svm, init_ix(&me), &user));
    assert_eq!(read(&svm, &me).count, 0);

    assert!(send(&mut svm, update_ix(&me, pda, instruction::Increment { step: 5 }.data()), &user));
    assert!(send(&mut svm, update_ix(&me, pda, instruction::Decrement { step: 2 }.data()), &user));
    let c = read(&svm, &me);
    assert_eq!((c.count, c.total_updates, c.authority), (3, 2, me));

    // ниже нуля нельзя
    assert!(!send(&mut svm, update_ix(&me, pda, instruction::Decrement { step: 4 }.data()), &user));
    // шаг 0 запрещён
    assert!(!send(&mut svm, update_ix(&me, pda, instruction::Increment { step: 0 }.data()), &user));

    assert!(send(&mut svm, update_ix(&me, pda, instruction::Reset {}.data()), &user));
    assert_eq!(read(&svm, &me).count, 0);
}

#[test]
fn cannot_initialize_twice() {
    let (mut svm, user) = setup();
    assert!(send(&mut svm, init_ix(&user.pubkey()), &user));
    assert!(!send(&mut svm, init_ix(&user.pubkey()), &user));
}

#[test]
fn stranger_cannot_touch_someone_elses_counter() {
    let (mut svm, owner) = setup();
    assert!(send(&mut svm, init_ix(&owner.pubkey()), &owner));

    let stranger = Keypair::new();
    svm.airdrop(&stranger.pubkey(), 1_000_000_000).unwrap();
    let owners_pda = counter_pda(&owner.pubkey());
    let ix = update_ix(&stranger.pubkey(), owners_pda, instruction::Increment { step: 1 }.data());
    assert!(!send(&mut svm, ix, &stranger));
    assert_eq!(read(&svm, &owner.pubkey()).count, 0);
}
