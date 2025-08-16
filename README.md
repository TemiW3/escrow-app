# Escrow App (Solana Anchor + React/Vite)

A minimal token-for-token escrow on Solana with an Anchor program and a React + Vite frontend scaffold. Makers lock token A into a program-controlled vault and specify how much token B they want. Takers accept an offer by paying token B; the program releases token A to the taker and closes the vault and offer.

- **Program ID**: `GP2mZSjyRK3tX151UHENPLyHbTRqfz7dvYPrQpqJrHzv`
- **Token support**: Uses Anchor SPL Token Interface (Token-2022 compatible).

### Repository structure

```
backend/
  Anchor.toml
  Cargo.toml
  programs/escrow-app/
    src/
      constants.rs
      error.rs
      instructions/
        accept_offer.rs
        make_offer.rs
        shared.rs
      state/
        offer.rs
      lib.rs
  tests/escrow-app.ts
  package.json (TypeScript test deps)
frontend/
  package.json (React + Vite + Tailwind)
  src/
    components/ (wallet adapter, cluster selector, UI shell)
    app.tsx
    main.tsx
```

### How it works (high level)

- **make_offer(id, token_a_offered_amount, token_b_wanted_amount)**
  - Transfers `token_a_offered_amount` of token A from the maker’s ATA to a program-owned vault (ATA where authority is a PDA of the `Offer`).
  - Saves an `Offer` account with the required metadata and bump.
- **accept_offer()**
  - Transfers `token_b_wanted_amount` of token B from the taker to the maker.
  - Releases the full vault balance of token A from the vault to the taker.
  - Closes the vault and `Offer` accounts, sending SOL rent back to the maker.

Both flows use Anchor SPL Token Interface so they work with legacy Token Program and Token-2022.

---

### Smart contract (program) API

- **Instructions**
  - **make_offer(id: u64, token_a_offered_amount: u64, token_b_wanted_amount: u64)**
    - **Accounts**:
      - `maker` (signer)
      - `token_mint_a` (Mint; interface)
      - `token_mint_b` (Mint; interface)
      - `maker_token_account_a` (ATA of `maker` for `token_mint_a`)
      - `offer` (PDA; seeds: `b"offer"`, `maker`, `id`), init
      - `vault` (ATA of `offer` for `token_mint_a`), init
      - `system_program`, `token_program` (TokenInterface), `associated_token_program`
  - **accept_offer()**
    - **Accounts**:
      - `taker` (signer)
      - `maker` (system account)
      - `token_mint_a` (Mint; interface)
      - `token_mint_b` (Mint; interface)
      - `taker_token_account_a` (ATA of `taker` for `token_mint_a`), init_if_needed
      - `taker_token_account_b` (ATA of `taker` for `token_mint_b`), mut
      - `maker_token_account_b` (ATA of `maker` for `token_mint_b`), init_if_needed
      - `offer` (PDA; seeds `b"offer"`, `maker`, `id`; `has_one`: `maker`, `token_mint_a`, `token_mint_b`; `close = maker`), mut
      - `vault` (ATA of `offer` for `token_mint_a`), mut
      - `system_program`, `token_program` (TokenInterface), `associated_token_program`

- **Accounts**
  - `Offer`:
    - `id: u64`
    - `maker: Pubkey`
    - `token_mint_a: Pubkey`
    - `token_mint_b: Pubkey`
    - `token_b_wanted_amount: u64`
    - `bump: u8`

- **PDAs and seeds**
  - `offer` PDA: seeds `[b"offer", maker, id.to_le_bytes()]`
  - `vault` ATA: ATA for `token_mint_a` with authority `offer` PDA

---

### Prerequisites

- Rust toolchain (`rustup`, `cargo`)
- Solana CLI v2.x (to match `solana-program = 2.0.3`)
- Anchor CLI `>= 0.30.1`
- Node.js `>= 18`
- Yarn (for Anchor test script) and/or npm

