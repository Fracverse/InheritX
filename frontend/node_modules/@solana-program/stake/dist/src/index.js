'use strict';

var kit = require('@solana/kit');

// src/generated/accounts/stakeStateAccount.ts
function getAuthorizedEncoder() {
  return kit.getStructEncoder([
    ["staker", kit.getAddressEncoder()],
    ["withdrawer", kit.getAddressEncoder()]
  ]);
}
function getAuthorizedDecoder() {
  return kit.getStructDecoder([
    ["staker", kit.getAddressDecoder()],
    ["withdrawer", kit.getAddressDecoder()]
  ]);
}
function getAuthorizedCodec() {
  return kit.combineCodec(getAuthorizedEncoder(), getAuthorizedDecoder());
}
function getDelegationEncoder() {
  return kit.getStructEncoder([
    ["voterPubkey", kit.getAddressEncoder()],
    ["stake", kit.getU64Encoder()],
    ["activationEpoch", kit.getU64Encoder()],
    ["deactivationEpoch", kit.getU64Encoder()],
    ["warmupCooldownRate", kit.getF64Encoder()]
  ]);
}
function getDelegationDecoder() {
  return kit.getStructDecoder([
    ["voterPubkey", kit.getAddressDecoder()],
    ["stake", kit.getU64Decoder()],
    ["activationEpoch", kit.getU64Decoder()],
    ["deactivationEpoch", kit.getU64Decoder()],
    ["warmupCooldownRate", kit.getF64Decoder()]
  ]);
}
function getDelegationCodec() {
  return kit.combineCodec(getDelegationEncoder(), getDelegationDecoder());
}
function getLockupEncoder() {
  return kit.getStructEncoder([
    ["unixTimestamp", kit.getI64Encoder()],
    ["epoch", kit.getU64Encoder()],
    ["custodian", kit.getAddressEncoder()]
  ]);
}
function getLockupDecoder() {
  return kit.getStructDecoder([
    ["unixTimestamp", kit.getI64Decoder()],
    ["epoch", kit.getU64Decoder()],
    ["custodian", kit.getAddressDecoder()]
  ]);
}
function getLockupCodec() {
  return kit.combineCodec(getLockupEncoder(), getLockupDecoder());
}
function getMetaEncoder() {
  return kit.getStructEncoder([
    ["rentExemptReserve", kit.getU64Encoder()],
    ["authorized", getAuthorizedEncoder()],
    ["lockup", getLockupEncoder()]
  ]);
}
function getMetaDecoder() {
  return kit.getStructDecoder([
    ["rentExemptReserve", kit.getU64Decoder()],
    ["authorized", getAuthorizedDecoder()],
    ["lockup", getLockupDecoder()]
  ]);
}
function getMetaCodec() {
  return kit.combineCodec(getMetaEncoder(), getMetaDecoder());
}
function getStakeEncoder() {
  return kit.getStructEncoder([
    ["delegation", getDelegationEncoder()],
    ["creditsObserved", kit.getU64Encoder()]
  ]);
}
function getStakeDecoder() {
  return kit.getStructDecoder([
    ["delegation", getDelegationDecoder()],
    ["creditsObserved", kit.getU64Decoder()]
  ]);
}
function getStakeCodec() {
  return kit.combineCodec(getStakeEncoder(), getStakeDecoder());
}
var StakeAuthorize = /* @__PURE__ */ ((StakeAuthorize2) => {
  StakeAuthorize2[StakeAuthorize2["Staker"] = 0] = "Staker";
  StakeAuthorize2[StakeAuthorize2["Withdrawer"] = 1] = "Withdrawer";
  return StakeAuthorize2;
})(StakeAuthorize || {});
function getStakeAuthorizeEncoder() {
  return kit.getEnumEncoder(StakeAuthorize);
}
function getStakeAuthorizeDecoder() {
  return kit.getEnumDecoder(StakeAuthorize);
}
function getStakeAuthorizeCodec() {
  return kit.combineCodec(getStakeAuthorizeEncoder(), getStakeAuthorizeDecoder());
}
function getStakeFlagsEncoder() {
  return kit.getStructEncoder([["bits", kit.getU8Encoder()]]);
}
function getStakeFlagsDecoder() {
  return kit.getStructDecoder([["bits", kit.getU8Decoder()]]);
}
function getStakeFlagsCodec() {
  return kit.combineCodec(getStakeFlagsEncoder(), getStakeFlagsDecoder());
}
function getStakeStateEncoder() {
  return kit.getDiscriminatedUnionEncoder(
    [
      ["Uninitialized", kit.getUnitEncoder()],
      [
        "Initialized",
        kit.getStructEncoder([["fields", kit.getTupleEncoder([getMetaEncoder()])]])
      ],
      [
        "Stake",
        kit.getStructEncoder([
          ["fields", kit.getTupleEncoder([getMetaEncoder(), getStakeEncoder()])]
        ])
      ],
      ["RewardsPool", kit.getUnitEncoder()]
    ],
    { size: kit.getU32Encoder() }
  );
}
function getStakeStateDecoder() {
  return kit.getDiscriminatedUnionDecoder(
    [
      ["Uninitialized", kit.getUnitDecoder()],
      [
        "Initialized",
        kit.getStructDecoder([["fields", kit.getTupleDecoder([getMetaDecoder()])]])
      ],
      [
        "Stake",
        kit.getStructDecoder([
          ["fields", kit.getTupleDecoder([getMetaDecoder(), getStakeDecoder()])]
        ])
      ],
      ["RewardsPool", kit.getUnitDecoder()]
    ],
    { size: kit.getU32Decoder() }
  );
}
function getStakeStateCodec() {
  return kit.combineCodec(getStakeStateEncoder(), getStakeStateDecoder());
}
function stakeState(kind, data) {
  return Array.isArray(data) ? { __kind: kind, fields: data } : { __kind: kind, ...data ?? {} };
}
function isStakeState(kind, value) {
  return value.__kind === kind;
}
function getStakeStateV2Encoder() {
  return kit.getDiscriminatedUnionEncoder(
    [
      ["Uninitialized", kit.getUnitEncoder()],
      [
        "Initialized",
        kit.getStructEncoder([["fields", kit.getTupleEncoder([getMetaEncoder()])]])
      ],
      [
        "Stake",
        kit.getStructEncoder([
          [
            "fields",
            kit.getTupleEncoder([
              getMetaEncoder(),
              getStakeEncoder(),
              getStakeFlagsEncoder()
            ])
          ]
        ])
      ],
      ["RewardsPool", kit.getUnitEncoder()]
    ],
    { size: kit.getU32Encoder() }
  );
}
function getStakeStateV2Decoder() {
  return kit.getDiscriminatedUnionDecoder(
    [
      ["Uninitialized", kit.getUnitDecoder()],
      [
        "Initialized",
        kit.getStructDecoder([["fields", kit.getTupleDecoder([getMetaDecoder()])]])
      ],
      [
        "Stake",
        kit.getStructDecoder([
          [
            "fields",
            kit.getTupleDecoder([
              getMetaDecoder(),
              getStakeDecoder(),
              getStakeFlagsDecoder()
            ])
          ]
        ])
      ],
      ["RewardsPool", kit.getUnitDecoder()]
    ],
    { size: kit.getU32Decoder() }
  );
}
function getStakeStateV2Codec() {
  return kit.combineCodec(getStakeStateV2Encoder(), getStakeStateV2Decoder());
}
function stakeStateV2(kind, data) {
  return Array.isArray(data) ? { __kind: kind, fields: data } : { __kind: kind, ...data ?? {} };
}
function isStakeStateV2(kind, value) {
  return value.__kind === kind;
}

// src/generated/accounts/stakeStateAccount.ts
function getStakeStateAccountEncoder() {
  return kit.getStructEncoder([["state", getStakeStateV2Encoder()]]);
}
function getStakeStateAccountDecoder() {
  return kit.getStructDecoder([["state", getStakeStateV2Decoder()]]);
}
function getStakeStateAccountCodec() {
  return kit.combineCodec(
    getStakeStateAccountEncoder(),
    getStakeStateAccountDecoder()
  );
}
function decodeStakeStateAccount(encodedAccount) {
  return kit.decodeAccount(
    encodedAccount,
    getStakeStateAccountDecoder()
  );
}
async function fetchStakeStateAccount(rpc, address, config) {
  const maybeAccount = await fetchMaybeStakeStateAccount(rpc, address, config);
  kit.assertAccountExists(maybeAccount);
  return maybeAccount;
}
async function fetchMaybeStakeStateAccount(rpc, address, config) {
  const maybeAccount = await kit.fetchEncodedAccount(rpc, address, config);
  return decodeStakeStateAccount(maybeAccount);
}
async function fetchAllStakeStateAccount(rpc, addresses, config) {
  const maybeAccounts = await fetchAllMaybeStakeStateAccount(
    rpc,
    addresses,
    config
  );
  kit.assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}
