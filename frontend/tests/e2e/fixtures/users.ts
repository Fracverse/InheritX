export const mockWallet = {
  id: "freighter",
  address: "GDE2KZQ4QGJZ5Z5QW2Y4B7Y6Q5D3P9V8N7M6L5K4J3H2G1FTEST",
  shortAddress: "GDE2...TEST",
};

export const adminUser = {
  email: "admin@inheritx.test",
  password: "test-password",
};

export const planFixture = {
  name: "Family Legacy Plan",
  description: "A deterministic E2E plan for loved ones.",
  amount: "2.5",
  transferDate: "2026-12-31",
  beneficiary: {
    name: "Ada Lovelace",
    email: "ada@example.com",
    relationship: "Daughter",
    allocation: "100",
  },
};

export const claimFixture = {
  beneficiaryName: "Jane Doe",
  beneficiaryEmail: "jane@example.com",
  claimCode: "102667",
};
