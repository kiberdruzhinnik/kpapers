import { defineConfig, devices } from "@playwright/test"

const isCI = process.env.CI === "true"

export default defineConfig({
  testDir: "./tests/browser",
  fullyParallel: true,
  forbidOnly: isCI,
  retries: 0,
  reporter: isCI ? "github" : "list",
  use: {
    baseURL: process.env.APP_BASE_URL ?? "http://127.0.0.1:8080",
    trace: "retain-on-failure",
    ...devices["Desktop Chrome"],
  },
})
