import { expect } from "@playwright/test";

/** Result of `POST /api/test/seed-data` (harness-only). */
export type SeedResult = {
  scenario: string;
  email: string;
  password: string;
  reset_token?: string;
  totp_secret?: string;
};

/** Call the harness seed endpoint (named scenarios from `lepton-test-support`). */
export async function seedTestData(
  request: import("@playwright/test").APIRequestContext,
  scenario: string,
  data: Record<string, string> = {},
): Promise<SeedResult> {
  const res = await request.post("/api/test/seed-data", {
    data: { scenario, ...data },
  });
  expect(res.ok(), await res.text()).toBeTruthy();
  return (await res.json()) as SeedResult;
}
