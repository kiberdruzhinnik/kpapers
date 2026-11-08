import "@testing-library/jest-dom/vitest"
import { cleanup } from "@testing-library/react"
import { toast } from "sonner"
import { afterEach, vi } from "vitest"

afterEach(() => {
  cleanup()
  toast.dismiss()
  vi.unstubAllGlobals()
})
