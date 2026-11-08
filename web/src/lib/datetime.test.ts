import { describe, expect, it } from "vitest"
import { formatLocalTimestamp } from "./datetime"

describe("local timestamps", () => {
  it("pads every field and uses a 24-hour clock without changing the instant", () => {
    const localDate = new Date(2026, 0, 2, 3, 4, 5)
    expect(formatLocalTimestamp(localDate.toISOString(), "Never")).toBe("2026-01-02 03:04:05")
    const evening = new Date(2026, 10, 8, 21, 9, 15)
    expect(formatLocalTimestamp(evening.toISOString(), "Pending")).toBe("2026-11-08 21:09:15")
  })

  it("parses explicit offsets and rolls over dates", () => {
    const instant = new Date("2026-01-01T00:30:00+14:00")
    const expected = new Intl.DateTimeFormat("sv-SE", {
      year: "numeric", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", second: "2-digit", hourCycle: "h23",
    }).format(instant)
    expect(formatLocalTimestamp("2026-01-01T00:30:00+14:00", "Pending")).toBe(expected)
  })

  it("retains missing labels and handles malformed values", () => {
    expect(formatLocalTimestamp(null, "Never")).toBe("Never")
    expect(formatLocalTimestamp(null, "Pending")).toBe("Pending")
    expect(formatLocalTimestamp("invalid", "Never")).toBe("Unavailable")
    expect(formatLocalTimestamp("", "Pending")).toBe("Unavailable")
  })
})