Useful links:
- [Solana CLI install](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor install](https://www.anchor-lang.com/docs/installation)

---

### Setup

- Configure Solana for local development:

```bash
solana config set --url localhost
solana-keygen new --no-bip39-passphrase # if you need a new local key
```

- Install JavaScript dependencies:

```bash
# Backend (tests and tooling)
cd backend && npm install

# Frontend
cd ../frontend && npm install
```

- Build the program:

```bash
cd ../backend
anchor build
```

---

### Run tests (backend)

Runs a local validator, builds the program, and executes TypeScript tests.

```bash
cd backend
anchor test
```

The test covers the full flow:
- Maker creates an offer and deposits token A into the vault
- Taker accepts the offer, transfers token B, and receives token A

---

### Run locally (frontend)

The frontend is a wallet-enabled scaffold (React + Vite + Tailwind) with a cluster selector. It does not yet call the program, but is ready for integration.

```bash
cd frontend
npm run dev
```

- Use the cluster selector to choose `devnet` or `local` (`http://localhost:8899`).
- Connect a wallet using the wallet modal. For `local`, fund your keypair:

```bash
solana airdrop 2 --url localhost
```

---

### Deploy to devnet

1) Fund your deployer wallet on devnet and set config:
```bash
solana config set --url devnet
solana airdrop 2 --url devnet
```

2) In `backend/Anchor.toml`, configure the provider and program mapping (if needed):

```toml
[provider]
cluster = "Devnet"
wallet = "~/.config/solana/id.json"

[programs.devnet]
escrow_app = "GP2mZSjyRK3tX151UHENPLyHbTRqfz7dvYPrQpqJrHzv"
```

3) Build and deploy:
```bash
cd backend
anchor build
anchor deploy
```

4) Ensure the on-chain program ID matches `declare_id!` in `programs/escrow-app/src/lib.rs`. If Anchor generated a new keypair/program ID, update both `Anchor.toml` and `declare_id!` accordingly, then rebuild and redeploy.

---

### Frontend integration (calling the program)

- Generate the IDL after building: `backend/target/idl/escrow_app.json`.
- Use `@coral-xyz/anchor` in the frontend with your wallet adapter to create a `Program` client and call `makeOffer` / `acceptOffer`.
- You’ll need to compute the PDA for `offer` and derive the `vault` ATA the same way as in the backend tests.

Minimal outline:
```ts
// in the browser
import { AnchorProvider, Program, Idl } from '@coral-xyz/anchor'
import idl from '../../backend/target/idl/escrow_app.json'

const provider = new AnchorProvider(connection, wallet, {})
const programId = new PublicKey('GP2mZS...')
const program = new Program(idl as Idl, programId, provider)

// program.methods.makeOffer(...).accounts({...}).rpc()
```

See `backend/tests/escrow-app.ts` for a complete reference on deriving PDAs and ATAs.

---

### Scripts

- Backend (from `backend/`):
  - `anchor build` – build the program
  - `anchor test` – spin up local validator and run tests
  - `npm run lint` / `npm run lint:fix` – Prettier checks (JS/TS tests)
- Frontend (from `frontend/`):
  - `npm run dev` – start Vite dev server
  - `npm run build` – build for production
  - `npm run preview` – preview production build
  - `npm run lint` – ESLint
  - `npm run format` / `npm run format:check` – Prettier

---

### Troubleshooting

- **Program ID mismatch**: If `declare_id!` in Rust doesn’t match the program keypair used for deployment, transactions will fail. Update `declare_id!` and `Anchor.toml` program mapping to the deployed address and rebuild.
- **Token program selection**: The tests use Token-2022. If you need legacy Token Program, adjust constants and ATAs accordingly. The program itself supports both via Token Interface.
- **Wallet and airdrops**: Devnet faucets may rate-limit. Try smaller airdrops or different RPC endpoints.
- **Local RPC**: Ensure `solana-test-validator` is running if directly hitting `localhost:8899` from the frontend.

---

### Security notes

This code is for educational/demo purposes and has not been audited. It does not implement features like offer cancellation, expirations, fees, or partial fills. Use at your own risk; deploy only to devnet/test environments unless you have performed a thorough security review and audit.