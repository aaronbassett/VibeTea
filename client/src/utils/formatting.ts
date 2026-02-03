/**
 * VibeTea Formatting Utilities
 *
 * Timestamp and duration formatting functions for consistent display
 * across the VibeTea client application.
 *
 * All functions are pure and handle invalid input gracefully.
 */

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/** Fallback string for invalid timestamp input */
const INVALID_TIMESTAMP_FALLBACK = '--:--:--';

/** Fallback string for invalid full timestamp input */
const INVALID_TIMESTAMP_FULL_FALLBACK = '----/--/-- --:--:--';

/** Fallback string for invalid relative time input */
const INVALID_RELATIVE_TIME_FALLBACK = 'unknown';

/** Fallback string for invalid duration input */
const INVALID_DURATION_FALLBACK = '0s';

/** Fallback string for invalid short duration input */
const INVALID_DURATION_SHORT_FALLBACK = '0:00';

/** Milliseconds in one second */
const MS_PER_SECOND = 1000;

/** Milliseconds in one minute */
const MS_PER_MINUTE = 60 * MS_PER_SECOND;

/** Milliseconds in one hour */
const MS_PER_HOUR = 60 * MS_PER_MINUTE;

/** Milliseconds in one day */
const MS_PER_DAY = 24 * MS_PER_HOUR;

/** Milliseconds in one week */
const MS_PER_WEEK = 7 * MS_PER_DAY;

// -----------------------------------------------------------------------------
// Helper Functions
// -----------------------------------------------------------------------------

/**
 * Parses an RFC 3339 timestamp string into a Date object.
 * Returns null if the timestamp is invalid.
 *
 * @param timestamp - RFC 3339 formatted timestamp string
 * @returns Date object or null if invalid
 */
function parseTimestamp(timestamp: string): Date | null {
  if (typeof timestamp !== 'string' || timestamp.trim() === '') {
    return null;
  }

  const date = new Date(timestamp);

  // Check for Invalid Date
  if (Number.isNaN(date.getTime())) {
    return null;
  }

  return date;
}

/**
 * Pads a number with leading zeros to ensure minimum width.
 *
 * @param value - Number to pad
 * @param width - Minimum width (default: 2)
 * @returns Padded string representation
 */
function padZero(value: number, width: number = 2): string {
  return String(value).padStart(width, '0');
}

/**
 * Checks if two dates are on the same calendar day.
 *
 * @param date1 - First date
 * @param date2 - Second date
 * @returns True if dates are on the same day
 */
function isSameDay(date1: Date, date2: Date): boolean {
  return (
    date1.getFullYear() === date2.getFullYear() &&
    date1.getMonth() === date2.getMonth() &&
    date1.getDate() === date2.getDate()
  );
}

/**
 * Checks if date1 is yesterday relative to date2.
 *
 * @param date1 - Date to check
 * @param date2 - Reference date (usually now)
 * @returns True if date1 is yesterday relative to date2
 */
function isYesterday(date1: Date, date2: Date): boolean {
  const yesterday = new Date(date2);
  yesterday.setDate(yesterday.getDate() - 1);
  return isSameDay(date1, yesterday);
}

// -----------------------------------------------------------------------------
// Timestamp Formatting Functions
// -----------------------------------------------------------------------------

/**
 * Formats an RFC 3339 timestamp for display as time only (HH:MM:SS).
 *
 * Uses the local timezone for display.
 *
 * @param timestamp - RFC 3339 formatted timestamp string (e.g., "2026-02-02T14:30:00Z")
 * @returns Formatted time string (e.g., "14:30:00") or fallback for invalid input
 *
 * @example
 * formatTimestamp("2026-02-02T14:30:00Z") // "14:30:00" (in UTC timezone)
 * formatTimestamp("invalid") // "--:--:--"
 */
export function formatTimestamp(timestamp: string): string {
  const date = parseTimestamp(timestamp);

  if (date === null) {
    return INVALID_TIMESTAMP_FALLBACK;
  }

  const hours = padZero(date.getHours());
  const minutes = padZero(date.getMinutes());
  const seconds = padZero(date.getSeconds());

  return `${hours}:${minutes}:${seconds}`;
}

/**
 * Formats an RFC 3339 timestamp with full date and time (YYYY-MM-DD HH:MM:SS).
 *
 * Uses the local timezone for display.
 *
 * @param timestamp - RFC 3339 formatted timestamp string (e.g., "2026-02-02T14:30:00Z")
 * @returns Formatted datetime string (e.g., "2026-02-02 14:30:00") or fallback for invalid input
 *
 * @example
 * formatTimestampFull("2026-02-02T14:30:00Z") // "2026-02-02 14:30:00" (in UTC timezone)
 * formatTimestampFull("invalid") // "----/--/-- --:--:--"
 */
export function formatTimestampFull(timestamp: string): string {
  const date = parseTimestamp(timestamp);

  if (date === null) {
    return INVALID_TIMESTAMP_FULL_FALLBACK;
  }

  const year = date.getFullYear();
  const month = padZero(date.getMonth() + 1);
  const day = padZero(date.getDate());
  const hours = padZero(date.getHours());
  const minutes = padZero(date.getMinutes());
  const seconds = padZero(date.getSeconds());

  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
}

