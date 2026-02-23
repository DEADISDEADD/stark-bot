/**
 * XMTP-compatible signer that proxies all signing through stark-bot's
 * internal wallet API. No private key ever crosses process boundaries.
 */

import { IdentifierKind } from "npm:@xmtp/node-sdk@^5.3.0";

const SELF_URL = Deno.env.get("STARKBOT_SELF_URL") || "http://localhost:3000";
const TOKEN = Deno.env.get("STARKBOT_INTERNAL_TOKEN");

if (!TOKEN) {
  console.error("[signer-proxy] STARKBOT_INTERNAL_TOKEN not set — signing will fail");
}

// Validate SELF_URL at startup to prevent SSRF via misconfiguration
try {
  const _parsed = new URL(SELF_URL);
  const host = _parsed.hostname;
  if (!["localhost", "127.0.0.1", "::1"].includes(host) && !host.endsWith(".internal") && !host.startsWith("10.") && !host.startsWith("172.") && !host.startsWith("192.168.")) {
    console.warn(`[signer-proxy] STARKBOT_SELF_URL points to non-local host: ${host} — ensure this is intentional`);
  }
} catch {
  console.error("[signer-proxy] Invalid STARKBOT_SELF_URL — signing proxy will fail");
}

/** Fetch the wallet address from the backend */
export async function getAddress() {
  const res = await fetch(`${SELF_URL}/api/internal/wallet/address`, {
    headers: { Authorization: `Bearer ${TOKEN}` },
  });
  if (!res.ok) {
    throw new Error(`Failed to get address: ${res.status} ${await res.text()}`);
  }
  const data = await res.json();
  return data.address;
}

/** Convert hex string to Uint8Array */
function hexToBytes(hex) {
  const clean = hex.replace(/^0x/, "");
  const bytes = new Uint8Array(clean.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(clean.substr(i * 2, 2), 16);
  }
  return bytes;
}

/** Convert Uint8Array to hex string */
function bytesToHex(bytes) {
  return Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("");
}

/**
 * Sign arbitrary bytes via the backend wallet provider.
 * @param {Uint8Array|string} message — bytes or utf8 string to sign
 * @returns {Promise<Uint8Array>} raw 65-byte signature
 */
export async function signMessage(message) {
  let body;
  if (message instanceof Uint8Array) {
    body = { message: "0x" + bytesToHex(message), encoding: "hex" };
  } else {
    body = { message: String(message), encoding: "utf8" };
  }

  const res = await fetch(`${SELF_URL}/api/internal/wallet/sign-message`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${TOKEN}`,
    },
    body: JSON.stringify(body),
  });

  if (!res.ok) {
    throw new Error(`sign-message failed: ${res.status} ${await res.text()}`);
  }

  const data = await res.json();
  if (!data.success) {
    throw new Error(`sign-message error: ${data.error}`);
  }

  return hexToBytes(data.signature);
}

/**
 * Create an XMTP-compatible signer object.
 * Satisfies the interface expected by @xmtp/node-sdk:
 *   { getIdentifier, signMessage, type }
 */
export async function createProxySigner() {
  const address = await getAddress();
  console.log(`[signer-proxy] Wallet address: ${address}`);

  return {
    type: "EOA",
    getIdentifier: () => ({
      identifier: address.toLowerCase(),
      identifierKind: IdentifierKind.Ethereum,
    }),
    signMessage: async (msg) => await signMessage(msg),
  };
}
