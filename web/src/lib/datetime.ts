export function formatLocalTimestamp(value: string | null, missingLabel: string): string {
  if (value === null) return missingLabel
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return "Unavailable"
  const pad = (part: number, width = 2) => String(part).padStart(width, "0")
  return `${pad(date.getFullYear(), 4)}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
}
