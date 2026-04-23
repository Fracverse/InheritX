# InheritX

**Programmable, Fiat‑Native Digital Inheritance Infrastructure on Stellar**

InheritX is a programmable, cross‑border inheritance and asset succession built on the **Stellar network** using **Soroban smart contracts**. It enables individuals and organizations to securely lock digital assets and automatically transfer them to beneficiaries when predefined real‑world conditions are met, with settlement directly to **local bank accounts or mobile money** via Stellar anchors.

InheritX transforms inheritance from a slow, manual, and legally opaque process into a **transparent, automated, and globally accessible financial lifecycle primitive**.

---

## 🌍 Why InheritX Exists

Inheritance today faces deep structural challenges:

* Assets are locked in legal and bureaucratic processes for months or years
* Cross‑border inheritance is expensive, slow, and unclear
* Existing crypto inheritance tools are:

  * Volatile and token‑speculative
  * Crypto‑native only
  * Not fiat‑settled or compliance‑aware
* Beneficiaries often lack crypto literacy or access to wallets

There is **no global, programmable, fiat‑native inheritance infrastructure**.

**InheritX fills this gap on Stellar.**

---

## 🧠 What InheritX Is (and Is Not)

**InheritX is:**

* Lifecycle financial infrastructure
* Fiat‑native and stablecoin‑based
* Non‑custodial and programmable
* Designed for real‑world settlement

**InheritX is NOT:**

* A DeFi yield or speculation protocol
* A volatile, token‑only inheritance tool

---

## ✨ Core Capabilities (MVP)

### 1. Inheritance Plan Creation

* Create an inheritance plan on‑chain
* Select asset types (USDC on Stellar)
* Define beneficiaries and allocation percentages
* Assign optional guardians

### 2. Asset Locking

* Assets are deposited into a **Soroban‑controlled vault**
* Funds are non‑custodial and programmatically restricted
* Owner cannot withdraw unless unlock conditions are met

### 3. Trigger Conditions

Supported triggers in MVP:

* **Time‑based** (e.g., after X years)
* **Proof‑of‑life failure** (missed periodic check‑ins)


Triggers are evaluated on‑chain or via trusted off‑chain attestations.

### 4. Beneficiary Acknowledgment & Notification

* Plan owners can track notifications sent to beneficiaries
* Beneficiaries can acknowledge their listing on-chain
* Optional requirement for acknowledgment prior to inheritance claim execution

### 5. Beneficiary Settlement

* Assets are released automatically when triggers are satisfied
* Beneficiaries can:

  * Withdraw to Stellar wallets
  * Redeem directly to **local fiat** via Stellar anchors
* Supports bank accounts and mobile money

### 6. Auditability & Transparency

Immutable on‑chain records of:

* Plan creation
* Asset locking
* Trigger execution
* Beneficiary payouts

---

## 🎯 Target Users

### Primary Users

* Diaspora individuals with cross‑border assets
* Families planning estate transfers
* SMEs and founders planning business succession

### Secondary Users

* Legal professionals and trustees
* Estate planners
* NGOs and cooperatives

### Beneficiaries

* Non‑crypto‑native users
* Banked and underbanked individuals
* Mobile money users

---


## 🏗 Technical Architecture

### On‑Chain (Soroban)

* **Inheritance Vault Contract** – locks and releases assets
* **Beneficiary Allocation Contract** – manages distribution logic
* **Trigger Evaluation Contract** – enforces conditions


### Off‑Chain

* Web dashboard (plan creation & monitoring)
* Notification & proof‑of‑life reminder system
* Anchor integration services
* Compliance metadata & document storage

---

## 🤝 Contributing

Contributions are welcome.

- **Clone the repository**
```bash 
git clone https://github.com/Fracverse/InheritX
```

- **Navigate into the project directory**
```bash
cd InheritX
```

- **Deploy Smart Contracts**
Make sure you have an identity set up with `stellar-cli`.
```bash
# Add identity via secret key if you haven't
stellar keys add deployer 

# Run the deployment script (defaults to testnet)
./scripts/deploy.sh --admin deployer

# Or specify network explicitly
./scripts/deploy.sh --network mainnet --admin deployer
```

- **Install dependencies**
```bash 
npm install
```

- **Start the development server**
```bash 
npm run dev
```
---

## 🔐 Security Model

* Fully non‑custodial asset control
* Explicit and auditable state transitions
* No reentrancy or hidden execution paths
* Upgradeability governed by protocol governance

---

## 💸 Pricing Model

* **0%** fee on plan creation
* **1%** fee on plan modification
* **2%** fee on successful inheritance execution

Fees are deducted only at settlement.

---

## 📊 Success Metrics (KPIs)

* Number of inheritance plans created
* Total Value Locked (TVL)
* Successful settlements executed
* Average settlement time
* Anchor payout success rate

---


## 🛣 Roadmap

### Phase 1 – MVP

* Stellar testnet deployment
* USDC‑based inheritance plans
* Time & guardian triggers
* Manual anchor payout

### Phase 2 – Expansion

* Multi‑currency support
* Automated anchor settlement
* Jurisdiction & compliance tagging

### Phase 3 – Institutional

* Legal & estate partnerships
* Enterprise dashboards
* White‑label offerings

---

## 🔮 Long‑Term Vision

InheritX aims to become the **global standard for programmable, cross‑border inheritance** — where value moves seamlessly from one generation to the next, without friction, borders, or uncertainty.

---

**Built on Stellar. Designed for the real world.**