/**
 * Formats an RFC 3339 timestamp as a human-readable relative time.
 *
 * Returns strings like "just now", "5m ago", "2h ago", "yesterday", "3d ago", "2w ago".
 * Uses the current time as the reference point.
 *
 * @param timestamp - RFC 3339 formatted timestamp string (e.g., "2026-02-02T14:30:00Z")
 * @param now - Optional reference time (defaults to current time, useful for testing)
 * @returns Relative time string (e.g., "5m ago") or fallback for invalid input
 *
 * @example
 * // Assuming current time is 2026-02-02T14:35:00Z
 * formatRelativeTime("2026-02-02T14:34:30Z") // "just now"
 * formatRelativeTime("2026-02-02T14:30:00Z") // "5m ago"
 * formatRelativeTime("2026-02-02T12:30:00Z") // "2h ago"
 * formatRelativeTime("2026-02-01T14:30:00Z") // "yesterday"
 * formatRelativeTime("invalid") // "unknown"
 */
export function formatRelativeTime(
  timestamp: string,
  now: Date = new Date()
): string {
  const date = parseTimestamp(timestamp);

  if (date === null) {
    return INVALID_RELATIVE_TIME_FALLBACK;
  }

  const diffMs = now.getTime() - date.getTime();

  // Handle future timestamps (negative diff) - show as "just now"
  if (diffMs < 0) {
    return 'just now';
  }

  // Less than 1 minute
  if (diffMs < MS_PER_MINUTE) {
    return 'just now';
  }

  // Less than 1 hour - show minutes
  if (diffMs < MS_PER_HOUR) {
    const minutes = Math.floor(diffMs / MS_PER_MINUTE);
    return `${minutes}m ago`;
  }

  // Less than 24 hours - show hours
  if (diffMs < MS_PER_DAY) {
    const hours = Math.floor(diffMs / MS_PER_HOUR);
    return `${hours}h ago`;
  }

  // Check if yesterday
  if (isYesterday(date, now)) {
    return 'yesterday';
  }

  // Less than 7 days - show days
  if (diffMs < MS_PER_WEEK) {
    const days = Math.floor(diffMs / MS_PER_DAY);
    return `${days}d ago`;
  }

  // More than a week - show weeks
  const weeks = Math.floor(diffMs / MS_PER_WEEK);
  return `${weeks}w ago`;
}

// -----------------------------------------------------------------------------
// Duration Formatting Functions
// -----------------------------------------------------------------------------

/**
 * Formats a duration in milliseconds to a human-readable form.
 *
 * Returns strings like "1h 30m", "5m 30s", "30s".
 * Only shows the two most significant units.
 *
 * @param milliseconds - Duration in milliseconds
 * @returns Formatted duration string (e.g., "1h 30m") or fallback for invalid input
 *
 * @example
 * formatDuration(5400000) // "1h 30m"
 * formatDuration(330000) // "5m 30s"
 * formatDuration(30000) // "30s"
 * formatDuration(0) // "0s"
 * formatDuration(-1000) // "0s"
 */
export function formatDuration(milliseconds: number): string {
  // Handle invalid input
  if (typeof milliseconds !== 'number' || Number.isNaN(milliseconds)) {
    return INVALID_DURATION_FALLBACK;
  }

  // Handle negative or zero duration
  if (milliseconds <= 0) {
    return INVALID_DURATION_FALLBACK;
  }

  const totalSeconds = Math.floor(milliseconds / MS_PER_SECOND);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  // Build result with up to two most significant units
  const parts: string[] = [];

  if (hours > 0) {
    parts.push(`${hours}h`);
    if (minutes > 0) {
      parts.push(`${minutes}m`);
    }
  } else if (minutes > 0) {
    parts.push(`${minutes}m`);
    if (seconds > 0) {
      parts.push(`${seconds}s`);
    }
  } else {
    parts.push(`${seconds}s`);
  }

  return parts.join(' ');
}

/**
 * Formats a duration in milliseconds to a compact digital clock format.
 *
 * Returns strings like "1:30:00" (hours), "5:30" (minutes), "0:30" (seconds).
 *
 * @param milliseconds - Duration in milliseconds
 * @returns Compact duration string (e.g., "1:30:00") or fallback for invalid input
 *
 * @example
 * formatDurationShort(5400000) // "1:30:00"
 * formatDurationShort(330000) // "5:30"
 * formatDurationShort(30000) // "0:30"
 * formatDurationShort(0) // "0:00"
 * formatDurationShort(-1000) // "0:00"
 */
export function formatDurationShort(milliseconds: number): string {
  // Handle invalid input
  if (typeof milliseconds !== 'number' || Number.isNaN(milliseconds)) {
    return INVALID_DURATION_SHORT_FALLBACK;
  }

  // Handle negative or zero duration
  if (milliseconds <= 0) {
    return INVALID_DURATION_SHORT_FALLBACK;
  }

  const totalSeconds = Math.floor(milliseconds / MS_PER_SECOND);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    // Format as H:MM:SS
    return `${hours}:${padZero(minutes)}:${padZero(seconds)}`;
  }

  // Format as M:SS (no leading zero on minutes for consistency with typical timers)
  return `${minutes}:${padZero(seconds)}`;
}