async function fetchAllMaybeStakeStateAccount(rpc, addresses, config) {
  const maybeAccounts = await kit.fetchEncodedAccounts(rpc, addresses, config);
  return maybeAccounts.map(
    (maybeAccount) => decodeStakeStateAccount(maybeAccount)
  );
}
var STAKE_PROGRAM_ADDRESS = "Stake11111111111111111111111111111111111111";
var StakeAccount = /* @__PURE__ */ ((StakeAccount2) => {
  StakeAccount2[StakeAccount2["StakeStateAccount"] = 0] = "StakeStateAccount";
  return StakeAccount2;
})(StakeAccount || {});
var StakeInstruction = /* @__PURE__ */ ((StakeInstruction2) => {
  StakeInstruction2[StakeInstruction2["Initialize"] = 0] = "Initialize";
  StakeInstruction2[StakeInstruction2["Authorize"] = 1] = "Authorize";
  StakeInstruction2[StakeInstruction2["DelegateStake"] = 2] = "DelegateStake";
  StakeInstruction2[StakeInstruction2["Split"] = 3] = "Split";
  StakeInstruction2[StakeInstruction2["Withdraw"] = 4] = "Withdraw";
  StakeInstruction2[StakeInstruction2["Deactivate"] = 5] = "Deactivate";
  StakeInstruction2[StakeInstruction2["SetLockup"] = 6] = "SetLockup";
  StakeInstruction2[StakeInstruction2["Merge"] = 7] = "Merge";
  StakeInstruction2[StakeInstruction2["AuthorizeWithSeed"] = 8] = "AuthorizeWithSeed";
  StakeInstruction2[StakeInstruction2["InitializeChecked"] = 9] = "InitializeChecked";
  StakeInstruction2[StakeInstruction2["AuthorizeChecked"] = 10] = "AuthorizeChecked";
  StakeInstruction2[StakeInstruction2["AuthorizeCheckedWithSeed"] = 11] = "AuthorizeCheckedWithSeed";
  StakeInstruction2[StakeInstruction2["SetLockupChecked"] = 12] = "SetLockupChecked";
  StakeInstruction2[StakeInstruction2["GetMinimumDelegation"] = 13] = "GetMinimumDelegation";
  StakeInstruction2[StakeInstruction2["DeactivateDelinquent"] = 14] = "DeactivateDelinquent";
  StakeInstruction2[StakeInstruction2["MoveStake"] = 15] = "MoveStake";
  StakeInstruction2[StakeInstruction2["MoveLamports"] = 16] = "MoveLamports";
  return StakeInstruction2;
})(StakeInstruction || {});
function identifyStakeInstruction(instruction) {
  const data = "data" in instruction ? instruction.data : instruction;
  if (kit.containsBytes(data, kit.getU32Encoder().encode(0), 0)) {
    return 0 /* Initialize */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(1), 0)) {
    return 1 /* Authorize */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(2), 0)) {
    return 2 /* DelegateStake */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(3), 0)) {
    return 3 /* Split */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(4), 0)) {
    return 4 /* Withdraw */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(5), 0)) {
    return 5 /* Deactivate */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(6), 0)) {
    return 6 /* SetLockup */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(7), 0)) {
    return 7 /* Merge */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(8), 0)) {
    return 8 /* AuthorizeWithSeed */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(9), 0)) {
    return 9 /* InitializeChecked */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(10), 0)) {
    return 10 /* AuthorizeChecked */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(11), 0)) {
    return 11 /* AuthorizeCheckedWithSeed */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(12), 0)) {
    return 12 /* SetLockupChecked */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(13), 0)) {
    return 13 /* GetMinimumDelegation */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(14), 0)) {
    return 14 /* DeactivateDelinquent */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(16), 0)) {
    return 15 /* MoveStake */;
  }
  if (kit.containsBytes(data, kit.getU32Encoder().encode(17), 0)) {
    return 16 /* MoveLamports */;
  }
  throw new Error(
    "The provided instruction could not be identified as a stake instruction."
  );
}

// src/generated/errors/stake.ts
var STAKE_ERROR__NO_CREDITS_TO_REDEEM = 0;
var STAKE_ERROR__LOCKUP_IN_FORCE = 1;
var STAKE_ERROR__ALREADY_DEACTIVATED = 2;
var STAKE_ERROR__TOO_SOON_TO_REDELEGATE = 3;
var STAKE_ERROR__INSUFFICIENT_STAKE = 4;
var STAKE_ERROR__MERGE_TRANSIENT_STAKE = 5;
var STAKE_ERROR__MERGE_MISMATCH = 6;
var STAKE_ERROR__CUSTODIAN_MISSING = 7;
var STAKE_ERROR__CUSTODIAN_SIGNATURE_MISSING = 8;
var STAKE_ERROR__INSUFFICIENT_REFERENCE_VOTES = 9;
var STAKE_ERROR__VOTE_ADDRESS_MISMATCH = 10;
var STAKE_ERROR__MINIMUM_DELINQUENT_EPOCHS_FOR_DEACTIVATION_NOT_MET = 11;
var STAKE_ERROR__INSUFFICIENT_DELEGATION = 12;
var STAKE_ERROR__REDELEGATE_TRANSIENT_OR_INACTIVE_STAKE = 13;
var STAKE_ERROR__REDELEGATE_TO_SAME_VOTE_ACCOUNT = 14;
var STAKE_ERROR__REDELEGATED_STAKE_MUST_FULLY_ACTIVATE_BEFORE_DEACTIVATION_IS_PERMITTED = 15;
var STAKE_ERROR__EPOCH_REWARDS_ACTIVE = 16;
var stakeErrorMessages;
if (process.env.NODE_ENV !== "production") {
  stakeErrorMessages = {
    [STAKE_ERROR__ALREADY_DEACTIVATED]: `Stake already deactivated`,
    [STAKE_ERROR__CUSTODIAN_MISSING]: `Custodian address not present`,
    [STAKE_ERROR__CUSTODIAN_SIGNATURE_MISSING]: `Custodian signature not present`,
    [STAKE_ERROR__EPOCH_REWARDS_ACTIVE]: `Stake action is not permitted while the epoch rewards period is active`,
    [STAKE_ERROR__INSUFFICIENT_DELEGATION]: `Delegation amount is less than the minimum`,
    [STAKE_ERROR__INSUFFICIENT_REFERENCE_VOTES]: `Insufficient voting activity in the reference vote account`,
    [STAKE_ERROR__INSUFFICIENT_STAKE]: `Split amount is more than is staked`,
    [STAKE_ERROR__LOCKUP_IN_FORCE]: `Lockup has not yet expired`,
    [STAKE_ERROR__MERGE_MISMATCH]: `Stake account merge failed due to different authority, lockups or state`,
    [STAKE_ERROR__MERGE_TRANSIENT_STAKE]: `Stake account with transient stake cannot be merged`,
    [STAKE_ERROR__MINIMUM_DELINQUENT_EPOCHS_FOR_DEACTIVATION_NOT_MET]: `Stake account has not been delinquent for the minimum epochs required for deactivation`,
    [STAKE_ERROR__NO_CREDITS_TO_REDEEM]: `Not enough credits to redeem`,
    [STAKE_ERROR__REDELEGATED_STAKE_MUST_FULLY_ACTIVATE_BEFORE_DEACTIVATION_IS_PERMITTED]: `Redelegated stake must be fully activated before deactivation`,
    [STAKE_ERROR__REDELEGATE_TO_SAME_VOTE_ACCOUNT]: `Stake redelegation to the same vote account is not permitted`,
    [STAKE_ERROR__REDELEGATE_TRANSIENT_OR_INACTIVE_STAKE]: `Stake account with transient or inactive stake cannot be redelegated`,
    [STAKE_ERROR__TOO_SOON_TO_REDELEGATE]: `One re-delegation permitted per epoch`,
    [STAKE_ERROR__VOTE_ADDRESS_MISMATCH]: `Stake account is not delegated to the provided vote account`
  };
}
function getStakeErrorMessage(code) {
  if (process.env.NODE_ENV !== "production") {
    return stakeErrorMessages[code];
  }
  return "Error message not available in production bundles.";
}
function isStakeError(error, transactionMessage, code) {
  return kit.isProgramError(
    error,
    transactionMessage,
    STAKE_PROGRAM_ADDRESS,
    code
  );
}
function expectAddress(value) {
  if (!value) {
    throw new Error("Expected a Address.");
  }
  if (typeof value === "object" && "address" in value) {
    return value.address;
  }
  if (Array.isArray(value)) {
    return value[0];
  }
  return value;
}
function getAccountMetaFactory(programAddress, optionalAccountStrategy) {
  return (account) => {
    if (!account.value) {
      return;
    }
    const writableRole = account.isWritable ? kit.AccountRole.WRITABLE : kit.AccountRole.READONLY;
    return Object.freeze({
      address: expectAddress(account.value),
      role: isTransactionSigner(account.value) ? kit.upgradeRoleToSigner(writableRole) : writableRole,
      ...isTransactionSigner(account.value) ? { signer: account.value } : {}
    });
  };
}
function isTransactionSigner(value) {
  return !!value && typeof value === "object" && "address" in value && kit.isTransactionSigner(value);
}

