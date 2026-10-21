//! Тесты профиля на LiteSVM.
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
    week6_anchor_profile::{accounts, instruction, Profile, PROFILE_SEED, USERNAME_SEED},
};

fn setup() -> LiteSVM {
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/week6_anchor_profile.so"));
    svm.add_program(week6_anchor_profile::id(), bytes).unwrap();
    svm
}

fn user(svm: &mut LiteSVM) -> Keypair {
    let kp = Keypair::new();
    svm.airdrop(&kp.pubkey(), 1_000_000_000).unwrap();
    kp
}

fn pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &week6_anchor_profile::id()).0
}

fn profile_pda(authority: &Pubkey) -> Pubkey {
    pda(&[PROFILE_SEED, authority.as_ref()])
}

fn username_pda(name: &str) -> Pubkey {
    pda(&[USERNAME_SEED, name.as_bytes()])
}

fn send(svm: &mut LiteSVM, ix: Instruction, signer: &Keypair) -> bool {
    let msg = Message::new_with_blockhash(&[ix], Some(&signer.pubkey()), &svm.latest_blockhash());
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[signer]).unwrap();
    let ok = svm.send_transaction(tx).is_ok();
    svm.expire_blockhash();
    ok
}

fn create_ix(owner: &Pubkey, username: &str, bio: &str) -> Instruction {
    Instruction::new_with_bytes(
        week6_anchor_profile::id(),
        &instruction::CreateProfile { username: username.into(), bio: bio.into(), avatar_url: "https://x.y/a.png".into() }
            .data(),
        accounts::CreateProfile {
            authority: *owner,
            profile: profile_pda(owner),
            username_record: username_pda(username),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn read(svm: &LiteSVM, owner: &Pubkey) -> Option<Profile> {
    let account = svm.get_account(&profile_pda(owner))?;
    if account.data.is_empty() {
        return None;
    }
    Profile::try_deserialize(&mut account.data.as_slice()).ok()
}

#[test]
fn create_update_delete() {
    let mut svm = setup();
    let alice = user(&mut svm);
    let me = alice.pubkey();

    assert!(send(&mut svm, create_ix(&me, "alice", "gm"), &alice));
    let p = read(&svm, &me).unwrap();
    assert_eq!((p.username.as_str(), p.bio.as_str(), p.authority), ("alice", "gm", me));

    let update = Instruction::new_with_bytes(
        week6_anchor_profile::id(),
        &instruction::UpdateProfile { bio: "building on Solana".into(), avatar_url: String::new() }.data(),
        accounts::UpdateProfile { authority: me, profile: profile_pda(&me) }.to_account_metas(None),
    );
    assert!(send(&mut svm, update, &alice));
    assert_eq!(read(&svm, &me).unwrap().bio, "building on Solana");

    let delete = Instruction::new_with_bytes(
        week6_anchor_profile::id(),
        &instruction::DeleteProfile {}.data(),
        accounts::DeleteProfile { authority: me, profile: profile_pda(&me), username_record: username_pda("alice") }
            .to_account_metas(None),
    );
    assert!(send(&mut svm, delete, &alice));
    assert!(read(&svm, &me).is_none());

    // После удаления имя снова свободно
    let bob = user(&mut svm);
    assert!(send(&mut svm, create_ix(&bob.pubkey(), "alice", ""), &bob));
}

#[test]
fn usernames_are_unique_and_validated() {
    let mut svm = setup();
    let alice = user(&mut svm);
    let bob = user(&mut svm);
    assert!(send(&mut svm, create_ix(&alice.pubkey(), "satoshi", ""), &alice));
    assert!(!send(&mut svm, create_ix(&bob.pubkey(), "satoshi", ""), &bob)); // имя занято
    assert!(!send(&mut svm, create_ix(&bob.pubkey(), "Bad Name", ""), &bob)); // невалидное имя
    assert!(!send(&mut svm, create_ix(&bob.pubkey(), "bob", &"x".repeat(201)), &bob)); // длинное bio
    assert!(send(&mut svm, create_ix(&bob.pubkey(), "bob", "ok"), &bob));
}

#[test]
fn one_profile_per_wallet_and_only_owner_updates() {
    let mut svm = setup();
    let alice = user(&mut svm);
    let mallory = user(&mut svm);
    assert!(send(&mut svm, create_ix(&alice.pubkey(), "alice", ""), &alice));
    assert!(!send(&mut svm, create_ix(&alice.pubkey(), "alice2", ""), &alice)); // второй профиль нельзя

    // Mallory пытается обновить профиль Alice, подставив её PDA
    let hack = Instruction::new_with_bytes(
        week6_anchor_profile::id(),
        &instruction::UpdateProfile { bio: "hacked".into(), avatar_url: String::new() }.data(),
        accounts::UpdateProfile { authority: mallory.pubkey(), profile: profile_pda(&alice.pubkey()) }
            .to_account_metas(None),
    );
    assert!(!send(&mut svm, hack, &mallory));
    assert_eq!(read(&svm, &alice.pubkey()).unwrap().bio, "");
}
