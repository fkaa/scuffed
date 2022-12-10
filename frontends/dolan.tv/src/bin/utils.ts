export function delay(ms: number) {
  return new Promise((resolve) => setTimeout(() => resolve(true), ms))
}

const getTimeUnit = (difference: number, short: boolean) => {
  if (difference < 60) {
    // Seconds
    return `${difference}${short ? "s" : " second(s)"}`
  } else if (difference >= 60 && difference < 3600) {
    // Minutes
    return `${Math.floor(difference / 60)}${short ? "m" : " minute(s)"}`
  } else if (difference >= 3600 && difference < 86400) {
    // Hours
    return `${Math.floor(difference / 3600)}${short ? "h" : " hour(s)"}`
  } else if (difference >= 86400 && difference < 604800) {
    // Days
    return `${Math.floor(difference / 86400)}${short ? "d" : " day(s)"}`
  } else if (difference >= 604800 && difference < 7889231) {
    // Weeks
    return `${Math.floor(difference / 604800)}${short ? "w" : " week(s)"}`
  } else if (difference >= 7889231) {
    // Months
    return `${Math.floor(difference / 2628000)}${short ? "M" : " month(s)"}`
  }
}

export function getTimeFromTimestamp(timestamp: Date | number) {
  return new Date(timestamp).getTime()
}

export function getDurationSince(timestamp: number, short = true, ago = false) {
  if (!timestamp) return "never"

  const now = Date.now()
  const difference = Math.floor((now - getTimeFromTimestamp(timestamp)) / 1000)

  let returnString = getTimeUnit(difference, short)

  if (ago) returnString += " ago"

  return returnString
}

export function isEven(a: number): boolean {
  return a % 2 === 0
}
