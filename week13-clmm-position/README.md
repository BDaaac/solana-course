# Неделя 13. Позиция в CLMM-пуле Raydium

**Задание:** открыть из vault позицию в выбранном диапазоне, собрать комиссии, закрыть позицию и переставить её при выходе цены из диапазона.

Код: [`open_position.rs`](../liquidity-vault/programs/simple_vault/src/instructions/open_position.rs), [`collect_fees.rs`](../liquidity-vault/programs/simple_vault/src/instructions/collect_fees.rs), [`close_position.rs`](../liquidity-vault/programs/simple_vault/src/instructions/close_position.rs); клиент [`04-position.ts`](../liquidity-vault/scripts/04-position.ts) (`npm run vault:position`). Все действия — только для admin.

## Инструкции

- `open_raydium_position(tick_lower, tick_upper, …, liquidity, amount_0_max, amount_1_max)` — CPI `open_position_with_token22_nft`. NFT позиции принадлежит PDA vault. `amount_*_max` — максимум токенов, который vault готов отдать (защита). Raydium списывает токены от имени плательщика, поэтому PDA на время CPI делает админа delegate (`approve`) на нужные суммы и затем отзывает (`revoke`).
- `collect_fees` — CPI `decrease_liquidity_v2` с `liquidity = 0`: позиция не уменьшается, комиссии переводятся на счета vault.
- `close_raydium_position(amount_0_min, amount_1_min)` — выводит всю ликвидность с минимумом и закрывает позицию (сжигает NFT).

## Сценарий в devnet

1. **Открытие.** Цена 0,99981, диапазон **0,9498 … 1,0498** (±5 %). Внесено 80 VLT и 84,199006 QTE — [tx](https://explorer.solana.com/tx/3ErXjraLpC3BSwncbwgjbnuEqXUEEYzLSVQZxvQZaqekXKP4SZScT3qzTzMH1uTQ6mi8uZ5joWGMyqv5RG7TpSA2?cluster=devnet), NFT `9EKC4NrY…YCuK`.
2. **Сделки с другого кошелька** (кошелёк админа, не vault): 12 обменов по 4 000 токенов туда-обратно, цена остаётся в диапазоне.
3. **Сбор комиссий** — [tx](https://explorer.solana.com/tx/3ti1FpeQwnefHsCeDugs6WwxYsjLH8J5U3jbSL5BHiRZAK7r6N6ysUh6sXUJvCjgQyxYeW86tLpbEAq7EuAEdQUR?cluster=devnet): **0,031014 VLT + 0,031014 QTE**.
4. **Выход из диапазона.** Серия обменов A→B сдвинула цену до **0,93765** — ниже нижней границы 0,9498.
5. **Закрытие** — [tx](https://explorer.solana.com/tx/2Eta9AxJJ9DnQJ7k3n7CHERmGxu3bCs5xZYHDcchDg9jiBYd9zDA6fD2f15DL7XUcCqazfL7XxMPLvVxnSEcEj1F?cluster=devnet): вернулось **166,461305 VLT и 0 QTE** (минимум был задан: 97,7 / 0). Позиция целиком превратилась в токен A.
6. **Ребалансировка.** Vault меняет 75 VLT → QTE ([tx](https://explorer.solana.com/tx/34fA6Py73tNRNmzLfKv86Np9T6XRG6msM4ueH6eHn5HgnCk5Xz8MKrW6A9VLPizEtafchFm1TYn1JEvY6mxstJp5?cluster=devnet)) и открывает новую позицию вокруг новой цены 0,93752: диапазон **0,8908 … 0,9845**, внесено 80 VLT и 79,514216 QTE — [tx](https://explorer.solana.com/tx/4iVCsgLwVohKPMHeXjbHXDm282gv1q6qELeKKs2r5xt11Egir4huGDVm1YnyRA5pfdTZEWPo3QpE76zCCr2XeYS1?cluster=devnet), NFT [`4fV9S7A9…4hd9`](https://explorer.solana.com/address/4fV9S7A93C4g9vu6TLjFKtQW1ExdcwtUDy24c9Qd4hd9?cluster=devnet).

Позиция №2 оставлена открытой. Чтобы вернуть деньги пользователям: `close_raydium_position` → при необходимости swap B→A → `sync_balance`.

Замечание: комиссии небольшие, потому что доля vault в пуле мала (вместе с ручной позицией 50 000 токенов, добавленной на неделе 11) и объём сделок на devnet символический.

## Проверьте себя

- **Почему соотношение двух токенов в позиции зависит от того, где цена внутри диапазона?** Пока цена движется, обмены меняют один токен позиции на другой: чем ближе цена к нижней границе, тем больше в позиции токена A, и наоборот. Пример выше: позиция 80 A + 84 B превратилась в 166 A, когда цена упала.
- **Что будет, если цена уйдёт выше верхней границы?** Симметрично: позиция состоит только из токена B, перестаёт зарабатывать комиссии до возврата цены или ребалансировки.
- **Зачем нужна ребалансировка и кто её делает?** Позиция вне диапазона не работает. Администратор (в реальных продуктах — бот/keeper) закрывает её и открывает новую вокруг текущей цены; при необходимости предварительно меняет токены в нужной пропорции.
