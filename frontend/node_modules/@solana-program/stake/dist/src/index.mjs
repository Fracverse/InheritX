import { getStructEncoder, getAddressEncoder, getStructDecoder, getAddressDecoder, combineCodec, getU64Encoder, getF64Encoder, getU64Decoder, getF64Decoder, getI64Encoder, getI64Decoder, getEnumEncoder, getEnumDecoder, getU8Encoder, getU8Decoder, getDiscriminatedUnionEncoder, getUnitEncoder, getTupleEncoder, getU32Encoder, getDiscriminatedUnionDecoder, getUnitDecoder, getTupleDecoder, getU32Decoder, decodeAccount, assertAccountExists, fetchEncodedAccount, assertAccountsExist, fetchEncodedAccounts, containsBytes, isProgramError, transformEncoder, addEncoderSizePrefix, getUtf8Encoder, addDecoderSizePrefix, getUtf8Decoder, getOptionEncoder, getOptionDecoder, AccountRole, upgradeRoleToSigner, isTransactionSigner as isTransactionSigner$1 } from '@solana/kit';

// src/generated/accounts/stakeStateAccount.ts
function getAuthorizedEncoder() {
  return getStructEncoder([
    ["staker", getAddressEncoder()],
    ["withdrawer", getAddressEncoder()]
  ]);
}
function getAuthorizedDecoder() {
  return getStructDecoder([
    ["staker", getAddressDecoder()],
    ["withdrawer", getAddressDecoder()]
  ]);
}
function getAuthorizedCodec() {
  return combineCodec(getAuthorizedEncoder(), getAuthorizedDecoder());
}
function getDelegationEncoder() {
  return getStructEncoder([
    ["voterPubkey", getAddressEncoder()],
    ["stake", getU64Encoder()],
    ["activationEpoch", getU64Encoder()],
    ["deactivationEpoch", getU64Encoder()],
    ["warmupCooldownRate", getF64Encoder()]
  ]);
}
function getDelegationDecoder() {
  return getStructDecoder([
    ["voterPubkey", getAddressDecoder()],
    ["stake", getU64Decoder()],
    ["activationEpoch", getU64Decoder()],
    ["deactivationEpoch", getU64Decoder()],
    ["warmupCooldownRate", getF64Decoder()]
  ]);
}
function getDelegationCodec() {
  return combineCodec(getDelegationEncoder(), getDelegationDecoder());
}
function getLockupEncoder() {
  return getStructEncoder([
    ["unixTimestamp", getI64Encoder()],
    ["epoch", getU64Encoder()],
    ["custodian", getAddressEncoder()]
  ]);
}
function getLockupDecoder() {
  return getStructDecoder([
    ["unixTimestamp", getI64Decoder()],
    ["epoch", getU64Decoder()],
    ["custodian", getAddressDecoder()]
  ]);
}
function getLockupCodec() {
  return combineCodec(getLockupEncoder(), getLockupDecoder());
}
function getMetaEncoder() {
  return getStructEncoder([
    ["rentExemptReserve", getU64Encoder()],
    ["authorized", getAuthorizedEncoder()],
    ["lockup", getLockupEncoder()]
  ]);
}
function getMetaDecoder() {
  return getStructDecoder([
    ["rentExemptReserve", getU64Decoder()],
    ["authorized", getAuthorizedDecoder()],
    ["lockup", getLockupDecoder()]
  ]);
}
function getMetaCodec() {
  return combineCodec(getMetaEncoder(), getMetaDecoder());
}
function getStakeEncoder() {
  return getStructEncoder([
    ["delegation", getDelegationEncoder()],
    ["creditsObserved", getU64Encoder()]
  ]);
}
function getStakeDecoder() {
  return getStructDecoder([
    ["delegation", getDelegationDecoder()],
    ["creditsObserved", getU64Decoder()]
  ]);
}
function getStakeCodec() {
  return combineCodec(getStakeEncoder(), getStakeDecoder());
}
var StakeAuthorize = /* @__PURE__ */ ((StakeAuthorize2) => {
  StakeAuthorize2[StakeAuthorize2["Staker"] = 0] = "Staker";
  StakeAuthorize2[StakeAuthorize2["Withdrawer"] = 1] = "Withdrawer";
  return StakeAuthorize2;
})(StakeAuthorize || {});
function getStakeAuthorizeEncoder() {
  return getEnumEncoder(StakeAuthorize);
}
function getStakeAuthorizeDecoder() {
  return getEnumDecoder(StakeAuthorize);
}
function getStakeAuthorizeCodec() {
  return combineCodec(getStakeAuthorizeEncoder(), getStakeAuthorizeDecoder());
}
function getStakeFlagsEncoder() {
  return getStructEncoder([["bits", getU8Encoder()]]);
}
function getStakeFlagsDecoder() {
  return getStructDecoder([["bits", getU8Decoder()]]);
}
function getStakeFlagsCodec() {
  return combineCodec(getStakeFlagsEncoder(), getStakeFlagsDecoder());
}
function getStakeStateEncoder() {
  return getDiscriminatedUnionEncoder(
    [
      ["Uninitialized", getUnitEncoder()],
      [
        "Initialized",
        getStructEncoder([["fields", getTupleEncoder([getMetaEncoder()])]])
      ],
      [
        "Stake",
        getStructEncoder([
          ["fields", getTupleEncoder([getMetaEncoder(), getStakeEncoder()])]
        ])
      ],
      ["RewardsPool", getUnitEncoder()]
    ],
    { size: getU32Encoder() }
  );
}
function getStakeStateDecoder() {
  return getDiscriminatedUnionDecoder(
    [
      ["Uninitialized", getUnitDecoder()],
      [
        "Initialized",
        getStructDecoder([["fields", getTupleDecoder([getMetaDecoder()])]])
      ],
      [
        "Stake",
        getStructDecoder([
          ["fields", getTupleDecoder([getMetaDecoder(), getStakeDecoder()])]
        ])
      ],
      ["RewardsPool", getUnitDecoder()]
    ],
    { size: getU32Decoder() }
  );
}
function getStakeStateCodec() {
  return combineCodec(getStakeStateEncoder(), getStakeStateDecoder());
}
function stakeState(kind, data) {
  return Array.isArray(data) ? { __kind: kind, fields: data } : { __kind: kind, ...data ?? {} };
}
function isStakeState(kind, value) {
  return value.__kind === kind;
}
function getStakeStateV2Encoder() {
  return getDiscriminatedUnionEncoder(
    [
      ["Uninitialized", getUnitEncoder()],
      [
        "Initialized",
        getStructEncoder([["fields", getTupleEncoder([getMetaEncoder()])]])
      ],
      [
        "Stake",
        getStructEncoder([
          [
            "fields",
            getTupleEncoder([
              getMetaEncoder(),
              getStakeEncoder(),
              getStakeFlagsEncoder()
            ])
          ]
        ])
      ],
      ["RewardsPool", getUnitEncoder()]
    ],
    { size: getU32Encoder() }
  );
}
function getStakeStateV2Decoder() {
  return getDiscriminatedUnionDecoder(
    [
      ["Uninitialized", getUnitDecoder()],
      [
        "Initialized",
        getStructDecoder([["fields", getTupleDecoder([getMetaDecoder()])]])
      ],
      [
        "Stake",
        getStructDecoder([
          [
            "fields",
            getTupleDecoder([
              getMetaDecoder(),
              getStakeDecoder(),
              getStakeFlagsDecoder()
            ])
          ]
        ])
      ],
      ["RewardsPool", getUnitDecoder()]
    ],
    { size: getU32Decoder() }
  );
}
function getStakeStateV2Codec() {
  return combineCodec(getStakeStateV2Encoder(), getStakeStateV2Decoder());
}
function stakeStateV2(kind, data) {
  return Array.isArray(data) ? { __kind: kind, fields: data } : { __kind: kind, ...data ?? {} };
}
function isStakeStateV2(kind, value) {
  return value.__kind === kind;
}

