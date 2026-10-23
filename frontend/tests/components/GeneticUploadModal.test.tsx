import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { GeneticUploadModal } from "@/components/plans/GeneticUploadModal";
import { GeneticVerificationAPI } from "@/app/lib/api/geneticVerification";

const wallet = vi.hoisted(() => ({
  address: "owner",
  networkPassphrase: "Test SDF Network ; September 2015",
  kit: { signMessage: vi.fn() },
}));
vi.mock("@/context/WalletContext", () => ({ useWallet: () => wallet }));

beforeEach(() => {
  // JSDOM has no native dialog implementation; browser checks cover focus/inertness.
  Object.defineProperty(HTMLDialogElement.prototype, "showModal", {
    configurable: true,
    value: function (this: HTMLDialogElement) {
      this.setAttribute("open", "");
    },
  });
  Object.defineProperty(HTMLDialogElement.prototype, "close", {
    configurable: true,
    value: function (this: HTMLDialogElement) {
      this.removeAttribute("open");
    },
  });
  wallet.kit.signMessage.mockResolvedValue({
    signedMessage: "signature",
    signerAddress: "owner",
  });
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.clearAllMocks();
});

function open() {
  const onClose = vi.fn();
  render(
    <GeneticUploadModal
      planId="plan-one"
      planTitle="Family plan"
      onClose={onClose}
    />,
  );
  return onClose;
}
function select(name = "sample.vcf") {
  fireEvent.change(screen.getByLabelText("Choose a file or drag it here"), {
    target: { files: [new File(["synthetic"], name)] },
  });
}

describe("genetic upload modal", () => {
  it("opens an accessible modal, locks scrolling, and restores it on unmount", () => {
    document.body.style.overflow = "auto";
    open();
    expect(
      screen.getByRole("dialog", { name: "Upload genetic proof" }),
    ).toBeVisible();
    expect(document.body.style.overflow).toBe("hidden");
    cleanup();
    expect(document.body.style.overflow).toBe("auto");
  });
  it("rejects invalid selection and multiple dropped files before upload", () => {
    const upload = vi.spyOn(
      GeneticVerificationAPI.prototype,
      "uploadGeneticData",
    );
    open();
    select("bad.exe");
    expect(screen.getByRole("alert")).toHaveTextContent("Choose a .vcf");
    const dropzone = screen.getByText(
      "Choose a file or drag it here",
    ).parentElement!;
    fireEvent.drop(dropzone, {
      dataTransfer: {
        files: [new File(["a"], "a.vcf"), new File(["b"], "b.vcf")],
      },
    });
    expect(screen.getByRole("alert")).toHaveTextContent("one genetic file");
    expect(
      screen.getByRole("button", { name: "Sign and submit proof" }),
    ).toBeDisabled();
    expect(upload).not.toHaveBeenCalled();
  });
  it("accepts drag-and-drop and allows removing a selected file", () => {
    open();
    fireEvent.drop(
      screen.getByText("Choose a file or drag it here").parentElement!,
      { dataTransfer: { files: [new File(["a"], "sample.fasta")] } },
    );
    expect(screen.getByText(/sample.fasta/)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Remove" }));
    expect(
      screen.getByRole("button", { name: "Sign and submit proof" }),
    ).toBeDisabled();
  });
  it("shows real stages, prevents duplicate submission, and reports submission separately from verification", async () => {
    let resolve!: () => void;
    const pending = new Promise<void>((done) => {
      resolve = done;
    });
    const upload = vi
      .spyOn(GeneticVerificationAPI.prototype, "uploadGeneticData")
      .mockImplementation(async (_id, _file, options) => {
        options.onStage?.("signing");
        expect(await options.signPayload("proof")).toBe("signature");
        options.onStage?.("uploading");
        await pending;
        return { dnaHash: "hash", status: "pending" };
      });
    open();
    select();
    fireEvent.click(
      screen.getByRole("button", { name: "Sign and submit proof" }),
    );
    await waitFor(() =>
      expect(screen.getByRole("status")).toHaveTextContent(
        "Submitting your signed proof",
      ),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Sign and submit proof" }),
    );
    expect(upload).toHaveBeenCalledTimes(1);
    expect(upload.mock.calls[0][0]).toBe("plan-one");
    await act(async () => {
      resolve();
    });
    expect(screen.getByRole("status")).toHaveTextContent("Proof submitted");
    expect(
      screen.queryByRole("button", { name: "Sign and submit proof" }),
    ).not.toBeInTheDocument();
  });
  it("shows an error and lets the user retry without selecting the file again", async () => {
    const upload = vi
      .spyOn(GeneticVerificationAPI.prototype, "uploadGeneticData")
      .mockRejectedValueOnce(new Error("Signature declined"))
      .mockResolvedValueOnce({ dnaHash: "hash", status: "pending" });
    open();
    select();
    fireEvent.click(
      screen.getByRole("button", { name: "Sign and submit proof" }),
    );
    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent("Signature declined"),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Sign and submit proof" }),
    );
    await waitFor(() =>
      expect(screen.getByRole("status")).toHaveTextContent("Proof submitted"),
    );
    expect(upload).toHaveBeenCalledTimes(2);
  });
  it("aborts an in-flight attempt when closed", async () => {
    const upload = vi
      .spyOn(GeneticVerificationAPI.prototype, "uploadGeneticData")
      .mockImplementation(() => new Promise(() => {}));
    const onClose = open();
    select();
    fireEvent.click(
      screen.getByRole("button", { name: "Sign and submit proof" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(upload.mock.calls[0][2].signal?.aborted).toBe(true);
    expect(onClose).toHaveBeenCalledOnce();
  });
});
