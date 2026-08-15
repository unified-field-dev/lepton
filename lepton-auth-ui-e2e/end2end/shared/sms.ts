const SMS_SINK_URL = process.env.UF_SMS_SINK_URL ?? "http://127.0.0.1:8099";

/** Clear SMS sink messages (best-effort; no-op when the sink is down). */
export async function clearSmsSink(
  request: import("@playwright/test").APIRequestContext,
): Promise<void> {
  try {
    await request.delete(`${SMS_SINK_URL}/v1/messages`);
  } catch {
    // Sink not running yet / already torn down.
  }
}

/** Poll SMS sink for the latest OTP. */
export async function waitSmsOtp(
  request: import("@playwright/test").APIRequestContext,
  timeoutMs = 20_000,
): Promise<string> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const res = await request.get(`${SMS_SINK_URL}/v1/messages`);
    if (res.ok()) {
      const messages = (await res.json()) as Array<{
        otp_code?: string;
        body?: string;
      }>;
      const last = messages[messages.length - 1];
      if (last?.otp_code) {
        return last.otp_code;
      }
      const fromBody = last?.body?.match(/\b(\d{6})\b/);
      if (fromBody?.[1]) {
        return fromBody[1];
      }
    }
    await new Promise((r) => setTimeout(r, 250));
  }
  throw new Error("timed out waiting for SMS OTP from sink");
}