// src/generated/accounts/stakeStateAccount.ts
function getStakeStateAccountEncoder() {
  return getStructEncoder([["state", getStakeStateV2Encoder()]]);
}
function getStakeStateAccountDecoder() {
  return getStructDecoder([["state", getStakeStateV2Decoder()]]);
}
function getStakeStateAccountCodec() {
  return combineCodec(
    getStakeStateAccountEncoder(),
    getStakeStateAccountDecoder()
  );
}
function decodeStakeStateAccount(encodedAccount) {
  return decodeAccount(
    encodedAccount,
    getStakeStateAccountDecoder()
  );
}
async function fetchStakeStateAccount(rpc, address, config) {
  const maybeAccount = await fetchMaybeStakeStateAccount(rpc, address, config);
  assertAccountExists(maybeAccount);
  return maybeAccount;
}
async function fetchMaybeStakeStateAccount(rpc, address, config) {
  const maybeAccount = await fetchEncodedAccount(rpc, address, config);
  return decodeStakeStateAccount(maybeAccount);
}
async function fetchAllStakeStateAccount(rpc, addresses, config) {
  const maybeAccounts = await fetchAllMaybeStakeStateAccount(
    rpc,
    addresses,
    config
  );
  assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}
async function fetchAllMaybeStakeStateAccount(rpc, addresses, config) {
  const maybeAccounts = await fetchEncodedAccounts(rpc, addresses, config);
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
  if (containsBytes(data, getU32Encoder().encode(0), 0)) {
    return 0 /* Initialize */;
  }
  if (containsBytes(data, getU32Encoder().encode(1), 0)) {
    return 1 /* Authorize */;
  }
  if (containsBytes(data, getU32Encoder().encode(2), 0)) {
    return 2 /* DelegateStake */;
  }
  if (containsBytes(data, getU32Encoder().encode(3), 0)) {
    return 3 /* Split */;
  }
  if (containsBytes(data, getU32Encoder().encode(4), 0)) {
    return 4 /* Withdraw */;
  }
  if (containsBytes(data, getU32Encoder().encode(5), 0)) {
    return 5 /* Deactivate */;
  }
  if (containsBytes(data, getU32Encoder().encode(6), 0)) {
    return 6 /* SetLockup */;
  }
  if (containsBytes(data, getU32Encoder().encode(7), 0)) {
    return 7 /* Merge */;
  }
  if (containsBytes(data, getU32Encoder().encode(8), 0)) {
    return 8 /* AuthorizeWithSeed */;
  }
  if (containsBytes(data, getU32Encoder().encode(9), 0)) {
    return 9 /* InitializeChecked */;
  }
  if (containsBytes(data, getU32Encoder().encode(10), 0)) {
    return 10 /* AuthorizeChecked */;
  }
  if (containsBytes(data, getU32Encoder().encode(11), 0)) {
    return 11 /* AuthorizeCheckedWithSeed */;
  }
  if (containsBytes(data, getU32Encoder().encode(12), 0)) {
    return 12 /* SetLockupChecked */;
  }
  if (containsBytes(data, getU32Encoder().encode(13), 0)) {
    return 13 /* GetMinimumDelegation */;
  }
  if (containsBytes(data, getU32Encoder().encode(14), 0)) {
    return 14 /* DeactivateDelinquent */;
  }
  if (containsBytes(data, getU32Encoder().encode(16), 0)) {
    return 15 /* MoveStake */;
  }
  if (containsBytes(data, getU32Encoder().encode(17), 0)) {
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
  return isProgramError(
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
    const writableRole = account.isWritable ? AccountRole.WRITABLE : AccountRole.READONLY;
    return Object.freeze({
      address: expectAddress(account.value),
      role: isTransactionSigner(account.value) ? upgradeRoleToSigner(writableRole) : writableRole,
      ...isTransactionSigner(account.value) ? { signer: account.value } : {}
    });
  };
}
function isTransactionSigner(value) {
  return !!value && typeof value === "object" && "address" in value && isTransactionSigner$1(value);
}

// src/generated/instructions/authorize.ts
var AUTHORIZE_DISCRIMINATOR = 1;
function getAuthorizeDiscriminatorBytes() {
  return getU32Encoder().encode(AUTHORIZE_DISCRIMINATOR);
}
function getAuthorizeInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([
      ["discriminator", getU32Encoder()],
      ["arg0", getAddressEncoder()],
      ["arg1", getStakeAuthorizeEncoder()]
    ]),
    (value) => ({ ...value, discriminator: AUTHORIZE_DISCRIMINATOR })
  );
}
function getAuthorizeInstructionDataDecoder() {
  return getStructDecoder([
    ["discriminator", getU32Decoder()],
    ["arg0", getAddressDecoder()],
    ["arg1", getStakeAuthorizeDecoder()]
  ]);
}
function getAuthorizeInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(AUTHORIZE_CHECKED_DISCRIMINATOR);
}
function getAuthorizeCheckedInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([
      ["discriminator", getU32Encoder()],
      ["stakeAuthorize", getStakeAuthorizeEncoder()]
    ]),
    (value) => ({ ...value, discriminator: AUTHORIZE_CHECKED_DISCRIMINATOR })
  );
}
function getAuthorizeCheckedInstructionDataDecoder() {
  return getStructDecoder([
    ["discriminator", getU32Decoder()],
    ["stakeAuthorize", getStakeAuthorizeDecoder()]
  ]);
}
function getAuthorizeCheckedInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(AUTHORIZE_CHECKED_WITH_SEED_DISCRIMINATOR);
}
function getAuthorizeCheckedWithSeedInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([
      ["discriminator", getU32Encoder()],
      ["stakeAuthorize", getStakeAuthorizeEncoder()],
      [
        "authoritySeed",
        addEncoderSizePrefix(getUtf8Encoder(), getU32Encoder())
      ],
      ["authorityOwner", getAddressEncoder()]
    ]),
    (value) => ({
      ...value,
      discriminator: AUTHORIZE_CHECKED_WITH_SEED_DISCRIMINATOR
    })
  );
}
function getAuthorizeCheckedWithSeedInstructionDataDecoder() {
  return getStructDecoder([
    ["discriminator", getU32Decoder()],
    ["stakeAuthorize", getStakeAuthorizeDecoder()],
    ["authoritySeed", addDecoderSizePrefix(getUtf8Decoder(), getU32Decoder())],
    ["authorityOwner", getAddressDecoder()]
  ]);
}
function getAuthorizeCheckedWithSeedInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(AUTHORIZE_WITH_SEED_DISCRIMINATOR);
}
function getAuthorizeWithSeedInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([
      ["discriminator", getU32Encoder()],
      ["newAuthorizedPubkey", getAddressEncoder()],
      ["stakeAuthorize", getStakeAuthorizeEncoder()],
      [
        "authoritySeed",
        addEncoderSizePrefix(getUtf8Encoder(), getU32Encoder())
      ],
      ["authorityOwner", getAddressEncoder()]
    ]),
    (value) => ({ ...value, discriminator: AUTHORIZE_WITH_SEED_DISCRIMINATOR })
  );
}
function getAuthorizeWithSeedInstructionDataDecoder() {
  return getStructDecoder([
    ["discriminator", getU32Decoder()],
    ["newAuthorizedPubkey", getAddressDecoder()],
    ["stakeAuthorize", getStakeAuthorizeDecoder()],
    ["authoritySeed", addDecoderSizePrefix(getUtf8Decoder(), getU32Decoder())],
    ["authorityOwner", getAddressDecoder()]
  ]);
}
function getAuthorizeWithSeedInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(DEACTIVATE_DISCRIMINATOR);
}
function getDeactivateInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([["discriminator", getU32Encoder()]]),
    (value) => ({ ...value, discriminator: DEACTIVATE_DISCRIMINATOR })
  );
}
function getDeactivateInstructionDataDecoder() {
  return getStructDecoder([["discriminator", getU32Decoder()]]);
}
function getDeactivateInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(DEACTIVATE_DELINQUENT_DISCRIMINATOR);
}
function getDeactivateDelinquentInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([["discriminator", getU32Encoder()]]),
    (value) => ({
      ...value,
      discriminator: DEACTIVATE_DELINQUENT_DISCRIMINATOR
    })
  );
}
function getDeactivateDelinquentInstructionDataDecoder() {
  return getStructDecoder([["discriminator", getU32Decoder()]]);
}
function getDeactivateDelinquentInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(DELEGATE_STAKE_DISCRIMINATOR);
}
function getDelegateStakeInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([["discriminator", getU32Encoder()]]),
    (value) => ({ ...value, discriminator: DELEGATE_STAKE_DISCRIMINATOR })
  );
}
function getDelegateStakeInstructionDataDecoder() {
  return getStructDecoder([["discriminator", getU32Decoder()]]);
}
function getDelegateStakeInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(GET_MINIMUM_DELEGATION_DISCRIMINATOR);
}
function getGetMinimumDelegationInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([["discriminator", getU32Encoder()]]),
    (value) => ({
      ...value,
      discriminator: GET_MINIMUM_DELEGATION_DISCRIMINATOR
    })
  );
}
function getGetMinimumDelegationInstructionDataDecoder() {
  return getStructDecoder([["discriminator", getU32Decoder()]]);
}
function getGetMinimumDelegationInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(INITIALIZE_DISCRIMINATOR);
}
function getInitializeInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([
      ["discriminator", getU32Encoder()],
      ["arg0", getAuthorizedEncoder()],
      ["arg1", getLockupEncoder()]
    ]),
    (value) => ({ ...value, discriminator: INITIALIZE_DISCRIMINATOR })
  );
}
function getInitializeInstructionDataDecoder() {
  return getStructDecoder([
    ["discriminator", getU32Decoder()],
    ["arg0", getAuthorizedDecoder()],
    ["arg1", getLockupDecoder()]
  ]);
}
function getInitializeInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(INITIALIZE_CHECKED_DISCRIMINATOR);
}
function getInitializeCheckedInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([["discriminator", getU32Encoder()]]),
    (value) => ({ ...value, discriminator: INITIALIZE_CHECKED_DISCRIMINATOR })
  );
}
function getInitializeCheckedInstructionDataDecoder() {
  return getStructDecoder([["discriminator", getU32Decoder()]]);
}
function getInitializeCheckedInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(MERGE_DISCRIMINATOR);
}
function getMergeInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([["discriminator", getU32Encoder()]]),
    (value) => ({ ...value, discriminator: MERGE_DISCRIMINATOR })
  );
}
function getMergeInstructionDataDecoder() {
  return getStructDecoder([["discriminator", getU32Decoder()]]);
}
function getMergeInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(MOVE_LAMPORTS_DISCRIMINATOR);
}
function getMoveLamportsInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([
      ["discriminator", getU32Encoder()],
      ["args", getU64Encoder()]
    ]),
    (value) => ({ ...value, discriminator: MOVE_LAMPORTS_DISCRIMINATOR })
  );
}
function getMoveLamportsInstructionDataDecoder() {
  return getStructDecoder([
    ["discriminator", getU32Decoder()],
    ["args", getU64Decoder()]
  ]);
}
function getMoveLamportsInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(MOVE_STAKE_DISCRIMINATOR);
}
function getMoveStakeInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([
      ["discriminator", getU32Encoder()],
      ["args", getU64Encoder()]
    ]),
    (value) => ({ ...value, discriminator: MOVE_STAKE_DISCRIMINATOR })
  );
}
function getMoveStakeInstructionDataDecoder() {
  return getStructDecoder([
    ["discriminator", getU32Decoder()],
    ["args", getU64Decoder()]
  ]);
}
function getMoveStakeInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(SET_LOCKUP_DISCRIMINATOR);
}
function getSetLockupInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([
      ["discriminator", getU32Encoder()],
      ["unixTimestamp", getOptionEncoder(getI64Encoder())],
      ["epoch", getOptionEncoder(getU64Encoder())],
      ["custodian", getOptionEncoder(getAddressEncoder())]
    ]),
    (value) => ({ ...value, discriminator: SET_LOCKUP_DISCRIMINATOR })
  );
}
function getSetLockupInstructionDataDecoder() {
  return getStructDecoder([
    ["discriminator", getU32Decoder()],
    ["unixTimestamp", getOptionDecoder(getI64Decoder())],
    ["epoch", getOptionDecoder(getU64Decoder())],
    ["custodian", getOptionDecoder(getAddressDecoder())]
  ]);
}
function getSetLockupInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(SET_LOCKUP_CHECKED_DISCRIMINATOR);
}
function getSetLockupCheckedInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([
      ["discriminator", getU32Encoder()],
      ["unixTimestamp", getOptionEncoder(getI64Encoder())],
      ["epoch", getOptionEncoder(getU64Encoder())]
    ]),
    (value) => ({ ...value, discriminator: SET_LOCKUP_CHECKED_DISCRIMINATOR })
  );
}
function getSetLockupCheckedInstructionDataDecoder() {
  return getStructDecoder([
    ["discriminator", getU32Decoder()],
    ["unixTimestamp", getOptionDecoder(getI64Decoder())],
    ["epoch", getOptionDecoder(getU64Decoder())]
  ]);
}
function getSetLockupCheckedInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(SPLIT_DISCRIMINATOR);
}
function getSplitInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([
      ["discriminator", getU32Encoder()],
      ["args", getU64Encoder()]
    ]),
    (value) => ({ ...value, discriminator: SPLIT_DISCRIMINATOR })
  );
}
function getSplitInstructionDataDecoder() {
  return getStructDecoder([
    ["discriminator", getU32Decoder()],
    ["args", getU64Decoder()]
  ]);
}
function getSplitInstructionDataCodec() {
  return combineCodec(
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
  return getU32Encoder().encode(WITHDRAW_DISCRIMINATOR);
}
function getWithdrawInstructionDataEncoder() {
  return transformEncoder(
    getStructEncoder([
      ["discriminator", getU32Encoder()],
      ["args", getU64Encoder()]
    ]),
    (value) => ({ ...value, discriminator: WITHDRAW_DISCRIMINATOR })
  );
}
function getWithdrawInstructionDataDecoder() {
  return getStructDecoder([
    ["discriminator", getU32Decoder()],
    ["args", getU64Decoder()]
  ]);
}
function getWithdrawInstructionDataCodec() {
  return combineCodec(
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

export { AUTHORIZE_CHECKED_DISCRIMINATOR, AUTHORIZE_CHECKED_WITH_SEED_DISCRIMINATOR, AUTHORIZE_DISCRIMINATOR, AUTHORIZE_WITH_SEED_DISCRIMINATOR, DEACTIVATE_DELINQUENT_DISCRIMINATOR, DEACTIVATE_DISCRIMINATOR, DELEGATE_STAKE_DISCRIMINATOR, GET_MINIMUM_DELEGATION_DISCRIMINATOR, INITIALIZE_CHECKED_DISCRIMINATOR, INITIALIZE_DISCRIMINATOR, MERGE_DISCRIMINATOR, MOVE_LAMPORTS_DISCRIMINATOR, MOVE_STAKE_DISCRIMINATOR, SET_LOCKUP_CHECKED_DISCRIMINATOR, SET_LOCKUP_DISCRIMINATOR, SPLIT_DISCRIMINATOR, STAKE_ERROR__ALREADY_DEACTIVATED, STAKE_ERROR__CUSTODIAN_MISSING, STAKE_ERROR__CUSTODIAN_SIGNATURE_MISSING, STAKE_ERROR__EPOCH_REWARDS_ACTIVE, STAKE_ERROR__INSUFFICIENT_DELEGATION, STAKE_ERROR__INSUFFICIENT_REFERENCE_VOTES, STAKE_ERROR__INSUFFICIENT_STAKE, STAKE_ERROR__LOCKUP_IN_FORCE, STAKE_ERROR__MERGE_MISMATCH, STAKE_ERROR__MERGE_TRANSIENT_STAKE, STAKE_ERROR__MINIMUM_DELINQUENT_EPOCHS_FOR_DEACTIVATION_NOT_MET, STAKE_ERROR__NO_CREDITS_TO_REDEEM, STAKE_ERROR__REDELEGATED_STAKE_MUST_FULLY_ACTIVATE_BEFORE_DEACTIVATION_IS_PERMITTED, STAKE_ERROR__REDELEGATE_TO_SAME_VOTE_ACCOUNT, STAKE_ERROR__REDELEGATE_TRANSIENT_OR_INACTIVE_STAKE, STAKE_ERROR__TOO_SOON_TO_REDELEGATE, STAKE_ERROR__VOTE_ADDRESS_MISMATCH, STAKE_PROGRAM_ADDRESS, StakeAccount, StakeAuthorize, StakeInstruction, WITHDRAW_DISCRIMINATOR, decodeStakeStateAccount, fetchAllMaybeStakeStateAccount, fetchAllStakeStateAccount, fetchMaybeStakeStateAccount, fetchStakeStateAccount, getAuthorizeCheckedDiscriminatorBytes, getAuthorizeCheckedInstruction, getAuthorizeCheckedInstructionDataCodec, getAuthorizeCheckedInstructionDataDecoder, getAuthorizeCheckedInstructionDataEncoder, getAuthorizeCheckedWithSeedDiscriminatorBytes, getAuthorizeCheckedWithSeedInstruction, getAuthorizeCheckedWithSeedInstructionDataCodec, getAuthorizeCheckedWithSeedInstructionDataDecoder, getAuthorizeCheckedWithSeedInstructionDataEncoder, getAuthorizeDiscriminatorBytes, getAuthorizeInstruction, getAuthorizeInstructionDataCodec, getAuthorizeInstructionDataDecoder, getAuthorizeInstructionDataEncoder, getAuthorizeWithSeedDiscriminatorBytes, getAuthorizeWithSeedInstruction, getAuthorizeWithSeedInstructionDataCodec, getAuthorizeWithSeedInstructionDataDecoder, getAuthorizeWithSeedInstructionDataEncoder, getAuthorizedCodec, getAuthorizedDecoder, getAuthorizedEncoder, getDeactivateDelinquentDiscriminatorBytes, getDeactivateDelinquentInstruction, getDeactivateDelinquentInstructionDataCodec, getDeactivateDelinquentInstructionDataDecoder, getDeactivateDelinquentInstructionDataEncoder, getDeactivateDiscriminatorBytes, getDeactivateInstruction, getDeactivateInstructionDataCodec, getDeactivateInstructionDataDecoder, getDeactivateInstructionDataEncoder, getDelegateStakeDiscriminatorBytes, getDelegateStakeInstruction, getDelegateStakeInstructionDataCodec, getDelegateStakeInstructionDataDecoder, getDelegateStakeInstructionDataEncoder, getDelegationCodec, getDelegationDecoder, getDelegationEncoder, getGetMinimumDelegationDiscriminatorBytes, getGetMinimumDelegationInstruction, getGetMinimumDelegationInstructionDataCodec, getGetMinimumDelegationInstructionDataDecoder, getGetMinimumDelegationInstructionDataEncoder, getInitializeCheckedDiscriminatorBytes, getInitializeCheckedInstruction, getInitializeCheckedInstructionDataCodec, getInitializeCheckedInstructionDataDecoder, getInitializeCheckedInstructionDataEncoder, getInitializeDiscriminatorBytes, getInitializeInstruction, getInitializeInstructionDataCodec, getInitializeInstructionDataDecoder, getInitializeInstructionDataEncoder, getLockupCodec, getLockupDecoder, getLockupEncoder, getMergeDiscriminatorBytes, getMergeInstruction, getMergeInstructionDataCodec, getMergeInstructionDataDecoder, getMergeInstructionDataEncoder, getMetaCodec, getMetaDecoder, getMetaEncoder, getMoveLamportsDiscriminatorBytes, getMoveLamportsInstruction, getMoveLamportsInstructionDataCodec, getMoveLamportsInstructionDataDecoder, getMoveLamportsInstructionDataEncoder, getMoveStakeDiscriminatorBytes, getMoveStakeInstruction, getMoveStakeInstructionDataCodec, getMoveStakeInstructionDataDecoder, getMoveStakeInstructionDataEncoder, getSetLockupCheckedDiscriminatorBytes, getSetLockupCheckedInstruction, getSetLockupCheckedInstructionDataCodec, getSetLockupCheckedInstructionDataDecoder, getSetLockupCheckedInstructionDataEncoder, getSetLockupDiscriminatorBytes, getSetLockupInstruction, getSetLockupInstructionDataCodec, getSetLockupInstructionDataDecoder, getSetLockupInstructionDataEncoder, getSplitDiscriminatorBytes, getSplitInstruction, getSplitInstructionDataCodec, getSplitInstructionDataDecoder, getSplitInstructionDataEncoder, getStakeAuthorizeCodec, getStakeAuthorizeDecoder, getStakeAuthorizeEncoder, getStakeCodec, getStakeDecoder, getStakeEncoder, getStakeErrorMessage, getStakeFlagsCodec, getStakeFlagsDecoder, getStakeFlagsEncoder, getStakeStateAccountCodec, getStakeStateAccountDecoder, getStakeStateAccountEncoder, getStakeStateCodec, getStakeStateDecoder, getStakeStateEncoder, getStakeStateV2Codec, getStakeStateV2Decoder, getStakeStateV2Encoder, getWithdrawDiscriminatorBytes, getWithdrawInstruction, getWithdrawInstructionDataCodec, getWithdrawInstructionDataDecoder, getWithdrawInstructionDataEncoder, identifyStakeInstruction, isStakeError, isStakeState, isStakeStateV2, parseAuthorizeCheckedInstruction, parseAuthorizeCheckedWithSeedInstruction, parseAuthorizeInstruction, parseAuthorizeWithSeedInstruction, parseDeactivateDelinquentInstruction, parseDeactivateInstruction, parseDelegateStakeInstruction, parseGetMinimumDelegationInstruction, parseInitializeCheckedInstruction, parseInitializeInstruction, parseMergeInstruction, parseMoveLamportsInstruction, parseMoveStakeInstruction, parseSetLockupCheckedInstruction, parseSetLockupInstruction, parseSplitInstruction, parseWithdrawInstruction, stakeState, stakeStateV2 };
//# sourceMappingURL=index.mjs.map
//# sourceMappingURL=index.mjs.map