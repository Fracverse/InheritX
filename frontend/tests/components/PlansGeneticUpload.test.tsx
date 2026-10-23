import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import PlansPage from "@/app/asset-owner/plans/page";

vi.mock("@/context/WalletContext", () => ({
  useWallet: () => ({
    address: "owner",
    isConnected: true,
    openModal: vi.fn(),
  }),
}));
vi.mock("@/lib/api/dataSource", () => ({
  getPlans: async () => [
    {
      id: "owned-plan",
      title: "My plan",
      owner_address: "owner",
      status: "active",
    },
    {
      id: "other-plan",
      title: "Another plan",
      owner_address: "other",
      status: "active",
    },
  ],
}));
vi.mock("@/components/plans/BeneficiaryMFAModal", () => ({
  BeneficiaryMFAModal: () => null,
}));
vi.mock("@/components/plans/GeneticUploadModal", () => ({
  GeneticUploadModal: ({
    planId,
    planTitle,
    onClose,
  }: {
    planId: string;
    planTitle: string;
    onClose: () => void;
  }) => (
    <div role="dialog" aria-label={planTitle}>
      <span>{planId}</span>
      <button onClick={onClose}>Close upload</button>
    </div>
  ),
}));
afterEach(cleanup);

it("opens the upload for the selected owned plan and excludes other owners", async () => {
  render(<PlansPage />);
  const buttons = await screen.findAllByRole("button", {
    name: "Upload genetic proof",
  });
  expect(buttons).toHaveLength(1);
  fireEvent.click(buttons[0]);
  expect(screen.getByRole("dialog", { name: "My plan" })).toHaveTextContent(
    "owned-plan",
  );
  fireEvent.click(screen.getByRole("button", { name: "Close upload" }));
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
});