// src/generated/instructions/authorize.ts
var AUTHORIZE_DISCRIMINATOR = 1;
function getAuthorizeDiscriminatorBytes() {
  return kit.getU32Encoder().encode(AUTHORIZE_DISCRIMINATOR);
}
function getAuthorizeInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([
      ["discriminator", kit.getU32Encoder()],
      ["arg0", kit.getAddressEncoder()],
      ["arg1", getStakeAuthorizeEncoder()]
    ]),
    (value) => ({ ...value, discriminator: AUTHORIZE_DISCRIMINATOR })
  );
}
function getAuthorizeInstructionDataDecoder() {
  return kit.getStructDecoder([
    ["discriminator", kit.getU32Decoder()],
    ["arg0", kit.getAddressDecoder()],
    ["arg1", getStakeAuthorizeDecoder()]
  ]);
}
function getAuthorizeInstructionDataCodec() {
  return kit.combineCodec(
    getAuthorizeInstructionDataEncoder(),
    getAuthorizeInstructionDataDecoder()
  );
}
function getAuthorizeInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    stake: { value: input.stake ?? null, isWritable: true },
    clockSysvar: { value: input.clockSysvar ?? null, isWritable: false },
    authority: { value: input.authority ?? null, isWritable: false },
    lockupAuthority: {
      value: input.lockupAuthority ?? null,
      isWritable: false
    }
  };
  const accounts = originalAccounts;
  const args = { ...input };
  if (!accounts.clockSysvar.value) {
    accounts.clockSysvar.value = "SysvarC1ock11111111111111111111111111111111";
  }
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.clockSysvar),
      getAccountMeta(accounts.authority),
      getAccountMeta(accounts.lockupAuthority)
    ].filter((x) => x !== void 0),
    programAddress,
    data: getAuthorizeInstructionDataEncoder().encode(
      args
    )
  };
  return instruction;
}
function parseAuthorizeInstruction(instruction) {
  if (instruction.accounts.length < 3) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  let optionalAccountsRemaining = instruction.accounts.length - 3;
  const getNextOptionalAccount = () => {
    if (optionalAccountsRemaining === 0) return void 0;
    optionalAccountsRemaining -= 1;
    return getNextAccount();
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stake: getNextAccount(),
      clockSysvar: getNextAccount(),
      authority: getNextAccount(),
      lockupAuthority: getNextOptionalAccount()
    },
    data: getAuthorizeInstructionDataDecoder().decode(instruction.data)
  };
}
var AUTHORIZE_CHECKED_DISCRIMINATOR = 10;
function getAuthorizeCheckedDiscriminatorBytes() {
  return kit.getU32Encoder().encode(AUTHORIZE_CHECKED_DISCRIMINATOR);
}
function getAuthorizeCheckedInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([
      ["discriminator", kit.getU32Encoder()],
      ["stakeAuthorize", getStakeAuthorizeEncoder()]
    ]),
    (value) => ({ ...value, discriminator: AUTHORIZE_CHECKED_DISCRIMINATOR })
  );
}
function getAuthorizeCheckedInstructionDataDecoder() {
  return kit.getStructDecoder([
    ["discriminator", kit.getU32Decoder()],
    ["stakeAuthorize", getStakeAuthorizeDecoder()]
  ]);
}
function getAuthorizeCheckedInstructionDataCodec() {
  return kit.combineCodec(
    getAuthorizeCheckedInstructionDataEncoder(),
    getAuthorizeCheckedInstructionDataDecoder()
  );
}
function getAuthorizeCheckedInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    stake: { value: input.stake ?? null, isWritable: true },
    clockSysvar: { value: input.clockSysvar ?? null, isWritable: false },
    authority: { value: input.authority ?? null, isWritable: false },
    newAuthority: { value: input.newAuthority ?? null, isWritable: false },
    lockupAuthority: {
      value: input.lockupAuthority ?? null,
      isWritable: false
    }
  };
  const accounts = originalAccounts;
  const args = { ...input };
  if (!accounts.clockSysvar.value) {
    accounts.clockSysvar.value = "SysvarC1ock11111111111111111111111111111111";
  }
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.clockSysvar),
      getAccountMeta(accounts.authority),
      getAccountMeta(accounts.newAuthority),
      getAccountMeta(accounts.lockupAuthority)
    ].filter((x) => x !== void 0),
    programAddress,
    data: getAuthorizeCheckedInstructionDataEncoder().encode(
      args
    )
  };
  return instruction;
}
function parseAuthorizeCheckedInstruction(instruction) {
  if (instruction.accounts.length < 4) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  let optionalAccountsRemaining = instruction.accounts.length - 4;
  const getNextOptionalAccount = () => {
    if (optionalAccountsRemaining === 0) return void 0;
    optionalAccountsRemaining -= 1;
    return getNextAccount();
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stake: getNextAccount(),
      clockSysvar: getNextAccount(),
      authority: getNextAccount(),
      newAuthority: getNextAccount(),
      lockupAuthority: getNextOptionalAccount()
    },
    data: getAuthorizeCheckedInstructionDataDecoder().decode(instruction.data)
  };
}
var AUTHORIZE_CHECKED_WITH_SEED_DISCRIMINATOR = 11;
function getAuthorizeCheckedWithSeedDiscriminatorBytes() {
  return kit.getU32Encoder().encode(AUTHORIZE_CHECKED_WITH_SEED_DISCRIMINATOR);
}
function getAuthorizeCheckedWithSeedInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([
      ["discriminator", kit.getU32Encoder()],
      ["stakeAuthorize", getStakeAuthorizeEncoder()],
      [
        "authoritySeed",
        kit.addEncoderSizePrefix(kit.getUtf8Encoder(), kit.getU32Encoder())
      ],
      ["authorityOwner", kit.getAddressEncoder()]
    ]),
    (value) => ({
      ...value,
      discriminator: AUTHORIZE_CHECKED_WITH_SEED_DISCRIMINATOR
    })
  );
}
function getAuthorizeCheckedWithSeedInstructionDataDecoder() {
  return kit.getStructDecoder([
    ["discriminator", kit.getU32Decoder()],
    ["stakeAuthorize", getStakeAuthorizeDecoder()],
    ["authoritySeed", kit.addDecoderSizePrefix(kit.getUtf8Decoder(), kit.getU32Decoder())],
    ["authorityOwner", kit.getAddressDecoder()]
  ]);
}
function getAuthorizeCheckedWithSeedInstructionDataCodec() {
  return kit.combineCodec(
    getAuthorizeCheckedWithSeedInstructionDataEncoder(),
    getAuthorizeCheckedWithSeedInstructionDataDecoder()
  );
}
function getAuthorizeCheckedWithSeedInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    stake: { value: input.stake ?? null, isWritable: true },
    base: { value: input.base ?? null, isWritable: false },
    clockSysvar: { value: input.clockSysvar ?? null, isWritable: false },
    newAuthority: { value: input.newAuthority ?? null, isWritable: false },
    lockupAuthority: {
      value: input.lockupAuthority ?? null,
      isWritable: false
    }
  };
  const accounts = originalAccounts;
  const args = { ...input };
  if (!accounts.clockSysvar.value) {
    accounts.clockSysvar.value = "SysvarC1ock11111111111111111111111111111111";
  }
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.base),
      getAccountMeta(accounts.clockSysvar),
      getAccountMeta(accounts.newAuthority),
      getAccountMeta(accounts.lockupAuthority)
    ].filter((x) => x !== void 0),
    programAddress,
    data: getAuthorizeCheckedWithSeedInstructionDataEncoder().encode(
      args
    )
  };
  return instruction;
}
function parseAuthorizeCheckedWithSeedInstruction(instruction) {
  if (instruction.accounts.length < 4) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  let optionalAccountsRemaining = instruction.accounts.length - 4;
  const getNextOptionalAccount = () => {
    if (optionalAccountsRemaining === 0) return void 0;
    optionalAccountsRemaining -= 1;
    return getNextAccount();
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stake: getNextAccount(),
      base: getNextAccount(),
      clockSysvar: getNextAccount(),
      newAuthority: getNextAccount(),
      lockupAuthority: getNextOptionalAccount()
    },
    data: getAuthorizeCheckedWithSeedInstructionDataDecoder().decode(
      instruction.data
    )
  };
}
var AUTHORIZE_WITH_SEED_DISCRIMINATOR = 8;
function getAuthorizeWithSeedDiscriminatorBytes() {
  return kit.getU32Encoder().encode(AUTHORIZE_WITH_SEED_DISCRIMINATOR);
}
function getAuthorizeWithSeedInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([
      ["discriminator", kit.getU32Encoder()],
      ["newAuthorizedPubkey", kit.getAddressEncoder()],
      ["stakeAuthorize", getStakeAuthorizeEncoder()],
      [
        "authoritySeed",
        kit.addEncoderSizePrefix(kit.getUtf8Encoder(), kit.getU32Encoder())
      ],
      ["authorityOwner", kit.getAddressEncoder()]
    ]),
    (value) => ({ ...value, discriminator: AUTHORIZE_WITH_SEED_DISCRIMINATOR })
  );
}
function getAuthorizeWithSeedInstructionDataDecoder() {
  return kit.getStructDecoder([
    ["discriminator", kit.getU32Decoder()],
    ["newAuthorizedPubkey", kit.getAddressDecoder()],
    ["stakeAuthorize", getStakeAuthorizeDecoder()],
    ["authoritySeed", kit.addDecoderSizePrefix(kit.getUtf8Decoder(), kit.getU32Decoder())],
    ["authorityOwner", kit.getAddressDecoder()]
  ]);
}
function getAuthorizeWithSeedInstructionDataCodec() {
  return kit.combineCodec(
    getAuthorizeWithSeedInstructionDataEncoder(),
    getAuthorizeWithSeedInstructionDataDecoder()
  );
}
function getAuthorizeWithSeedInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    stake: { value: input.stake ?? null, isWritable: true },
    base: { value: input.base ?? null, isWritable: false },
    clockSysvar: { value: input.clockSysvar ?? null, isWritable: false },
    lockupAuthority: {
      value: input.lockupAuthority ?? null,
      isWritable: false
    }
  };
  const accounts = originalAccounts;
  const args = { ...input };
  if (!accounts.clockSysvar.value) {
    accounts.clockSysvar.value = "SysvarC1ock11111111111111111111111111111111";
  }
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.base),
      getAccountMeta(accounts.clockSysvar),
      getAccountMeta(accounts.lockupAuthority)
    ].filter((x) => x !== void 0),
    programAddress,
    data: getAuthorizeWithSeedInstructionDataEncoder().encode(
      args
    )
  };
  return instruction;
}
function parseAuthorizeWithSeedInstruction(instruction) {
  if (instruction.accounts.length < 3) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  let optionalAccountsRemaining = instruction.accounts.length - 3;
  const getNextOptionalAccount = () => {
    if (optionalAccountsRemaining === 0) return void 0;
    optionalAccountsRemaining -= 1;
    return getNextAccount();
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stake: getNextAccount(),
      base: getNextAccount(),
      clockSysvar: getNextAccount(),
      lockupAuthority: getNextOptionalAccount()
    },
    data: getAuthorizeWithSeedInstructionDataDecoder().decode(instruction.data)
  };
}
var DEACTIVATE_DISCRIMINATOR = 5;
function getDeactivateDiscriminatorBytes() {
  return kit.getU32Encoder().encode(DEACTIVATE_DISCRIMINATOR);
}
function getDeactivateInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([["discriminator", kit.getU32Encoder()]]),
    (value) => ({ ...value, discriminator: DEACTIVATE_DISCRIMINATOR })
  );
}
function getDeactivateInstructionDataDecoder() {
  return kit.getStructDecoder([["discriminator", kit.getU32Decoder()]]);
}
function getDeactivateInstructionDataCodec() {
  return kit.combineCodec(
    getDeactivateInstructionDataEncoder(),
    getDeactivateInstructionDataDecoder()
  );
}
function getDeactivateInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    stake: { value: input.stake ?? null, isWritable: true },
    clockSysvar: { value: input.clockSysvar ?? null, isWritable: false },
    stakeAuthority: { value: input.stakeAuthority ?? null, isWritable: false }
  };
  const accounts = originalAccounts;
  if (!accounts.clockSysvar.value) {
    accounts.clockSysvar.value = "SysvarC1ock11111111111111111111111111111111";
  }
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.clockSysvar),
      getAccountMeta(accounts.stakeAuthority)
    ],
    programAddress,
    data: getDeactivateInstructionDataEncoder().encode({})
  };
  return instruction;
}
function parseDeactivateInstruction(instruction) {
  if (instruction.accounts.length < 3) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stake: getNextAccount(),
      clockSysvar: getNextAccount(),
      stakeAuthority: getNextAccount()
    },
    data: getDeactivateInstructionDataDecoder().decode(instruction.data)
  };
}
var DEACTIVATE_DELINQUENT_DISCRIMINATOR = 14;
function getDeactivateDelinquentDiscriminatorBytes() {
  return kit.getU32Encoder().encode(DEACTIVATE_DELINQUENT_DISCRIMINATOR);
}
function getDeactivateDelinquentInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([["discriminator", kit.getU32Encoder()]]),
    (value) => ({
      ...value,
      discriminator: DEACTIVATE_DELINQUENT_DISCRIMINATOR
    })
  );
}
function getDeactivateDelinquentInstructionDataDecoder() {
  return kit.getStructDecoder([["discriminator", kit.getU32Decoder()]]);
}
function getDeactivateDelinquentInstructionDataCodec() {
  return kit.combineCodec(
    getDeactivateDelinquentInstructionDataEncoder(),
    getDeactivateDelinquentInstructionDataDecoder()
  );
}
function getDeactivateDelinquentInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    stake: { value: input.stake ?? null, isWritable: true },
    delinquentVote: { value: input.delinquentVote ?? null, isWritable: false },
    referenceVote: { value: input.referenceVote ?? null, isWritable: false }
  };
  const accounts = originalAccounts;
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.delinquentVote),
      getAccountMeta(accounts.referenceVote)
    ],
    programAddress,
    data: getDeactivateDelinquentInstructionDataEncoder().encode({})
  };
  return instruction;
}
function parseDeactivateDelinquentInstruction(instruction) {
  if (instruction.accounts.length < 3) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stake: getNextAccount(),
      delinquentVote: getNextAccount(),
      referenceVote: getNextAccount()
    },
    data: getDeactivateDelinquentInstructionDataDecoder().decode(
      instruction.data
    )
  };
}
var DELEGATE_STAKE_DISCRIMINATOR = 2;
function getDelegateStakeDiscriminatorBytes() {
  return kit.getU32Encoder().encode(DELEGATE_STAKE_DISCRIMINATOR);
}
function getDelegateStakeInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([["discriminator", kit.getU32Encoder()]]),
    (value) => ({ ...value, discriminator: DELEGATE_STAKE_DISCRIMINATOR })
  );
}
function getDelegateStakeInstructionDataDecoder() {
  return kit.getStructDecoder([["discriminator", kit.getU32Decoder()]]);
}
function getDelegateStakeInstructionDataCodec() {
  return kit.combineCodec(
    getDelegateStakeInstructionDataEncoder(),
    getDelegateStakeInstructionDataDecoder()
  );
}
function getDelegateStakeInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    stake: { value: input.stake ?? null, isWritable: true },
    vote: { value: input.vote ?? null, isWritable: false },
    clockSysvar: { value: input.clockSysvar ?? null, isWritable: false },
    stakeHistory: { value: input.stakeHistory ?? null, isWritable: false },
    unused: { value: input.unused ?? null, isWritable: false },
    stakeAuthority: { value: input.stakeAuthority ?? null, isWritable: false }
  };
  const accounts = originalAccounts;
  if (!accounts.clockSysvar.value) {
    accounts.clockSysvar.value = "SysvarC1ock11111111111111111111111111111111";
  }
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.vote),
      getAccountMeta(accounts.clockSysvar),
      getAccountMeta(accounts.stakeHistory),
      getAccountMeta(accounts.unused),
      getAccountMeta(accounts.stakeAuthority)
    ],
    programAddress,
    data: getDelegateStakeInstructionDataEncoder().encode({})
  };
  return instruction;
}
function parseDelegateStakeInstruction(instruction) {
  if (instruction.accounts.length < 6) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stake: getNextAccount(),
      vote: getNextAccount(),
      clockSysvar: getNextAccount(),
      stakeHistory: getNextAccount(),
      unused: getNextAccount(),
      stakeAuthority: getNextAccount()
    },
    data: getDelegateStakeInstructionDataDecoder().decode(instruction.data)
  };
}
var GET_MINIMUM_DELEGATION_DISCRIMINATOR = 13;
function getGetMinimumDelegationDiscriminatorBytes() {
  return kit.getU32Encoder().encode(GET_MINIMUM_DELEGATION_DISCRIMINATOR);
}
function getGetMinimumDelegationInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([["discriminator", kit.getU32Encoder()]]),
    (value) => ({
      ...value,
      discriminator: GET_MINIMUM_DELEGATION_DISCRIMINATOR
    })
  );
}
function getGetMinimumDelegationInstructionDataDecoder() {
  return kit.getStructDecoder([["discriminator", kit.getU32Decoder()]]);
}
function getGetMinimumDelegationInstructionDataCodec() {
  return kit.combineCodec(
    getGetMinimumDelegationInstructionDataEncoder(),
    getGetMinimumDelegationInstructionDataDecoder()
  );
}
function getGetMinimumDelegationInstruction(config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const instruction = {
    programAddress,
    data: getGetMinimumDelegationInstructionDataEncoder().encode({})
  };
  return instruction;
}
function parseGetMinimumDelegationInstruction(instruction) {
  return {
    programAddress: instruction.programAddress,
    data: getGetMinimumDelegationInstructionDataDecoder().decode(
      instruction.data
    )
  };
}
var INITIALIZE_DISCRIMINATOR = 0;
function getInitializeDiscriminatorBytes() {
  return kit.getU32Encoder().encode(INITIALIZE_DISCRIMINATOR);
}
function getInitializeInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([
      ["discriminator", kit.getU32Encoder()],
      ["arg0", getAuthorizedEncoder()],
      ["arg1", getLockupEncoder()]
    ]),
    (value) => ({ ...value, discriminator: INITIALIZE_DISCRIMINATOR })
  );
}
function getInitializeInstructionDataDecoder() {
  return kit.getStructDecoder([
    ["discriminator", kit.getU32Decoder()],
    ["arg0", getAuthorizedDecoder()],
    ["arg1", getLockupDecoder()]
  ]);
}
function getInitializeInstructionDataCodec() {
  return kit.combineCodec(
    getInitializeInstructionDataEncoder(),
    getInitializeInstructionDataDecoder()
  );
}
function getInitializeInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    stake: { value: input.stake ?? null, isWritable: true },
    rentSysvar: { value: input.rentSysvar ?? null, isWritable: false }
  };
  const accounts = originalAccounts;
  const args = { ...input };
  if (!accounts.rentSysvar.value) {
    accounts.rentSysvar.value = "SysvarRent111111111111111111111111111111111";
  }
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.rentSysvar)
    ],
    programAddress,
    data: getInitializeInstructionDataEncoder().encode(
      args
    )
  };
  return instruction;
}
function parseInitializeInstruction(instruction) {
  if (instruction.accounts.length < 2) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stake: getNextAccount(),
      rentSysvar: getNextAccount()
    },
    data: getInitializeInstructionDataDecoder().decode(instruction.data)
  };
}
var INITIALIZE_CHECKED_DISCRIMINATOR = 9;
function getInitializeCheckedDiscriminatorBytes() {
  return kit.getU32Encoder().encode(INITIALIZE_CHECKED_DISCRIMINATOR);
}
function getInitializeCheckedInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([["discriminator", kit.getU32Encoder()]]),
    (value) => ({ ...value, discriminator: INITIALIZE_CHECKED_DISCRIMINATOR })
  );
}
function getInitializeCheckedInstructionDataDecoder() {
  return kit.getStructDecoder([["discriminator", kit.getU32Decoder()]]);
}
function getInitializeCheckedInstructionDataCodec() {
  return kit.combineCodec(
    getInitializeCheckedInstructionDataEncoder(),
    getInitializeCheckedInstructionDataDecoder()
  );
}
function getInitializeCheckedInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    stake: { value: input.stake ?? null, isWritable: true },
    rentSysvar: { value: input.rentSysvar ?? null, isWritable: false },
    stakeAuthority: { value: input.stakeAuthority ?? null, isWritable: false },
    withdrawAuthority: {
      value: input.withdrawAuthority ?? null,
      isWritable: false
    }
  };
  const accounts = originalAccounts;
  if (!accounts.rentSysvar.value) {
    accounts.rentSysvar.value = "SysvarRent111111111111111111111111111111111";
  }
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.rentSysvar),
      getAccountMeta(accounts.stakeAuthority),
      getAccountMeta(accounts.withdrawAuthority)
    ],
    programAddress,
    data: getInitializeCheckedInstructionDataEncoder().encode({})
  };
  return instruction;
}
function parseInitializeCheckedInstruction(instruction) {
  if (instruction.accounts.length < 4) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stake: getNextAccount(),
      rentSysvar: getNextAccount(),
      stakeAuthority: getNextAccount(),
      withdrawAuthority: getNextAccount()
    },
    data: getInitializeCheckedInstructionDataDecoder().decode(instruction.data)
  };
}
var MERGE_DISCRIMINATOR = 7;
function getMergeDiscriminatorBytes() {
  return kit.getU32Encoder().encode(MERGE_DISCRIMINATOR);
}
function getMergeInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([["discriminator", kit.getU32Encoder()]]),
    (value) => ({ ...value, discriminator: MERGE_DISCRIMINATOR })
  );
}
function getMergeInstructionDataDecoder() {
  return kit.getStructDecoder([["discriminator", kit.getU32Decoder()]]);
}
function getMergeInstructionDataCodec() {
  return kit.combineCodec(
    getMergeInstructionDataEncoder(),
    getMergeInstructionDataDecoder()
  );
}
function getMergeInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    destinationStake: {
      value: input.destinationStake ?? null,
      isWritable: true
    },
    sourceStake: { value: input.sourceStake ?? null, isWritable: true },
    clockSysvar: { value: input.clockSysvar ?? null, isWritable: false },
    stakeHistory: { value: input.stakeHistory ?? null, isWritable: false },
    stakeAuthority: { value: input.stakeAuthority ?? null, isWritable: false }
  };
  const accounts = originalAccounts;
  if (!accounts.clockSysvar.value) {
    accounts.clockSysvar.value = "SysvarC1ock11111111111111111111111111111111";
  }
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.destinationStake),
      getAccountMeta(accounts.sourceStake),
      getAccountMeta(accounts.clockSysvar),
      getAccountMeta(accounts.stakeHistory),
      getAccountMeta(accounts.stakeAuthority)
    ],
    programAddress,
    data: getMergeInstructionDataEncoder().encode({})
  };
  return instruction;
}
function parseMergeInstruction(instruction) {
  if (instruction.accounts.length < 5) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      destinationStake: getNextAccount(),
      sourceStake: getNextAccount(),
      clockSysvar: getNextAccount(),
      stakeHistory: getNextAccount(),
      stakeAuthority: getNextAccount()
    },
    data: getMergeInstructionDataDecoder().decode(instruction.data)
  };
}
var MOVE_LAMPORTS_DISCRIMINATOR = 17;
function getMoveLamportsDiscriminatorBytes() {
  return kit.getU32Encoder().encode(MOVE_LAMPORTS_DISCRIMINATOR);
}
function getMoveLamportsInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([
      ["discriminator", kit.getU32Encoder()],
      ["args", kit.getU64Encoder()]
    ]),
    (value) => ({ ...value, discriminator: MOVE_LAMPORTS_DISCRIMINATOR })
  );
}
function getMoveLamportsInstructionDataDecoder() {
  return kit.getStructDecoder([
    ["discriminator", kit.getU32Decoder()],
    ["args", kit.getU64Decoder()]
  ]);
}
function getMoveLamportsInstructionDataCodec() {
  return kit.combineCodec(
    getMoveLamportsInstructionDataEncoder(),
    getMoveLamportsInstructionDataDecoder()
  );
}
function getMoveLamportsInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    sourceStake: { value: input.sourceStake ?? null, isWritable: true },
    destinationStake: {
      value: input.destinationStake ?? null,
      isWritable: true
    },
    stakeAuthority: { value: input.stakeAuthority ?? null, isWritable: false }
  };
  const accounts = originalAccounts;
  const args = { ...input };
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.sourceStake),
      getAccountMeta(accounts.destinationStake),
      getAccountMeta(accounts.stakeAuthority)
    ],
    programAddress,
    data: getMoveLamportsInstructionDataEncoder().encode(
      args
    )
  };
  return instruction;
}
function parseMoveLamportsInstruction(instruction) {
  if (instruction.accounts.length < 3) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      sourceStake: getNextAccount(),
      destinationStake: getNextAccount(),
      stakeAuthority: getNextAccount()
    },
    data: getMoveLamportsInstructionDataDecoder().decode(instruction.data)
  };
}
var MOVE_STAKE_DISCRIMINATOR = 16;
function getMoveStakeDiscriminatorBytes() {
  return kit.getU32Encoder().encode(MOVE_STAKE_DISCRIMINATOR);
}
function getMoveStakeInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([
      ["discriminator", kit.getU32Encoder()],
      ["args", kit.getU64Encoder()]
    ]),
    (value) => ({ ...value, discriminator: MOVE_STAKE_DISCRIMINATOR })
  );
}
function getMoveStakeInstructionDataDecoder() {
  return kit.getStructDecoder([
    ["discriminator", kit.getU32Decoder()],
    ["args", kit.getU64Decoder()]
  ]);
}
function getMoveStakeInstructionDataCodec() {
  return kit.combineCodec(
    getMoveStakeInstructionDataEncoder(),
    getMoveStakeInstructionDataDecoder()
  );
}
function getMoveStakeInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    sourceStake: { value: input.sourceStake ?? null, isWritable: true },
    destinationStake: {
      value: input.destinationStake ?? null,
      isWritable: true
    },
    stakeAuthority: { value: input.stakeAuthority ?? null, isWritable: false }
  };
  const accounts = originalAccounts;
  const args = { ...input };
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.sourceStake),
      getAccountMeta(accounts.destinationStake),
      getAccountMeta(accounts.stakeAuthority)
    ],
    programAddress,
    data: getMoveStakeInstructionDataEncoder().encode(
      args
    )
  };
  return instruction;
}
function parseMoveStakeInstruction(instruction) {
  if (instruction.accounts.length < 3) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      sourceStake: getNextAccount(),
      destinationStake: getNextAccount(),
      stakeAuthority: getNextAccount()
    },
    data: getMoveStakeInstructionDataDecoder().decode(instruction.data)
  };
}
var SET_LOCKUP_DISCRIMINATOR = 6;
function getSetLockupDiscriminatorBytes() {
  return kit.getU32Encoder().encode(SET_LOCKUP_DISCRIMINATOR);
}
function getSetLockupInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([
      ["discriminator", kit.getU32Encoder()],
      ["unixTimestamp", kit.getOptionEncoder(kit.getI64Encoder())],
      ["epoch", kit.getOptionEncoder(kit.getU64Encoder())],
      ["custodian", kit.getOptionEncoder(kit.getAddressEncoder())]
    ]),
    (value) => ({ ...value, discriminator: SET_LOCKUP_DISCRIMINATOR })
  );
}
function getSetLockupInstructionDataDecoder() {
  return kit.getStructDecoder([
    ["discriminator", kit.getU32Decoder()],
    ["unixTimestamp", kit.getOptionDecoder(kit.getI64Decoder())],
    ["epoch", kit.getOptionDecoder(kit.getU64Decoder())],
    ["custodian", kit.getOptionDecoder(kit.getAddressDecoder())]
  ]);
}
function getSetLockupInstructionDataCodec() {
  return kit.combineCodec(
    getSetLockupInstructionDataEncoder(),
    getSetLockupInstructionDataDecoder()
  );
}
function getSetLockupInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    stake: { value: input.stake ?? null, isWritable: true },
    authority: { value: input.authority ?? null, isWritable: false }
  };
  const accounts = originalAccounts;
  const args = { ...input };
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.authority)
    ],
    programAddress,
    data: getSetLockupInstructionDataEncoder().encode(
      args
    )
  };
  return instruction;
}
function parseSetLockupInstruction(instruction) {
  if (instruction.accounts.length < 2) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stake: getNextAccount(),
      authority: getNextAccount()
    },
    data: getSetLockupInstructionDataDecoder().decode(instruction.data)
  };
}
var SET_LOCKUP_CHECKED_DISCRIMINATOR = 12;
function getSetLockupCheckedDiscriminatorBytes() {
  return kit.getU32Encoder().encode(SET_LOCKUP_CHECKED_DISCRIMINATOR);
}
function getSetLockupCheckedInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([
      ["discriminator", kit.getU32Encoder()],
      ["unixTimestamp", kit.getOptionEncoder(kit.getI64Encoder())],
      ["epoch", kit.getOptionEncoder(kit.getU64Encoder())]
    ]),
    (value) => ({ ...value, discriminator: SET_LOCKUP_CHECKED_DISCRIMINATOR })
  );
}
function getSetLockupCheckedInstructionDataDecoder() {
  return kit.getStructDecoder([
    ["discriminator", kit.getU32Decoder()],
    ["unixTimestamp", kit.getOptionDecoder(kit.getI64Decoder())],
    ["epoch", kit.getOptionDecoder(kit.getU64Decoder())]
  ]);
}
function getSetLockupCheckedInstructionDataCodec() {
  return kit.combineCodec(
    getSetLockupCheckedInstructionDataEncoder(),
    getSetLockupCheckedInstructionDataDecoder()
  );
}
function getSetLockupCheckedInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    stake: { value: input.stake ?? null, isWritable: true },
    authority: { value: input.authority ?? null, isWritable: false },
    newAuthority: { value: input.newAuthority ?? null, isWritable: false }
  };
  const accounts = originalAccounts;
  const args = { ...input };
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.authority),
      getAccountMeta(accounts.newAuthority)
    ].filter((x) => x !== void 0),
    programAddress,
    data: getSetLockupCheckedInstructionDataEncoder().encode(
      args
    )
  };
  return instruction;
}
function parseSetLockupCheckedInstruction(instruction) {
  if (instruction.accounts.length < 2) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  let optionalAccountsRemaining = instruction.accounts.length - 2;
  const getNextOptionalAccount = () => {
    if (optionalAccountsRemaining === 0) return void 0;
    optionalAccountsRemaining -= 1;
    return getNextAccount();
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stake: getNextAccount(),
      authority: getNextAccount(),
      newAuthority: getNextOptionalAccount()
    },
    data: getSetLockupCheckedInstructionDataDecoder().decode(instruction.data)
  };
}
var SPLIT_DISCRIMINATOR = 3;
function getSplitDiscriminatorBytes() {
  return kit.getU32Encoder().encode(SPLIT_DISCRIMINATOR);
}
function getSplitInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([
      ["discriminator", kit.getU32Encoder()],
      ["args", kit.getU64Encoder()]
    ]),
    (value) => ({ ...value, discriminator: SPLIT_DISCRIMINATOR })
  );
}
function getSplitInstructionDataDecoder() {
  return kit.getStructDecoder([
    ["discriminator", kit.getU32Decoder()],
    ["args", kit.getU64Decoder()]
  ]);
}
function getSplitInstructionDataCodec() {
  return kit.combineCodec(
    getSplitInstructionDataEncoder(),
    getSplitInstructionDataDecoder()
  );
}
function getSplitInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    stake: { value: input.stake ?? null, isWritable: true },
    splitStake: { value: input.splitStake ?? null, isWritable: true },
    stakeAuthority: { value: input.stakeAuthority ?? null, isWritable: false }
  };
  const accounts = originalAccounts;
  const args = { ...input };
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.splitStake),
      getAccountMeta(accounts.stakeAuthority)
    ],
    programAddress,
    data: getSplitInstructionDataEncoder().encode(
      args
    )
  };
  return instruction;
}
function parseSplitInstruction(instruction) {
  if (instruction.accounts.length < 3) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stake: getNextAccount(),
      splitStake: getNextAccount(),
      stakeAuthority: getNextAccount()
    },
    data: getSplitInstructionDataDecoder().decode(instruction.data)
  };
}
var WITHDRAW_DISCRIMINATOR = 4;
function getWithdrawDiscriminatorBytes() {
  return kit.getU32Encoder().encode(WITHDRAW_DISCRIMINATOR);
}
function getWithdrawInstructionDataEncoder() {
  return kit.transformEncoder(
    kit.getStructEncoder([
      ["discriminator", kit.getU32Encoder()],
      ["args", kit.getU64Encoder()]
    ]),
    (value) => ({ ...value, discriminator: WITHDRAW_DISCRIMINATOR })
  );
}
function getWithdrawInstructionDataDecoder() {
  return kit.getStructDecoder([
    ["discriminator", kit.getU32Decoder()],
    ["args", kit.getU64Decoder()]
  ]);
}
function getWithdrawInstructionDataCodec() {
  return kit.combineCodec(
    getWithdrawInstructionDataEncoder(),
    getWithdrawInstructionDataDecoder()
  );
}
function getWithdrawInstruction(input, config) {
  const programAddress = config?.programAddress ?? STAKE_PROGRAM_ADDRESS;
  const originalAccounts = {
    stake: { value: input.stake ?? null, isWritable: true },
    recipient: { value: input.recipient ?? null, isWritable: true },
    clockSysvar: { value: input.clockSysvar ?? null, isWritable: false },
    stakeHistory: { value: input.stakeHistory ?? null, isWritable: false },
    withdrawAuthority: {
      value: input.withdrawAuthority ?? null,
      isWritable: false
    },
    lockupAuthority: {
      value: input.lockupAuthority ?? null,
      isWritable: false
    }
  };
  const accounts = originalAccounts;
  const args = { ...input };
  if (!accounts.clockSysvar.value) {
    accounts.clockSysvar.value = "SysvarC1ock11111111111111111111111111111111";
  }
  const getAccountMeta = getAccountMetaFactory();
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.recipient),
      getAccountMeta(accounts.clockSysvar),
      getAccountMeta(accounts.stakeHistory),
      getAccountMeta(accounts.withdrawAuthority),
      getAccountMeta(accounts.lockupAuthority)
    ].filter((x) => x !== void 0),
    programAddress,
    data: getWithdrawInstructionDataEncoder().encode(
      args
    )
  };
  return instruction;
}
function parseWithdrawInstruction(instruction) {
  if (instruction.accounts.length < 5) {
    throw new Error("Not enough accounts");
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts[accountIndex];
    accountIndex += 1;
    return accountMeta;
  };
  let optionalAccountsRemaining = instruction.accounts.length - 5;
  const getNextOptionalAccount = () => {
    if (optionalAccountsRemaining === 0) return void 0;
    optionalAccountsRemaining -= 1;
    return getNextAccount();
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stake: getNextAccount(),
      recipient: getNextAccount(),
      clockSysvar: getNextAccount(),
      stakeHistory: getNextAccount(),
      withdrawAuthority: getNextAccount(),
      lockupAuthority: getNextOptionalAccount()
    },
    data: getWithdrawInstructionDataDecoder().decode(instruction.data)
  };
}

