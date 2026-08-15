const MAILPIT_URL = process.env.UF_MAILPIT_URL ?? "http://127.0.0.1:8025";

function extractVerificationCode(body: string): string | null {
  const labeled = body.match(/verification code is:\s*([A-Za-z0-9_-]+)/i);
  if (labeled?.[1]) {
    return labeled[1].trim();
  }
  const codeTag = body.match(/<code>([^<]+)<\/code>/i);
  if (codeTag?.[1]) {
    return codeTag[1].trim();
  }
  const hex = body.match(/\b([a-f0-9]{16,})\b/i);
  return hex?.[1]?.trim() ?? null;
}

/** Clear Mailpit inbox (best-effort; no-op when Mailpit is down). */
export async function clearMailpit(
  request: import("@playwright/test").APIRequestContext,
): Promise<void> {
  try {
    await request.delete(`${MAILPIT_URL}/api/v1/messages`);
  } catch {
    // Mailpit not running yet / already torn down.
  }
}

/** Poll Mailpit for a verification code addressed to `email`. */
export async function waitMailpitCode(
  request: import("@playwright/test").APIRequestContext,
  email: string,
  timeoutMs = 20_000,
): Promise<string> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const listRes = await request.get(`${MAILPIT_URL}/api/v1/messages`);
    if (listRes.ok()) {
      const page = (await listRes.json()) as {
        messages?: Array<{ ID: string; To?: Array<{ Address?: string }> }>;
      };
      const match = (page.messages ?? []).find((m) =>
        (m.To ?? []).some(
          (t) => (t.Address ?? "").toLowerCase() === email.toLowerCase(),
        ),
      );
      if (match?.ID) {
        const detailRes = await request.get(
          `${MAILPIT_URL}/api/v1/message/${match.ID}`,
        );
        if (detailRes.ok()) {
          const detail = (await detailRes.json()) as {
            Text?: string;
            HTML?: string;
          };
          const body = `${detail.Text ?? ""}\n${detail.HTML ?? ""}`;
          const code = extractVerificationCode(body);
          if (code) {
            return code;
          }
        }
      }
    }
    await new Promise((r) => setTimeout(r, 250));
  }
  throw new Error(`timed out waiting for Mailpit code for ${email}`);
}
