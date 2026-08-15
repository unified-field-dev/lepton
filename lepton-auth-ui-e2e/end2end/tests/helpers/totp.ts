import { authenticator } from "otplib";

/** Current 6-digit TOTP for a base32 secret (harness seeds or enroll UI). */
export function totpCode(secret: string): string {
  return authenticator.generate(normalizeBase32(secret));
}

/** Strip spaces / lowercase for otplib / authenticator apps. */
export function normalizeBase32(secret: string): string {
  return secret.replace(/\s+/g, "").toUpperCase();
}

/** Read manual-entry secret text from the enroll Scan step. */
export async function totpSecretFromManualLocator(
  locator: import("@playwright/test").Locator,
): Promise<string> {
  const raw = (await locator.innerText()).trim();
  const secret = normalizeBase32(raw);
  if (!secret) {
    throw new Error("empty TOTP manual secret from page");
  }
  return secret;
}

/** Extract `secret=` from an otpauth URI (optional `data-otpauth` seam). */
export function totpSecretFromOtpauthUri(uri: string): string {
  const trimmed = uri.trim();
  if (!trimmed) {
    throw new Error("empty otpauth URI");
  }
  let parsed: URL;
  try {
    parsed = new URL(trimmed);
  } catch {
    throw new Error(`unparseable otpauth URI: ${trimmed.slice(0, 48)}`);
  }
  const secret = parsed.searchParams.get("secret")?.trim() ?? "";
  if (!secret) {
    throw new Error("otpauth URI missing secret=");
  }
  return normalizeBase32(secret);
}