exports.AUTHORIZE_CHECKED_DISCRIMINATOR = AUTHORIZE_CHECKED_DISCRIMINATOR;
exports.AUTHORIZE_CHECKED_WITH_SEED_DISCRIMINATOR = AUTHORIZE_CHECKED_WITH_SEED_DISCRIMINATOR;
exports.AUTHORIZE_DISCRIMINATOR = AUTHORIZE_DISCRIMINATOR;
exports.AUTHORIZE_WITH_SEED_DISCRIMINATOR = AUTHORIZE_WITH_SEED_DISCRIMINATOR;
exports.DEACTIVATE_DELINQUENT_DISCRIMINATOR = DEACTIVATE_DELINQUENT_DISCRIMINATOR;
exports.DEACTIVATE_DISCRIMINATOR = DEACTIVATE_DISCRIMINATOR;
exports.DELEGATE_STAKE_DISCRIMINATOR = DELEGATE_STAKE_DISCRIMINATOR;
exports.GET_MINIMUM_DELEGATION_DISCRIMINATOR = GET_MINIMUM_DELEGATION_DISCRIMINATOR;
exports.INITIALIZE_CHECKED_DISCRIMINATOR = INITIALIZE_CHECKED_DISCRIMINATOR;
exports.INITIALIZE_DISCRIMINATOR = INITIALIZE_DISCRIMINATOR;
exports.MERGE_DISCRIMINATOR = MERGE_DISCRIMINATOR;
exports.MOVE_LAMPORTS_DISCRIMINATOR = MOVE_LAMPORTS_DISCRIMINATOR;
exports.MOVE_STAKE_DISCRIMINATOR = MOVE_STAKE_DISCRIMINATOR;
exports.SET_LOCKUP_CHECKED_DISCRIMINATOR = SET_LOCKUP_CHECKED_DISCRIMINATOR;
exports.SET_LOCKUP_DISCRIMINATOR = SET_LOCKUP_DISCRIMINATOR;
exports.SPLIT_DISCRIMINATOR = SPLIT_DISCRIMINATOR;
exports.STAKE_ERROR__ALREADY_DEACTIVATED = STAKE_ERROR__ALREADY_DEACTIVATED;
exports.STAKE_ERROR__CUSTODIAN_MISSING = STAKE_ERROR__CUSTODIAN_MISSING;
exports.STAKE_ERROR__CUSTODIAN_SIGNATURE_MISSING = STAKE_ERROR__CUSTODIAN_SIGNATURE_MISSING;
exports.STAKE_ERROR__EPOCH_REWARDS_ACTIVE = STAKE_ERROR__EPOCH_REWARDS_ACTIVE;
exports.STAKE_ERROR__INSUFFICIENT_DELEGATION = STAKE_ERROR__INSUFFICIENT_DELEGATION;
exports.STAKE_ERROR__INSUFFICIENT_REFERENCE_VOTES = STAKE_ERROR__INSUFFICIENT_REFERENCE_VOTES;
exports.STAKE_ERROR__INSUFFICIENT_STAKE = STAKE_ERROR__INSUFFICIENT_STAKE;
exports.STAKE_ERROR__LOCKUP_IN_FORCE = STAKE_ERROR__LOCKUP_IN_FORCE;
exports.STAKE_ERROR__MERGE_MISMATCH = STAKE_ERROR__MERGE_MISMATCH;
exports.STAKE_ERROR__MERGE_TRANSIENT_STAKE = STAKE_ERROR__MERGE_TRANSIENT_STAKE;
exports.STAKE_ERROR__MINIMUM_DELINQUENT_EPOCHS_FOR_DEACTIVATION_NOT_MET = STAKE_ERROR__MINIMUM_DELINQUENT_EPOCHS_FOR_DEACTIVATION_NOT_MET;
exports.STAKE_ERROR__NO_CREDITS_TO_REDEEM = STAKE_ERROR__NO_CREDITS_TO_REDEEM;
exports.STAKE_ERROR__REDELEGATED_STAKE_MUST_FULLY_ACTIVATE_BEFORE_DEACTIVATION_IS_PERMITTED = STAKE_ERROR__REDELEGATED_STAKE_MUST_FULLY_ACTIVATE_BEFORE_DEACTIVATION_IS_PERMITTED;
exports.STAKE_ERROR__REDELEGATE_TO_SAME_VOTE_ACCOUNT = STAKE_ERROR__REDELEGATE_TO_SAME_VOTE_ACCOUNT;
exports.STAKE_ERROR__REDELEGATE_TRANSIENT_OR_INACTIVE_STAKE = STAKE_ERROR__REDELEGATE_TRANSIENT_OR_INACTIVE_STAKE;
exports.STAKE_ERROR__TOO_SOON_TO_REDELEGATE = STAKE_ERROR__TOO_SOON_TO_REDELEGATE;
exports.STAKE_ERROR__VOTE_ADDRESS_MISMATCH = STAKE_ERROR__VOTE_ADDRESS_MISMATCH;
exports.STAKE_PROGRAM_ADDRESS = STAKE_PROGRAM_ADDRESS;
exports.StakeAccount = StakeAccount;
exports.StakeAuthorize = StakeAuthorize;
exports.StakeInstruction = StakeInstruction;
exports.WITHDRAW_DISCRIMINATOR = WITHDRAW_DISCRIMINATOR;
exports.decodeStakeStateAccount = decodeStakeStateAccount;
exports.fetchAllMaybeStakeStateAccount = fetchAllMaybeStakeStateAccount;
exports.fetchAllStakeStateAccount = fetchAllStakeStateAccount;
exports.fetchMaybeStakeStateAccount = fetchMaybeStakeStateAccount;
exports.fetchStakeStateAccount = fetchStakeStateAccount;
exports.getAuthorizeCheckedDiscriminatorBytes = getAuthorizeCheckedDiscriminatorBytes;
exports.getAuthorizeCheckedInstruction = getAuthorizeCheckedInstruction;
exports.getAuthorizeCheckedInstructionDataCodec = getAuthorizeCheckedInstructionDataCodec;
exports.getAuthorizeCheckedInstructionDataDecoder = getAuthorizeCheckedInstructionDataDecoder;
exports.getAuthorizeCheckedInstructionDataEncoder = getAuthorizeCheckedInstructionDataEncoder;
exports.getAuthorizeCheckedWithSeedDiscriminatorBytes = getAuthorizeCheckedWithSeedDiscriminatorBytes;
exports.getAuthorizeCheckedWithSeedInstruction = getAuthorizeCheckedWithSeedInstruction;
exports.getAuthorizeCheckedWithSeedInstructionDataCodec = getAuthorizeCheckedWithSeedInstructionDataCodec;
exports.getAuthorizeCheckedWithSeedInstructionDataDecoder = getAuthorizeCheckedWithSeedInstructionDataDecoder;
exports.getAuthorizeCheckedWithSeedInstructionDataEncoder = getAuthorizeCheckedWithSeedInstructionDataEncoder;
exports.getAuthorizeDiscriminatorBytes = getAuthorizeDiscriminatorBytes;
exports.getAuthorizeInstruction = getAuthorizeInstruction;
exports.getAuthorizeInstructionDataCodec = getAuthorizeInstructionDataCodec;
exports.getAuthorizeInstructionDataDecoder = getAuthorizeInstructionDataDecoder;
exports.getAuthorizeInstructionDataEncoder = getAuthorizeInstructionDataEncoder;
exports.getAuthorizeWithSeedDiscriminatorBytes = getAuthorizeWithSeedDiscriminatorBytes;
exports.getAuthorizeWithSeedInstruction = getAuthorizeWithSeedInstruction;
exports.getAuthorizeWithSeedInstructionDataCodec = getAuthorizeWithSeedInstructionDataCodec;
exports.getAuthorizeWithSeedInstructionDataDecoder = getAuthorizeWithSeedInstructionDataDecoder;
exports.getAuthorizeWithSeedInstructionDataEncoder = getAuthorizeWithSeedInstructionDataEncoder;
exports.getAuthorizedCodec = getAuthorizedCodec;
exports.getAuthorizedDecoder = getAuthorizedDecoder;
exports.getAuthorizedEncoder = getAuthorizedEncoder;
exports.getDeactivateDelinquentDiscriminatorBytes = getDeactivateDelinquentDiscriminatorBytes;
exports.getDeactivateDelinquentInstruction = getDeactivateDelinquentInstruction;
exports.getDeactivateDelinquentInstructionDataCodec = getDeactivateDelinquentInstructionDataCodec;
exports.getDeactivateDelinquentInstructionDataDecoder = getDeactivateDelinquentInstructionDataDecoder;
exports.getDeactivateDelinquentInstructionDataEncoder = getDeactivateDelinquentInstructionDataEncoder;
exports.getDeactivateDiscriminatorBytes = getDeactivateDiscriminatorBytes;
exports.getDeactivateInstruction = getDeactivateInstruction;
exports.getDeactivateInstructionDataCodec = getDeactivateInstructionDataCodec;
exports.getDeactivateInstructionDataDecoder = getDeactivateInstructionDataDecoder;
exports.getDeactivateInstructionDataEncoder = getDeactivateInstructionDataEncoder;
exports.getDelegateStakeDiscriminatorBytes = getDelegateStakeDiscriminatorBytes;
exports.getDelegateStakeInstruction = getDelegateStakeInstruction;
exports.getDelegateStakeInstructionDataCodec = getDelegateStakeInstructionDataCodec;
exports.getDelegateStakeInstructionDataDecoder = getDelegateStakeInstructionDataDecoder;
exports.getDelegateStakeInstructionDataEncoder = getDelegateStakeInstructionDataEncoder;
exports.getDelegationCodec = getDelegationCodec;
exports.getDelegationDecoder = getDelegationDecoder;
exports.getDelegationEncoder = getDelegationEncoder;
exports.getGetMinimumDelegationDiscriminatorBytes = getGetMinimumDelegationDiscriminatorBytes;
exports.getGetMinimumDelegationInstruction = getGetMinimumDelegationInstruction;
exports.getGetMinimumDelegationInstructionDataCodec = getGetMinimumDelegationInstructionDataCodec;
exports.getGetMinimumDelegationInstructionDataDecoder = getGetMinimumDelegationInstructionDataDecoder;
exports.getGetMinimumDelegationInstructionDataEncoder = getGetMinimumDelegationInstructionDataEncoder;
exports.getInitializeCheckedDiscriminatorBytes = getInitializeCheckedDiscriminatorBytes;
exports.getInitializeCheckedInstruction = getInitializeCheckedInstruction;
exports.getInitializeCheckedInstructionDataCodec = getInitializeCheckedInstructionDataCodec;
exports.getInitializeCheckedInstructionDataDecoder = getInitializeCheckedInstructionDataDecoder;
exports.getInitializeCheckedInstructionDataEncoder = getInitializeCheckedInstructionDataEncoder;
exports.getInitializeDiscriminatorBytes = getInitializeDiscriminatorBytes;
exports.getInitializeInstruction = getInitializeInstruction;
exports.getInitializeInstructionDataCodec = getInitializeInstructionDataCodec;
exports.getInitializeInstructionDataDecoder = getInitializeInstructionDataDecoder;
exports.getInitializeInstructionDataEncoder = getInitializeInstructionDataEncoder;
exports.getLockupCodec = getLockupCodec;
exports.getLockupDecoder = getLockupDecoder;
exports.getLockupEncoder = getLockupEncoder;
exports.getMergeDiscriminatorBytes = getMergeDiscriminatorBytes;
exports.getMergeInstruction = getMergeInstruction;
exports.getMergeInstructionDataCodec = getMergeInstructionDataCodec;
exports.getMergeInstructionDataDecoder = getMergeInstructionDataDecoder;
exports.getMergeInstructionDataEncoder = getMergeInstructionDataEncoder;
exports.getMetaCodec = getMetaCodec;
exports.getMetaDecoder = getMetaDecoder;
exports.getMetaEncoder = getMetaEncoder;
exports.getMoveLamportsDiscriminatorBytes = getMoveLamportsDiscriminatorBytes;
exports.getMoveLamportsInstruction = getMoveLamportsInstruction;
exports.getMoveLamportsInstructionDataCodec = getMoveLamportsInstructionDataCodec;
exports.getMoveLamportsInstructionDataDecoder = getMoveLamportsInstructionDataDecoder;
exports.getMoveLamportsInstructionDataEncoder = getMoveLamportsInstructionDataEncoder;
exports.getMoveStakeDiscriminatorBytes = getMoveStakeDiscriminatorBytes;
exports.getMoveStakeInstruction = getMoveStakeInstruction;
exports.getMoveStakeInstructionDataCodec = getMoveStakeInstructionDataCodec;
exports.getMoveStakeInstructionDataDecoder = getMoveStakeInstructionDataDecoder;
exports.getMoveStakeInstructionDataEncoder = getMoveStakeInstructionDataEncoder;
exports.getSetLockupCheckedDiscriminatorBytes = getSetLockupCheckedDiscriminatorBytes;
exports.getSetLockupCheckedInstruction = getSetLockupCheckedInstruction;
exports.getSetLockupCheckedInstructionDataCodec = getSetLockupCheckedInstructionDataCodec;
exports.getSetLockupCheckedInstructionDataDecoder = getSetLockupCheckedInstructionDataDecoder;
exports.getSetLockupCheckedInstructionDataEncoder = getSetLockupCheckedInstructionDataEncoder;
exports.getSetLockupDiscriminatorBytes = getSetLockupDiscriminatorBytes;
exports.getSetLockupInstruction = getSetLockupInstruction;
exports.getSetLockupInstructionDataCodec = getSetLockupInstructionDataCodec;
exports.getSetLockupInstructionDataDecoder = getSetLockupInstructionDataDecoder;
exports.getSetLockupInstructionDataEncoder = getSetLockupInstructionDataEncoder;
exports.getSplitDiscriminatorBytes = getSplitDiscriminatorBytes;
exports.getSplitInstruction = getSplitInstruction;
exports.getSplitInstructionDataCodec = getSplitInstructionDataCodec;
exports.getSplitInstructionDataDecoder = getSplitInstructionDataDecoder;
exports.getSplitInstructionDataEncoder = getSplitInstructionDataEncoder;
exports.getStakeAuthorizeCodec = getStakeAuthorizeCodec;
exports.getStakeAuthorizeDecoder = getStakeAuthorizeDecoder;
exports.getStakeAuthorizeEncoder = getStakeAuthorizeEncoder;
exports.getStakeCodec = getStakeCodec;
exports.getStakeDecoder = getStakeDecoder;
exports.getStakeEncoder = getStakeEncoder;
exports.getStakeErrorMessage = getStakeErrorMessage;
exports.getStakeFlagsCodec = getStakeFlagsCodec;
exports.getStakeFlagsDecoder = getStakeFlagsDecoder;
exports.getStakeFlagsEncoder = getStakeFlagsEncoder;
exports.getStakeStateAccountCodec = getStakeStateAccountCodec;
exports.getStakeStateAccountDecoder = getStakeStateAccountDecoder;
exports.getStakeStateAccountEncoder = getStakeStateAccountEncoder;
exports.getStakeStateCodec = getStakeStateCodec;
exports.getStakeStateDecoder = getStakeStateDecoder;
exports.getStakeStateEncoder = getStakeStateEncoder;
exports.getStakeStateV2Codec = getStakeStateV2Codec;
exports.getStakeStateV2Decoder = getStakeStateV2Decoder;
exports.getStakeStateV2Encoder = getStakeStateV2Encoder;
exports.getWithdrawDiscriminatorBytes = getWithdrawDiscriminatorBytes;
exports.getWithdrawInstruction = getWithdrawInstruction;
exports.getWithdrawInstructionDataCodec = getWithdrawInstructionDataCodec;
exports.getWithdrawInstructionDataDecoder = getWithdrawInstructionDataDecoder;
exports.getWithdrawInstructionDataEncoder = getWithdrawInstructionDataEncoder;
exports.identifyStakeInstruction = identifyStakeInstruction;
exports.isStakeError = isStakeError;
exports.isStakeState = isStakeState;
exports.isStakeStateV2 = isStakeStateV2;
exports.parseAuthorizeCheckedInstruction = parseAuthorizeCheckedInstruction;
exports.parseAuthorizeCheckedWithSeedInstruction = parseAuthorizeCheckedWithSeedInstruction;
exports.parseAuthorizeInstruction = parseAuthorizeInstruction;
exports.parseAuthorizeWithSeedInstruction = parseAuthorizeWithSeedInstruction;
exports.parseDeactivateDelinquentInstruction = parseDeactivateDelinquentInstruction;
exports.parseDeactivateInstruction = parseDeactivateInstruction;
exports.parseDelegateStakeInstruction = parseDelegateStakeInstruction;
exports.parseGetMinimumDelegationInstruction = parseGetMinimumDelegationInstruction;
exports.parseInitializeCheckedInstruction = parseInitializeCheckedInstruction;
exports.parseInitializeInstruction = parseInitializeInstruction;
exports.parseMergeInstruction = parseMergeInstruction;
exports.parseMoveLamportsInstruction = parseMoveLamportsInstruction;
exports.parseMoveStakeInstruction = parseMoveStakeInstruction;
exports.parseSetLockupCheckedInstruction = parseSetLockupCheckedInstruction;
exports.parseSetLockupInstruction = parseSetLockupInstruction;
exports.parseSplitInstruction = parseSplitInstruction;
exports.parseWithdrawInstruction = parseWithdrawInstruction;
exports.stakeState = stakeState;
exports.stakeStateV2 = stakeStateV2;
//# sourceMappingURL=index.js.map
//# sourceMappingURL=index.js.map