#!/usr/bin/env python3
"""
KYC Webhook Testing Script
Sends test webhook events to the InheritX backend for KYC verification flow testing.

Usage:
    python test_kyc_webhook.py --status approved --wallet GDTEST123
    python test_kyc_webhook.py --status rejected --wallet GDTEST123 --reason "Document expired"
"""

import hmac
import hashlib
import json
import requests
import argparse


def generate_signature(secret: str, payload: str) -> str:
    """Generate HMAC-SHA256 signature for webhook payload."""
    mac = hmac.new(secret.encode(), payload.encode(), hashlib.sha256)
    return f"sha256={mac.hexdigest()}"


def send_webhook(
    url: str,
    secret: str,
    wallet_address: str,
    status: str,
    provider_ref: str = None,
    event_type: str = "kyc.status_update",
    rejection_reason: str = None,
    can_resubmit: bool = True
):
    """Send a test webhook to the backend."""

    # Build payload
    payload = {
        "wallet_address": wallet_address,
        "status": status,
        "provider_reference": provider_ref or f"test-ref-{wallet_address[:8]}",
        "event_type": event_type
    }

    # Add rejection-specific fields
    if status == "rejected":
        if rejection_reason:
            payload["rejection_reason"] = rejection_reason
        payload["can_resubmit"] = can_resubmit

    # Serialize payload (no spaces, consistent formatting)
    payload_str = json.dumps(payload, separators=(',', ':'), sort_keys=True)

    # Generate signature
    signature = generate_signature(secret, payload_str)

    # Send request
    headers = {
        "Content-Type": "application/json",
        "X-KYC-Signature": signature
    }

    print(f"\n{'='*60}")
    print(f"Sending webhook to: {url}")
    print(f"{'='*60}")
    print(f"Payload:\n{json.dumps(payload, indent=2)}")
    print(f"\nSignature: {signature}")
    print(f"{'='*60}\n")

    try:
        response = requests.post(url, headers=headers, data=payload_str, timeout=10)

        print(f"Status Code: {response.status_code}")
        print(f"Response:\n{json.dumps(response.json(), indent=2)}")

        if response.status_code == 200:
            print(f"\n✅ Webhook processed successfully!")
        else:
            print(f"\n❌ Webhook failed!")

        return response

    except Exception as e:
        print(f"\n❌ Error sending webhook: {e}")
        return None


def main():
    parser = argparse.ArgumentParser(description="Test KYC webhook endpoints")
    parser.add_argument(
        "--url",
        default="http://localhost:8080/api/kyc/webhook",
        help="Webhook endpoint URL"
    )
    parser.add_argument(
        "--secret",
        default="test-webhook-secret-123",
        help="KYC webhook secret"
    )
    parser.add_argument(
        "--wallet",
        required=True,
        help="Wallet address to test"
    )
    parser.add_argument(
        "--status",
        required=True,
        choices=["pending", "submitted", "approved", "rejected"],
        help="KYC status to set"
    )
    parser.add_argument(
        "--reason",
        help="Rejection reason (only for rejected status)"
    )
    parser.add_argument(
        "--no-resubmit",
        action="store_true",
        help="Set can_resubmit to false (for final rejections)"
    )
    parser.add_argument(
        "--event-type",
        default="kyc.status_update",
        help="Event type identifier"
    )

    args = parser.parse_args()

    send_webhook(
        url=args.url,
        secret=args.secret,
        wallet_address=args.wallet,
        status=args.status,
        event_type=args.event_type,
        rejection_reason=args.reason,
        can_resubmit=not args.no_resubmit
    )


if __name__ == "__main__":
    main()
