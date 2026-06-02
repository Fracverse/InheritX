import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import WitnessManagement from '../WitnessManagement';
import { vi } from 'vitest';

const sampleWitnesses = [
  { id: '1', name: 'Alice', email: 'a@example.com', status: 'pending', invited_at: '2026-01-01T00:00:00Z' },
  { id: '2', name: 'Bob', email: 'b@example.com', status: 'signed', invited_at: '2026-01-02T00:00:00Z', signed_at: '2026-01-03T00:00:00Z' },
];

describe('WitnessManagement', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('fetches and displays witnesses', async () => {
    global.fetch = vi.fn().mockResolvedValueOnce({ ok: true, json: async () => sampleWitnesses } as any);
    render(<WitnessManagement planId="plan1" />);
    expect(global.fetch).toHaveBeenCalledWith('/api/witness/plan1');
    await waitFor(() => expect(screen.getByText('Alice')).toBeInTheDocument());
    expect(screen.getByText('Bob')).toBeInTheDocument();
  });

  it('invites a witness and refreshes list', async () => {
    // first fetch
    global.fetch = vi.fn()
      .mockResolvedValueOnce({ ok: true, json: async () => sampleWitnesses } as any)
      // invite response
      .mockResolvedValueOnce({ ok: true, json: async () => ({ id: '3', name: 'Carol', email: 'c@example.com', status: 'pending', invited_at: new Date().toISOString() }) } as any)
      // refetch after invite
      .mockResolvedValueOnce({ ok: true, json: async () => [...sampleWitnesses, { id: '3', name: 'Carol', email: 'c@example.com', status: 'pending' }] } as any);

    render(<WitnessManagement planId="plan1" />);

    await waitFor(() => expect(screen.getByText('Alice')).toBeInTheDocument());

    fireEvent.change(screen.getByPlaceholderText('Witness Name'), { target: { value: 'Carol' } });
    fireEvent.change(screen.getByPlaceholderText('Witness Email'), { target: { value: 'c@example.com' } });
    fireEvent.click(screen.getByText('Send Invitation'));

    await waitFor(() => expect(screen.getByText('Carol')).toBeInTheDocument());
  });

  it('signs a witness', async () => {
    // initial fetch
    global.fetch = vi.fn()
      .mockResolvedValueOnce({ ok: true, json: async () => sampleWitnesses } as any)
      // sign POST
      .mockResolvedValueOnce({ ok: true, json: async () => ({ success: true }) } as any)
      // refetch after sign
      .mockResolvedValueOnce({ ok: true, json: async () => [
        { id: '1', name: 'Alice', email: 'a@example.com', status: 'signed', invited_at: '2026-01-01T00:00:00Z', signed_at: new Date().toISOString() },
        sampleWitnesses[1],
      ] } as any);

    render(<WitnessManagement planId="plan1" />);
    await waitFor(() => expect(screen.getByText('Alice')).toBeInTheDocument());

    // Click sign on Alice (pending)
    const signButton = screen.getAllByText('Sign')[0];
    fireEvent.click(signButton);

    await waitFor(() => expect(screen.getByText('Signed')).toBeInTheDocument());
  });
});
