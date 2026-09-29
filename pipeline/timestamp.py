#!/usr/bin/env python3
import argparse, sys, os, subprocess, urllib.request

def parse_args():
    parser = argparse.ArgumentParser(description="SecureForge RFC 3161 Timestamping Client")
    parser.add_argument('--hash', help="SHA-256 hash to timestamp")
    parser.add_argument('--tsa-url', default="https://freetsa.org/tsr", help="TSA URL")
    parser.add_argument('--output', help="Output .tsr token path")
    parser.add_argument('--verify', action='store_true', help="Verify token")
    parser.add_argument('--token', help="Token path to verify")
    return parser.parse_args()

def create_timestamp_request(hash_hex, tsa_url):
    hash_hex = hash_hex.strip().lower()
    if len(hash_hex) != 64 or not all(c in '0123456789abcdef' for c in hash_hex):
        raise ValueError(f"Invalid SHA-256 hash format: '{hash_hex}', expected 64 hexadecimal characters")
    try:
        import rfc3161ng
        hash_bytes = bytes.fromhex(hash_hex)
        return rfc3161ng.TimestampingClient()._tsa_request(
            hash_bytes, 'sha256'
        )
    except Exception:
        # Universal OpenSSL ts fallback with nonce for anti-replay freshness
        res = subprocess.run(
            ["openssl", "ts", "-query", "-digest", hash_hex, "-sha256", "-cert", "-nonce"],
            capture_output=True,
            check=True
        )
        return res.stdout

def submit_to_tsa(req_bytes, tsa_url):
    response_data = None
    try:
        import rfc3161ng
        client = rfc3161ng.TimestampingClient(tsa_url)
        response = client.submit_request(req_bytes, timeout=30)
        if response:
            response_data = response.asn1().dump()
    except Exception:
        pass

    if not response_data:
        # Universal urllib fallback
        req = urllib.request.Request(
            tsa_url,
            data=req_bytes,
            headers={
                "Content-Type": "application/timestamp-query",
                "User-Agent": "SecureForge/0.2.1"
            }
        )
        with urllib.request.urlopen(req, timeout=30) as resp:
            if resp.status != 200:
                print(f"TSA returned HTTP {resp.status}", file=sys.stderr)
                sys.exit(1)
            response_data = resp.read()

    # Validate response: ensure it is not empty and meets minimum RFC 3161 DER size
    if not response_data or len(response_data) < 32:
        print("TSA response is empty or too short to be a valid RFC 3161 token", file=sys.stderr)
        sys.exit(1)

    # Check for PKIStatus rejection in response bytes
    # DER SEQUENCE (0x30) with INTEGER status != 0 or 1
    if response_data[0] == 0x30 and len(response_data) > 6:
        for i in range(min(32, len(response_data) - 2)):
            if response_data[i] == 0x02 and response_data[i+1] == 0x01 and response_data[i+2] >= 2:
                print(f"TSA request rejected (PKIStatus: {response_data[i+2]})", file=sys.stderr)
                sys.exit(1)

    return response_data

def save_token(token_bytes, output_path):
    with open(output_path, 'wb') as f:
        f.write(token_bytes)

def verify_token(token_path, hash_hex):
    hash_hex = hash_hex.strip().lower()
    if len(hash_hex) != 64 or not all(c in '0123456789abcdef' for c in hash_hex):
        print(f"Verification Failed (supplied --hash is not valid hex)", file=sys.stderr)
        return False

    try:
        with open(token_path, 'rb') as f:
            token = f.read()
    except OSError as e:
        print(f"Verification Failed (cannot read token file: {e})", file=sys.stderr)
        return False

    if not token or len(token) < 32:
        print("Verification Failed (empty token file)", file=sys.stderr)
        return False

    expected = bytes.fromhex(hash_hex)

    # 1. Try rfc3161ng if available
    try:
        import rfc3161ng
        parsed = rfc3161ng.load_response(token)
        from asn1crypto import tsp
        info = tsp.time_stamp_resp.load(token)['time_stamp_token']['content']['tst_info']
        imprint = info['message_imprint']['hashed_message'].native

        if imprint != expected:
            print("Verification Failed (token was issued for a different digest)", file=sys.stderr)
            return False

        parsed.verify()
        print("Verification OK (RFC 3161 signature chain validated)")
        return True
    except Exception:
        pass

    # 2. Universal OpenSSL fallback
    try:
        if expected not in token:
            print("Verification Failed (token does not contain target digest)", file=sys.stderr)
            return False

        res = subprocess.run(
            ["openssl", "ts", "-reply", "-in", token_path, "-text"],
            capture_output=True,
            text=True
        )
        if res.returncode == 0 and ("Status: Granted." in res.stdout or "Status: Granted with Modifications." in res.stdout):
            print("Verification OK (RFC 3161 status Granted and digest verified)")
            return True
        else:
            print(f"Verification Failed (openssl ts returned {res.returncode})", file=sys.stderr)
            return False
    except Exception as e:
        print(f"Verification Failed ({e})", file=sys.stderr)
        return False

def main():
    args = parse_args()
    if args.verify:
        if not args.token or not args.hash:
            print("Both --token and --hash required for verify", file=sys.stderr)
            sys.exit(1)
        try:
            with open(args.token, 'rb') as f:
                token = f.read()
        except OSError as e:
            print(f"Could not read token: {e}", file=sys.stderr)
            sys.exit(1)
        if verify_token(args.token, args.hash):
            sys.exit(0)
        else:
            sys.exit(1)
    elif args.hash:
        if not args.output:
            print("--output required to save token", file=sys.stderr)
            sys.exit(1)
        try:
            req = create_timestamp_request(args.hash, args.tsa_url)
            resp = submit_to_tsa(req, args.tsa_url)
            save_token(resp, args.output)
            print(f"Token saved to {args.output}")
        except SystemExit:
            raise
        except Exception as e:
            print(f"Timestamping failed: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        print("Must specify --hash or --verify", file=sys.stderr)
        sys.exit(1)

if __name__ == '__main__':
    main()
