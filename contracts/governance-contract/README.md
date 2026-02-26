# Governance Parameter Contract

This contract manages global parameters for the InheritX protocol, including:
- Interest Rate
- Collateral Ratio
- Liquidation Bonus

## Implementation Details

- **Admin/Governance**: Only the authorized admin address can update these parameters.
- **Access Control**: Every update function (`update_interest_rate`, `update_collateral_ratio`, `update_liquidation_bonus`) requires administrative authorization via `require_auth()`.

## Storage
Values are stored in the contract's instance storage to ensure they are accessible but protected.
