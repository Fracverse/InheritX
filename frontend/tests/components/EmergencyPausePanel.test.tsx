import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

const wallet: {
  isConnected: boolean;
  address: string | null;
  openModal: ReturnType<typeof vi.fn>;
  signTransaction: ReturnType<typeof vi.fn>;
} = {
  isConnected: true,
  address: "GADMIN",
  openModal: vi.fn(),
  signTransaction: vi.fn(),
};
vi.mock("@/context/WalletContext", () => ({ useWallet: () => wallet }));

const invokeContractPause = vi.fn();
const readContractPaused = vi.fn();
vi.mock("@/app/services/inheritanceContractService", () => ({
  invokeContractPause: (...args: unknown[]) => invokeContractPause(...args),
  readContractPaused: (...args: unknown[]) => readContractPaused(...args),
}));

import { EmergencyPausePanel } from "@/components/admin/EmergencyPausePanel";

/** The dialog's confirm button (the panel's own button has the same name). */
const confirmButton = () => screen.getAllByRole("button", { name: "Pause contract" }).at(-1)!;

async function openDialog(user: ReturnType<typeof userEvent.setup>) {
  render(<EmergencyPausePanel />);
  await waitFor(() => expect(screen.getByTestId("pause-status")).toHaveTextContent("Active"));
  await user.click(screen.getByRole("button", { name: "Pause contract" }));
  return screen.getByRole("dialog", { name: "Pause the inheritance contract?" });
}

async function confirmPause(user: ReturnType<typeof userEvent.setup>) {
  await user.type(screen.getByRole("textbox"), "PAUSE");
  await user.click(screen.getByRole("checkbox"));
  await user.click(confirmButton());
}

describe("EmergencyPausePanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    wallet.isConnected = true;
    wallet.address = "GADMIN";
    readContractPaused.mockResolvedValue(false);
    invokeContractPause.mockImplementation(async ({ onProgress }) => {
      onProgress?.({ stage: "confirmed" });
      return { submission: { hash: "h" }, explorerUrl: "https://stellar.expert/tx/h" };
    });
  });

  it("shows the on-chain pause state", async () => {
    readContractPaused.mockResolvedValue(true);
    render(<EmergencyPausePanel />);

    await waitFor(() => expect(screen.getByTestId("pause-status")).toHaveTextContent("Paused"));
    expect(readContractPaused).toHaveBeenCalledWith("GADMIN");
    expect(screen.getByRole("button", { name: "Resume contract" })).toBeInTheDocument();
  });

  it("keeps confirm disabled until the word is typed and a co-admin approval is ticked", async () => {
    const user = userEvent.setup();
    const dialog = await openDialog(user);

    expect(dialog).toHaveAttribute("aria-modal", "true");
    expect(confirmButton()).toBeDisabled();
    await user.type(screen.getByRole("textbox"), "PAUSE");
    expect(confirmButton()).toBeDisabled();
    await user.click(screen.getByRole("checkbox"));
    expect(confirmButton()).toBeEnabled();
    expect(invokeContractPause).not.toHaveBeenCalled();
  });

  it("submits a signed pause for the connected admin after confirmation", async () => {
    const user = userEvent.setup();
    await openDialog(user);
    await confirmPause(user);

    expect(invokeContractPause).toHaveBeenCalledWith(
      expect.objectContaining({
        admin: "GADMIN",
        action: "pause",
        signTransaction: wallet.signTransaction,
      })
    );
    expect(await screen.findByRole("link", { name: /view transaction/i })).toHaveAttribute(
      "href",
      "https://stellar.expert/tx/h"
    );
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("closes the dialog on Escape without submitting", async () => {
    const user = userEvent.setup();
    await openDialog(user);
    await user.keyboard("{Escape}");

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(invokeContractPause).not.toHaveBeenCalled();
  });

  it("shows the error and returns to idle when the wallet declines", async () => {
    invokeContractPause.mockRejectedValue(new Error("Signature request was declined in the wallet."));
    const user = userEvent.setup();
    await openDialog(user);
    await confirmPause(user);

    expect(await screen.findByRole("alert")).toHaveTextContent("declined");
    expect(screen.getByRole("button", { name: "Pause contract" })).toBeEnabled();
  });

  it("asks a disconnected user to connect an admin wallet", () => {
    wallet.isConnected = false;
    wallet.address = null;
    render(<EmergencyPausePanel />);

    expect(screen.getByRole("button", { name: "Connect admin wallet" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Pause contract" })).not.toBeInTheDocument();
    expect(readContractPaused).not.toHaveBeenCalled();
  });
});
