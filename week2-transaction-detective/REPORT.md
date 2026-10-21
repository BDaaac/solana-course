# Transaction Detective: разбор реальных транзакций Solana (mainnet)

Сгенерировано скриптом `investigate.ts` 2026-09-30T09:39:20.801Z.

Роли аккаунтов: **fee payer** платит комиссию, **signer** подписал транзакцию,
**writable** будет изменён, **read-only** только читается, **program** — исполняемый аккаунт.
Это нужно рантайму Sealevel: транзакции с непересекающимися writable-аккаунтами идут параллельно.

### Случай 1. Обмен токенов через Jupiter

- Signature: [`5Wp56YfXiBGxeEiTS3dk...`](https://explorer.solana.com/tx/5Wp56YfXiBGxeEiTS3dk1jd7gRbYJv9WUov6kRa1Pyoh7ocTNryiEU1Z1VTqgvnCBqRWFo7nyiiFgxHZ8N7yyzdU)
- Cluster: mainnet, slot 451926629, 2026-09-30T09:39:10.000Z
- Status: Success; fee 10113 lamports; compute units 79604; version 0
- Signers: `7Q4EJwJAHiyRx8PzJc18nAQQAJYcwXpV2ZwxUECnKjMP`, `sighWH8KaiT7QhtV4w29ReVF8kG6D5yG3EQP1KYyGVF`

**Accounts**

| # | Address | Roles | SOL change |
|---|---|---|---|
| 0 | `7Q4EJwJAHiyRx8PzJc18nAQQAJYcwXpV2ZwxUECnKjMP` | fee payer, signer, writable | -0.000610113 |
| 1 | `sighWH8KaiT7QhtV4w29ReVF8kG6D5yG3EQP1KYyGVF` | signer, read-only | 0 |
| 2 | `3xSYy6Sm3KEDsA7Cm28TgTajYmPXkFv5zbsCCnPwVwJ6` | writable | 0 |
| 3 | `6MurRy3b2ZufYxuZqmNFehkc7YpwfhDhBCBGmunLWJnX` | writable | 0.000597 |
| 4 | `6QADSWK19vGUfbDa8qnz7GH9qb5hc5tEjGuwWbSFxuji` | writable | 0 |
| 5 | `8mdkYgQGViEbKGKdvNbExUppo3Ao6SUePHgq4varpiWS` | writable | 0 |
| 6 | `9kXgCLhLd1rx2gDxdZv9UDRYyAcSFTdD4qizJAUpPcby` | writable | 0 |
| 7 | `A5U7BC6NYxPGNVPhj4zHnZgC9ueYKVQ373nDsbwkHgnS` | writable | 0 |
| 8 | `ADKKyzoY8MUtPAMgjuBkuc9Y3BmZaTueeGkbMo7CVrdg` | writable | 0.000003 |
| 9 | `AHxyEVjDUrVe9KEgPFgtWLYzefRSrWiUWJkiqtYW3BXa` | writable | 0 |
| 10 | `BbKy8yZSmXD8xcoMju3gEyqxLGJ2wSPDCFhYRv17ngPY` | writable | 0 |
| 11 | `MDLxoYj5WAzQF18oyTsLh5B3tNarokSGjNzfuHr8bkM` | writable | 0 |
| 12 | `11111111111111111111111111111111` | read-only, program | 0 |
| 13 | `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL` | read-only, program | 0 |
| 14 | `ComputeBudget111111111111111111111111111111` | read-only, program | 0 |
| 15 | `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4` | read-only, program | 0 |
| 16 | `MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr` | read-only | 0 |
| 17 | `resideK2Ejv9apu1op2VBv89poumxCDCs8nzBxQ5q1y` | read-only | 0 |
| 18 | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | read-only, program | 0 |
| 19 | `D8cy77BBepLMngZx6ZukaTff5hCt1HrWyKk3Hnd9oitf` | read-only, from lookup table | 0 |
| 20 | `jitodontfront11111111111JustUseJupiterU1tra` | read-only, from lookup table | 0 |
| 21 | `So11111111111111111111111111111111111111112` | read-only, from lookup table | 0 |
| 22 | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | read-only, from lookup table | 0 |
| 23 | `6tBc3ABLaYTTWu94DiRD5PWi92HML34UpAQ8pPTYgudw` | read-only, from lookup table | 0 |
| 24 | `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK` | read-only, from lookup table | 0 |

**Instructions**

| # | Program | Type | Details |
|---|---|---|---|
| 1 | Compute Budget | raw | data (base58, 6 chars): HFVx5V |
| 2 | Compute Budget | raw | data (base58, 12 chars): 3s6TYGBZGZ8K |
| 3 | Associated Token Account | createIdempotent | {"account":"8mdkYgQGViEbKGKdvNbExUppo3Ao6SUePHgq4varpiWS","mint":"So11111111111111111111111111111111111111112","source":"7Q4EJwJAHiyRx8PzJc18nAQQAJYcwXpV2ZwxUEC |
| 3.1 (CPI) | SPL Token | getAccountDataSize | {"extensionTypes":["immutableOwner"],"mint":"So11111111111111111111111111111111111111112"} |
| 3.2 (CPI) | System Program | createAccount | {"lamports":1488440,"newAccount":"8mdkYgQGViEbKGKdvNbExUppo3Ao6SUePHgq4varpiWS","owner":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA","source":"7Q4EJwJAHiyRx8Pz |
| 3.3 (CPI) | SPL Token | initializeImmutableOwner | {"account":"8mdkYgQGViEbKGKdvNbExUppo3Ao6SUePHgq4varpiWS"} |
| 3.4 (CPI) | SPL Token | initializeAccount3 | {"account":"8mdkYgQGViEbKGKdvNbExUppo3Ao6SUePHgq4varpiWS","mint":"So11111111111111111111111111111111111111112","owner":"7Q4EJwJAHiyRx8PzJc18nAQQAJYcwXpV2ZwxUECn |
| 4 | System Program | transfer | {"destination":"8mdkYgQGViEbKGKdvNbExUppo3Ao6SUePHgq4varpiWS","lamports":600000,"source":"7Q4EJwJAHiyRx8PzJc18nAQQAJYcwXpV2ZwxUECnKjMP"} |
| 5 | SPL Token | syncNative | {"account":"8mdkYgQGViEbKGKdvNbExUppo3Ao6SUePHgq4varpiWS"} |
| 6 | Jupiter Aggregator v6 | raw | data (base58, 54 chars): 37MZM8vwf4KGscyLnxNbQoDQEBc1nLjhkGqxjhzojvirWEfDGwoeuN |
| 6.1 (CPI) | Raydium CLMM | raw | data (base58, 56 chars): ASCsAbe1UnDp6J7D5jmngyzTixXjiwYtap5q8qcPhtTa4wby9SEdFZ8U |
| 6.2 (CPI) | SPL Token | transferChecked | {"authority":"7Q4EJwJAHiyRx8PzJc18nAQQAJYcwXpV2ZwxUECnKjMP","destination":"6MurRy3b2ZufYxuZqmNFehkc7YpwfhDhBCBGmunLWJnX","mint":"So11111111111111111111111111111 |
| 6.3 (CPI) | Token-2022 (Token Extensions) | transferChecked | {"authority":"AHxyEVjDUrVe9KEgPFgtWLYzefRSrWiUWJkiqtYW3BXa","destination":"MDLxoYj5WAzQF18oyTsLh5B3tNarokSGjNzfuHr8bkM","mint":"resideK2Ejv9apu1op2VBv89poumxCDC |
| 6.4 (CPI) | Jupiter Aggregator v6 | raw | data (base58, 181 chars): 3drYVtAcBYiMtM5fDAD2VT79gAoN6vD18v9Jn7pH7sLP26vfth5Vxkahuq6c... |
| 6.5 (CPI) | SPL Token | transfer | {"amount":"3000","authority":"7Q4EJwJAHiyRx8PzJc18nAQQAJYcwXpV2ZwxUECnKjMP","destination":"ADKKyzoY8MUtPAMgjuBkuc9Y3BmZaTueeGkbMo7CVrdg","source":"8mdkYgQGViEbK |
| 7 | SPL Token | closeAccount | {"account":"8mdkYgQGViEbKGKdvNbExUppo3Ao6SUePHgq4varpiWS","destination":"7Q4EJwJAHiyRx8PzJc18nAQQAJYcwXpV2ZwxUECnKjMP","owner":"7Q4EJwJAHiyRx8PzJc18nAQQAJYcwXpV |

**Programs**

- Compute Budget `ComputeBudget111111111111111111111111111111`: 2 call(s)
- Associated Token Account `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL`: 1 call(s)
- SPL Token `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`: 7 call(s)
- System Program `11111111111111111111111111111111`: 2 call(s)
- Jupiter Aggregator v6 `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`: 2 call(s)
- Raydium CLMM `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK`: 1 call(s)
- Token-2022 (Token Extensions) `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`: 1 call(s)

**Token balance changes**

- owner `AHxyEVjDUrVe9KEgPFgtWLYzefRSrWiUWJkiqtYW3BXa`, mint `So11111111111111111111111111111111111111112`: +0.0005970000000001807
- owner `AHxyEVjDUrVe9KEgPFgtWLYzefRSrWiUWJkiqtYW3BXa`, mint `resideK2Ejv9apu1op2VBv89poumxCDCs8nzBxQ5q1y`: -279.4980239998549
- owner `6Gfahmxqn1VfJE1rZ3JsDZdztUKMc4F8bjR9pfu1XhWZ`, mint `So11111111111111111111111111111111111111112`: +0.0000030000000000030003
- owner `7Q4EJwJAHiyRx8PzJc18nAQQAJYcwXpV2ZwxUECnKjMP`, mint `resideK2Ejv9apu1op2VBv89poumxCDCs8nzBxQ5q1y`: +279.49802400000044

**Выводы**

- В транзакции 25 аккаунтов: 2 подписант(ов), 11 writable, остальные read-only.
- Верхнеуровневых инструкций: 7, вложенных вызовов (CPI): 9.
- Задействованные программы: Compute Budget, Associated Token Account, SPL Token, System Program, Jupiter Aggregator v6, Raydium CLMM, Token-2022 (Token Extensions).
- Комиссию 10113 лампортов заплатил fee payer `7Q4EJwJAHiyRx8PzJc18nAQQAJYcwXpV2ZwxUECnKjMP`.

### Случай 2. Memo-транзакция

- Signature: [`2pkEFwnVC3rZBqrA74p5...`](https://explorer.solana.com/tx/2pkEFwnVC3rZBqrA74p5j1rn24qEfTUERCyW2vVjFeHqYPVpF4DvqfW9VGjoh1xKStbh2cZAKGQbq5NJ1QvHfM79)
- Cluster: mainnet, slot 451926644, 2026-09-30T09:39:14.000Z
- Status: Success; fee 6000 lamports; compute units 78683; version 0
- Signers: `7YiS8wVM4B2wS1CSfncWX4n362jyH9yJc8YA18BtcksH`

**Accounts**

| # | Address | Roles | SOL change |
|---|---|---|---|
| 0 | `7YiS8wVM4B2wS1CSfncWX4n362jyH9yJc8YA18BtcksH` | fee payer, signer, writable | 1.156985838 |
| 1 | `3pSY1sdZ7QteSuQir2njHSFwT61U5vPawr74nJkT8qSu` | writable | 0 |
| 2 | `42uyxKNeaHMbGkags8H5G2HnEiF39njvsyC8VTWfSEgy` | writable | 0 |
| 3 | `4t3hGUfpuhTMDxdXZkcVGn4KocCak9NDC8MXpsuEXXQh` | writable | 0 |
| 4 | `6aNTbXW6gWWtKTPbcPFuWTAt1QaPaQxQ4DXHAmDpUcfW` | writable | 0 |
| 5 | `6w9FvmaPTWLH8H2EN8DR3qWi6YZC8qMNT3pWvsrz83WH` | writable | 0 |
| 6 | `AfoTwYyKciehbEvwqeja9Sm4VWSL2G1GetXjSWgYi912` | writable | -1.156991838 |
| 7 | `D4fPK2fTXd7FfPJiBP3sig64ywYwVcY4sTR21Aw1vc4H` | writable | 0 |
| 8 | `GWjApTsD4xn8PKuRuKeczdfKvWd6dyAJLguNo84qvBkf` | writable | 0 |
| 9 | `11111111111111111111111111111111` | read-only | 0 |
| 10 | `ComputeBudget111111111111111111111111111111` | read-only, program | 0 |
| 11 | `MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr` | read-only | 0 |
| 12 | `So11111111111111111111111111111111111111112` | read-only | 0 |
| 13 | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | read-only, program | 0 |
| 14 | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | read-only | 0 |
| 15 | `9iFER3bpjf1PTTCQCfTRu17EJgvsxo9pVyA9QWwEuX4x` | read-only | 0 |
| 16 | `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL` | read-only, program | 0 |
| 17 | `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK` | read-only, program | 0 |
| 18 | `HxCnwFKFSCNdLbDeX4AYfJiSyEwdKiKvtAdiTGMJJwfo` | read-only | 0 |

**Instructions**

| # | Program | Type | Details |
|---|---|---|---|
| 1 | Compute Budget | raw | data (base58, 6 chars): Kq1GWK |
| 2 | Compute Budget | raw | data (base58, 12 chars): 3EJZY1JD1woV |
| 3 | Associated Token Account | createIdempotent | {"account":"42uyxKNeaHMbGkags8H5G2HnEiF39njvsyC8VTWfSEgy","mint":"HxCnwFKFSCNdLbDeX4AYfJiSyEwdKiKvtAdiTGMJJwfo","source":"7YiS8wVM4B2wS1CSfncWX4n362jyH9yJc8YA18 |
| 4 | Associated Token Account | createIdempotent | {"account":"3pSY1sdZ7QteSuQir2njHSFwT61U5vPawr74nJkT8qSu","mint":"So11111111111111111111111111111111111111112","source":"7YiS8wVM4B2wS1CSfncWX4n362jyH9yJc8YA18B |
| 4.1 (CPI) | SPL Token | getAccountDataSize | {"extensionTypes":["immutableOwner"],"mint":"So11111111111111111111111111111111111111112"} |
| 4.2 (CPI) | System Program | createAccount | {"lamports":1488440,"newAccount":"3pSY1sdZ7QteSuQir2njHSFwT61U5vPawr74nJkT8qSu","owner":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA","source":"7YiS8wVM4B2wS1CS |
| 4.3 (CPI) | SPL Token | initializeImmutableOwner | {"account":"3pSY1sdZ7QteSuQir2njHSFwT61U5vPawr74nJkT8qSu"} |
| 4.4 (CPI) | SPL Token | initializeAccount3 | {"account":"3pSY1sdZ7QteSuQir2njHSFwT61U5vPawr74nJkT8qSu","mint":"So11111111111111111111111111111111111111112","owner":"7YiS8wVM4B2wS1CSfncWX4n362jyH9yJc8YA18Bt |
| 5 | Raydium CLMM | raw | data (base58, 56 chars): ASCsAbe1UnDoeAskHd9CKqLoCetSLeefodVSWbkSKcqtLS6myo3j7Pxc |
| 5.1 (CPI) | SPL Token | transferChecked | {"authority":"7YiS8wVM4B2wS1CSfncWX4n362jyH9yJc8YA18BtcksH","destination":"GWjApTsD4xn8PKuRuKeczdfKvWd6dyAJLguNo84qvBkf","mint":"HxCnwFKFSCNdLbDeX4AYfJiSyEwdKiK |
| 5.2 (CPI) | SPL Token | transferChecked | {"authority":"6w9FvmaPTWLH8H2EN8DR3qWi6YZC8qMNT3pWvsrz83WH","destination":"3pSY1sdZ7QteSuQir2njHSFwT61U5vPawr74nJkT8qSu","mint":"So11111111111111111111111111111 |
| 6 | SPL Token | closeAccount | {"account":"3pSY1sdZ7QteSuQir2njHSFwT61U5vPawr74nJkT8qSu","destination":"7YiS8wVM4B2wS1CSfncWX4n362jyH9yJc8YA18BtcksH","owner":"7YiS8wVM4B2wS1CSfncWX4n362jyH9yJ |

**Programs**

- Compute Budget `ComputeBudget111111111111111111111111111111`: 2 call(s)
- Associated Token Account `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL`: 2 call(s)
- SPL Token `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`: 6 call(s)
- System Program `11111111111111111111111111111111`: 1 call(s)
- Raydium CLMM `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK`: 1 call(s)

**Token balance changes**

- owner `7YiS8wVM4B2wS1CSfncWX4n362jyH9yJc8YA18BtcksH`, mint `HxCnwFKFSCNdLbDeX4AYfJiSyEwdKiKvtAdiTGMJJwfo`: -334384.502534
- owner `6w9FvmaPTWLH8H2EN8DR3qWi6YZC8qMNT3pWvsrz83WH`, mint `So11111111111111111111111111111111111111112`: -1.1569918380000104
- owner `6w9FvmaPTWLH8H2EN8DR3qWi6YZC8qMNT3pWvsrz83WH`, mint `HxCnwFKFSCNdLbDeX4AYfJiSyEwdKiKvtAdiTGMJJwfo`: +334384.50253400207

**Выводы**

- В транзакции 19 аккаунтов: 1 подписант(ов), 9 writable, остальные read-only.
- Верхнеуровневых инструкций: 6, вложенных вызовов (CPI): 6.
- Задействованные программы: Compute Budget, Associated Token Account, SPL Token, System Program, Raydium CLMM.
- Комиссию 6000 лампортов заплатил fee payer `7YiS8wVM4B2wS1CSfncWX4n362jyH9yJc8YA18BtcksH`.

### Случай 3. Создание Associated Token Account

- Signature: [`4xFVZSGDMexNWS5GHKs9...`](https://explorer.solana.com/tx/4xFVZSGDMexNWS5GHKs92JR9rijwQgntohgoHemb8VQefZ7kzUsmMoWWQfJAxajyPoH4rQF3Puee6vMKAAcU5dTV)
- Cluster: mainnet, slot 451926657, 2026-09-30T09:39:18.000Z
- Status: Success; fee 5000 lamports; compute units 103389; version 0
- Signers: `8Wawir6raLJJq1rcap2p5fNxRKfQgs9uYa6t6tv4acPG`

**Accounts**

| # | Address | Roles | SOL change |
|---|---|---|---|
| 0 | `8Wawir6raLJJq1rcap2p5fNxRKfQgs9uYa6t6tv4acPG` | fee payer, signer, writable | -0.04116639 |
| 1 | `9f5RgNbHUy4s3aZ1JLRfLadvRJVjQ4MjnTS9acYxGUaN` | writable | 0.00148844 |
| 2 | `29aak7D53VkmSVzf5jUubwqdnGfDrqBHxwxviU6ht6Ts` | writable | 0 |
| 3 | `HYHKGs8ZafshRcAtTWT6QLD5nsbRZcqVmDnvWwm1ZD9` | writable | 0 |
| 4 | `8AYiMvs7CyqNprcYizBBVBq3czE5KeAG8toNiNW1x64f` | writable | 0 |
| 5 | `AGA3QGRHxkz5ic7j9AtpaN12cNHyqFjM2iGfoJXG3XzB` | writable | 0 |
| 6 | `9SASE4tF68pZbvrFKQyX3n7ckEEpBCgyHoqLnKwm71i4` | writable | 0.03967295 |
| 7 | `EMzqb8465LmMur5rd5h4UFQr3r4KSQoJLdsfiHiag8NN` | writable | 0 |
| 8 | `45YMK1QkJESdm6v62nVQFqMfUt9fmRa8oAPMm2TZtWYM` | writable | 0 |
| 9 | `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL` | read-only, program | 0 |
| 10 | `4jEhfP9rRHucVkDNJpJBCCAYpqyCAmieqvf9WGouztHV` | read-only | 0 |
| 11 | `11111111111111111111111111111111` | read-only, program | 0 |
| 12 | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | read-only, program | 0 |
| 13 | `So11111111111111111111111111111111111111112` | read-only | 0 |
| 14 | `jitodontfront111111111111111111111111138623` | read-only | 0 |
| 15 | `5YxQFdt3Tr9zJLvkFccqXVUwhdTWJQc1fFg2YPbxvxeD` | read-only | 0 |
| 16 | `8N3GDaZ2iwN65oxVatKTLPNooAVUJTbfiVJ1ahyqwjSk` | read-only | 0 |
| 17 | `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA` | read-only, program | 0 |
| 18 | `ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw` | read-only | 0 |
| 19 | `JCRGumoE9Qi5BBgULTgdgTLjSgkCMSbF62ZZfGs84JeU` | read-only | 0 |
| 20 | `GS4CU59F31iL7aR2Q8zVS8DRrcRnXX1yjQ66TqNVQnaR` | read-only | 0 |
| 21 | `5PHirr8joyTMp9JMm6nW7hNDVyEYdkzDqazxPD7RaTjx` | read-only | 0 |
| 22 | `pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ` | read-only | 0 |

**Instructions**

| # | Program | Type | Details |
|---|---|---|---|
| 1 | Associated Token Account | create | {"account":"9f5RgNbHUy4s3aZ1JLRfLadvRJVjQ4MjnTS9acYxGUaN","mint":"4jEhfP9rRHucVkDNJpJBCCAYpqyCAmieqvf9WGouztHV","source":"8Wawir6raLJJq1rcap2p5fNxRKfQgs9uYa6t6t |
| 1.1 (CPI) | SPL Token | getAccountDataSize | {"extensionTypes":["immutableOwner"],"mint":"4jEhfP9rRHucVkDNJpJBCCAYpqyCAmieqvf9WGouztHV"} |
| 1.2 (CPI) | System Program | createAccount | {"lamports":1488440,"newAccount":"9f5RgNbHUy4s3aZ1JLRfLadvRJVjQ4MjnTS9acYxGUaN","owner":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA","source":"8Wawir6raLJJq1rc |
| 1.3 (CPI) | SPL Token | initializeImmutableOwner | {"account":"9f5RgNbHUy4s3aZ1JLRfLadvRJVjQ4MjnTS9acYxGUaN"} |
| 1.4 (CPI) | SPL Token | initializeAccount3 | {"account":"9f5RgNbHUy4s3aZ1JLRfLadvRJVjQ4MjnTS9acYxGUaN","mint":"4jEhfP9rRHucVkDNJpJBCCAYpqyCAmieqvf9WGouztHV","owner":"8Wawir6raLJJq1rcap2p5fNxRKfQgs9uYa6t6tv |
| 2 | Associated Token Account | createIdempotent | {"account":"29aak7D53VkmSVzf5jUubwqdnGfDrqBHxwxviU6ht6Ts","mint":"So11111111111111111111111111111111111111112","source":"8Wawir6raLJJq1rcap2p5fNxRKfQgs9uYa6t6tv |
| 2.1 (CPI) | SPL Token | getAccountDataSize | {"extensionTypes":["immutableOwner"],"mint":"So11111111111111111111111111111111111111112"} |
| 2.2 (CPI) | System Program | createAccount | {"lamports":1488440,"newAccount":"29aak7D53VkmSVzf5jUubwqdnGfDrqBHxwxviU6ht6Ts","owner":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA","source":"8Wawir6raLJJq1rc |
| 2.3 (CPI) | SPL Token | initializeImmutableOwner | {"account":"29aak7D53VkmSVzf5jUubwqdnGfDrqBHxwxviU6ht6Ts"} |
| 2.4 (CPI) | SPL Token | initializeAccount3 | {"account":"29aak7D53VkmSVzf5jUubwqdnGfDrqBHxwxviU6ht6Ts","mint":"So11111111111111111111111111111111111111112","owner":"8Wawir6raLJJq1rcap2p5fNxRKfQgs9uYa6t6tv4 |
| 3 | System Program | transfer | {"destination":"29aak7D53VkmSVzf5jUubwqdnGfDrqBHxwxviU6ht6Ts","lamports":39672950,"source":"8Wawir6raLJJq1rcap2p5fNxRKfQgs9uYa6t6tv4acPG"} |
| 4 | SPL Token | syncNative | {"account":"29aak7D53VkmSVzf5jUubwqdnGfDrqBHxwxviU6ht6Ts"} |
| 5 | Associated Token Account | createIdempotent | {"account":"9f5RgNbHUy4s3aZ1JLRfLadvRJVjQ4MjnTS9acYxGUaN","mint":"4jEhfP9rRHucVkDNJpJBCCAYpqyCAmieqvf9WGouztHV","source":"8Wawir6raLJJq1rcap2p5fNxRKfQgs9uYa6t6t |
| 6 | Associated Token Account | createIdempotent | {"account":"HYHKGs8ZafshRcAtTWT6QLD5nsbRZcqVmDnvWwm1ZD9","mint":"4jEhfP9rRHucVkDNJpJBCCAYpqyCAmieqvf9WGouztHV","source":"8Wawir6raLJJq1rcap2p5fNxRKfQgs9uYa6t6tv |
| 7 | Associated Token Account | createIdempotent | {"account":"8AYiMvs7CyqNprcYizBBVBq3czE5KeAG8toNiNW1x64f","mint":"4jEhfP9rRHucVkDNJpJBCCAYpqyCAmieqvf9WGouztHV","source":"8Wawir6raLJJq1rcap2p5fNxRKfQgs9uYa6t6t |
| 8 | Pump.fun AMM | raw | data (base58, 33 chars): 5jRcjdixRUDUgg95Qo1HPAkbbxQyWqbfD |
| 8.1 (CPI) | Unknown program | raw | data (base58, 78 chars): KX69VKtJ7u4DvcGpLSLbsjQ36dtvJzoYi6PgHYziKrP93PcRGLAkVHxgw1ps... |
| 8.2 (CPI) | SPL Token | transferChecked | {"authority":"8Wawir6raLJJq1rcap2p5fNxRKfQgs9uYa6t6tv4acPG","destination":"9SASE4tF68pZbvrFKQyX3n7ckEEpBCgyHoqLnKwm71i4","mint":"So11111111111111111111111111111 |
| 8.3 (CPI) | SPL Token | transferChecked | {"authority":"AGA3QGRHxkz5ic7j9AtpaN12cNHyqFjM2iGfoJXG3XzB","destination":"9f5RgNbHUy4s3aZ1JLRfLadvRJVjQ4MjnTS9acYxGUaN","mint":"4jEhfP9rRHucVkDNJpJBCCAYpqyCAmi |
| 8.4 (CPI) | SPL Token | transferChecked | {"authority":"AGA3QGRHxkz5ic7j9AtpaN12cNHyqFjM2iGfoJXG3XzB","destination":"45YMK1QkJESdm6v62nVQFqMfUt9fmRa8oAPMm2TZtWYM","mint":"4jEhfP9rRHucVkDNJpJBCCAYpqyCAmi |
| 8.5 (CPI) | SPL Token | transferChecked | {"authority":"AGA3QGRHxkz5ic7j9AtpaN12cNHyqFjM2iGfoJXG3XzB","destination":"HYHKGs8ZafshRcAtTWT6QLD5nsbRZcqVmDnvWwm1ZD9","mint":"4jEhfP9rRHucVkDNJpJBCCAYpqyCAmie |
| 8.6 (CPI) | Pump.fun AMM | raw | data (base58, 603 chars): 3WpVFTH6N5gEG5rjeG9cr8Jecgohd6NFRkKd2E2MML6CndxSTq4WDWoUmuVH... |
| 9 | SPL Token | closeAccount | {"account":"29aak7D53VkmSVzf5jUubwqdnGfDrqBHxwxviU6ht6Ts","destination":"8Wawir6raLJJq1rcap2p5fNxRKfQgs9uYa6t6tv4acPG","owner":"8Wawir6raLJJq1rcap2p5fNxRKfQgs9u |

**Programs**

- Associated Token Account `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL`: 5 call(s)
- SPL Token `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`: 12 call(s)
- System Program `11111111111111111111111111111111`: 3 call(s)
- Pump.fun AMM `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA`: 2 call(s)
- Unknown program `pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ`: 1 call(s)

**Token balance changes**

- owner `8Wawir6raLJJq1rcap2p5fNxRKfQgs9uYa6t6tv4acPG`, mint `4jEhfP9rRHucVkDNJpJBCCAYpqyCAmieqvf9WGouztHV`: +40121.878228
- owner `5YxQFdt3Tr9zJLvkFccqXVUwhdTWJQc1fFg2YPbxvxeD`, mint `4jEhfP9rRHucVkDNJpJBCCAYpqyCAmieqvf9WGouztHV`: +10.060652002692223
- owner `AGA3QGRHxkz5ic7j9AtpaN12cNHyqFjM2iGfoJXG3XzB`, mint `So11111111111111111111111111111111111111112`: +0.039672949999953744
- owner `AGA3QGRHxkz5ic7j9AtpaN12cNHyqFjM2iGfoJXG3XzB`, mint `4jEhfP9rRHucVkDNJpJBCCAYpqyCAmieqvf9WGouztHV`: -40141.99953198433
- owner `JCRGumoE9Qi5BBgULTgdgTLjSgkCMSbF62ZZfGs84JeU`, mint `4jEhfP9rRHucVkDNJpJBCCAYpqyCAmieqvf9WGouztHV`: +10.060651998966932

**Выводы**

- В транзакции 23 аккаунтов: 1 подписант(ов), 9 writable, остальные read-only.
- Верхнеуровневых инструкций: 9, вложенных вызовов (CPI): 14.
- Задействованные программы: Associated Token Account, SPL Token, System Program, Pump.fun AMM, Unknown program.
- Комиссию 5000 лампортов заплатил fee payer `8Wawir6raLJJq1rcap2p5fNxRKfQgs9uYa6t6tv4acPG`.
