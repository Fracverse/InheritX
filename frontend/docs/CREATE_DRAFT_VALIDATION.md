# CREATE_DRAFT_VALIDATION

Documents the inline server-side draft validation wired into the Create Commitment wizard configure step.

---

## Endpoint

```
POST /api/commitments/validate
```

### Request body

All fields are optional in the body; the endpoint returns per-field errors for whatever is present and invalid.

| Field | Type | Rules |
|---|---|---|
| `title` | `string` | Min 3 characters |
| `net_amount` | `number` | Positive finite number |
| `currency_preference` | `string` | One of `XLM USDC EURC NGN KES BRL PHP EUR USD` (case-insensitive) |
| `beneficiary_name` | `string` | Min 2 characters |
| `bank_account_number` | `string` | 6–20 digits, no spaces |
| `bank_name` | `string` | Min 2 characters |
| `inactivity_days` | `number` | 30–3650 inclusive (omit field to skip validation) |

### Response

```ts
{
  valid: boolean;       // true only when errors is empty
  errors: Array<{
    field: string;      // matches the request field name
    message: string;    // human-readable message shown next to the input
  }>;
}
```

**HTTP status** is always `200` for validation results. `400` is returned only for a malformed JSON body.

---

## Component contract — `CreateCommitmentStepConfigure`

```tsx
<CreateCommitmentStepConfigure
  initialValues?: Partial<ConfigureFormValues>
  onAdvance: (values: ConfigureFormValues) => void
/>
```

### Validation lifecycle

1. User edits any field.
2. A 500 ms debounce timer starts; any new edit resets it.
3. After 500 ms, `POST /api/commitments/validate` is called with an `AbortController` signal.
4. If a new edit arrives before the response, the in-flight request is aborted.
5. On success, field errors are mapped to the corresponding inputs and the Continue button is disabled while errors exist.
6. If the endpoint is unreachable (network error), a non-blocking warning banner is shown and the Continue button remains enabled.
7. On unmount, the current in-flight request is aborted.

### Accessibility

Every field that has a server error gets:

- `aria-invalid="true"` on the `<input>` / `<select>`
- `aria-describedby="<fieldId>-error"` pointing to the error `<span>`
- The error `<span>` carries `role="alert"` for live-region announcement

The Continue button carries `aria-busy="true"` while a validation request is in progress.

---

## Test coverage

`tests/components/CreateCommitmentStepConfigure.test.tsx` covers:

| Scenario | What is asserted |
|---|---|
| Render | All 7 fields and Continue button present |
| `initialValues` | Pre-filled inputs shown |
| Debounce — no early call | Fetch not called before 500 ms |
| Debounce — fires at 500 ms | Fetch called once after timer expires |
| Debounce — timer reset | Intermediate edits do not fire an extra request |
| Field error mapping | Per-field message text rendered |
| `aria-invalid` | Set to `"true"` on errored field |
| `aria-describedby` | Points to `<fieldId>-error` span |
| Error cleared on valid | Messages disappear after server returns `valid: true` |
| Continue disabled on error | Button is disabled while errors present |
| Continue enabled on valid | Button enabled after clean validation |
| `onAdvance` called | Called with current values on valid submit |
| `onAdvance` not called | Not called when errors exist |
| Server-down fallback | Warning banner shown, Continue not disabled |
| Abort on new input | In-flight request aborted when user types again |
| Abort on unmount | Request aborted when component unmounts |
| Loading state | "Validating…" label while request is pending |
| Multiple simultaneous errors | All field messages rendered at once |
