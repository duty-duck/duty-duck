import humanizeDuration from "humanize-duration"

export const formatDuration = (duration: number, locale: string) => {
  return humanizeDuration(duration, {
    maxDecimalPoints: 0,
    language: locale,
    // only display seconds if the duration is less than a day
    units:
      duration >= 24 * 60 * 60000
        ? ["y", "mo", "d", "h", "m"]
        : ["y", "mo", "d", "h", "m", "s"],
  })
}

export function formatDurationFromDates(startDate: Date | string, endDate: Date | string, locale: string) {
  const start = new Date(startDate);
  const end = new Date(endDate);
  const duration = end.getTime() - start.getTime()
  return formatDuration(duration, locale)
}