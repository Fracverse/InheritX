# Asset Freezing and Compliance Procedures

This document outlines the procedures for freezing assets and managing compliance holds within the InheritX Inheritance Contract.

## Overview

To comply with AML/sanctions regulations and legal requirements, the InheritX contract provides administrative functions to freeze inheritance plans or specific beneficiaries. Freezing prevents any withdrawals or claims from being processed until the freeze is lifted.

## Authorized Roles

Only the initialized **Contract Admin** or a designated **Compliance Officer** (if implemented via multi-sig or separate role) has the authority to:
- Freeze/Unfreeze a plan
- Add/Remove legal holds
- Freeze/Unfreeze a specific beneficiary

## Freezing a Plan

When a plan is frozen, the following actions are blocked:
- **Withdrawals**: The plan owner cannot withdraw funds.
- **Claims**: Beneficiaries cannot claim their inheritance.

### Procedure
1. Identify the `plan_id` to be frozen.
2. Provide a clear `reason` for the freeze (e.g., "Sanctions match", "Suspicious activity").
3. Call `freeze_plan(admin, plan_id, reason)`.

## Legal Holds

A legal hold is a specific type of freeze used for court orders or pending legal disputes.

### Procedure
1. Call `add_legal_hold(admin, plan_id)`.
2. This automatically sets the reason to "Legal Hold" and freezes the plan.
3. To lift the hold, call `remove_legal_hold(admin, plan_id)`.

## Freezing a Beneficiary

If only a specific beneficiary is flagged (e.g., during KYC/AML verification), their specific claim can be blocked without affecting other beneficiaries of the same plan.

### Procedure
1. Identify the `plan_id` and the beneficiary's `hashed_email`.
2. Provide a `reason`.
3. Call `freeze_beneficiary(admin, plan_id, hashed_email, reason)`.

## Unfreezing

Once a compliance review is completed or a legal order is lifted, assets should be unfrozen promptly.

### Procedure
1. Call `unfreeze_plan(admin, plan_id)` to lift a plan-level freeze.
2. Call `remove_legal_hold(admin, plan_id)` for legal holds.
3. Call `unfreeze_beneficiary(admin, plan_id, hashed_email)` to lift a beneficiary-level freeze.

## Event Logging

All freezing and unfreezing actions emit events for auditability:
- `PlanFrozen`: Emitted when a plan is frozen.
- `PlanUnfrozen`: Emitted when a plan is unfrozen.
- `BeneficiaryFrozen`: Emitted when a specific beneficiary is frozen.
- `BeneficiaryUnfrozen`: Emitted when a specific beneficiary is unfrozen.
