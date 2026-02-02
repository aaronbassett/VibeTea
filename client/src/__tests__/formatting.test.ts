/**
 * Tests for VibeTea formatting utilities
 */

import { describe, it, expect } from 'vitest';
import {
  formatTimestamp,
  formatTimestampFull,
  formatRelativeTime,
  formatDuration,
  formatDurationShort,
} from '../utils/formatting';

describe('formatTimestamp', () => {
  it('formats a valid RFC 3339 timestamp to HH:MM:SS', () => {
    // Note: This test uses UTC time. The actual output depends on local timezone.
    // Using a UTC timestamp and checking the result matches the local conversion.
    const timestamp = '2026-02-02T14:30:45Z';
    const date = new Date(timestamp);
    const expected = [
      String(date.getHours()).padStart(2, '0'),
      String(date.getMinutes()).padStart(2, '0'),
      String(date.getSeconds()).padStart(2, '0'),
    ].join(':');

    expect(formatTimestamp(timestamp)).toBe(expected);
  });

  it('handles timestamps with timezone offsets', () => {
    const timestamp = '2026-02-02T14:30:45+05:30';
    const result = formatTimestamp(timestamp);
    expect(result).toMatch(/^\d{2}:\d{2}:\d{2}$/);
  });

  it('returns fallback for empty string', () => {
    expect(formatTimestamp('')).toBe('--:--:--');
  });

  it('returns fallback for invalid timestamp', () => {
    expect(formatTimestamp('not-a-date')).toBe('--:--:--');
  });

  it('returns fallback for whitespace-only input', () => {
    expect(formatTimestamp('   ')).toBe('--:--:--');
  });
});

describe('formatTimestampFull', () => {
  it('formats a valid RFC 3339 timestamp to YYYY-MM-DD HH:MM:SS', () => {
    const timestamp = '2026-02-02T14:30:45Z';
    const date = new Date(timestamp);
    const expected = [
      date.getFullYear(),
      '-',
      String(date.getMonth() + 1).padStart(2, '0'),
      '-',
      String(date.getDate()).padStart(2, '0'),
      ' ',
      String(date.getHours()).padStart(2, '0'),
      ':',
      String(date.getMinutes()).padStart(2, '0'),
      ':',
      String(date.getSeconds()).padStart(2, '0'),
    ].join('');

    expect(formatTimestampFull(timestamp)).toBe(expected);
  });

  it('handles timestamps with timezone offsets', () => {
    const timestamp = '2026-02-02T14:30:45-08:00';
    const result = formatTimestampFull(timestamp);
    expect(result).toMatch(/^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$/);
  });

  it('returns fallback for empty string', () => {
    expect(formatTimestampFull('')).toBe('----/--/-- --:--:--');
  });

  it('returns fallback for invalid timestamp', () => {
    expect(formatTimestampFull('invalid')).toBe('----/--/-- --:--:--');
  });
});

describe('formatRelativeTime', () => {
  // Use a fixed reference time for consistent tests
  const now = new Date('2026-02-02T14:30:00Z');

  it('returns "just now" for timestamps less than a minute ago', () => {
    expect(formatRelativeTime('2026-02-02T14:29:30Z', now)).toBe('just now');
    expect(formatRelativeTime('2026-02-02T14:29:59Z', now)).toBe('just now');
  });

  it('returns "just now" for future timestamps', () => {
    expect(formatRelativeTime('2026-02-02T14:35:00Z', now)).toBe('just now');
  });

  it('returns minutes ago for timestamps less than an hour', () => {
    expect(formatRelativeTime('2026-02-02T14:29:00Z', now)).toBe('1m ago');
    expect(formatRelativeTime('2026-02-02T14:25:00Z', now)).toBe('5m ago');
    expect(formatRelativeTime('2026-02-02T13:31:00Z', now)).toBe('59m ago');
  });

  it('returns hours ago for timestamps less than a day', () => {
    expect(formatRelativeTime('2026-02-02T13:30:00Z', now)).toBe('1h ago');
    expect(formatRelativeTime('2026-02-02T12:30:00Z', now)).toBe('2h ago');
    expect(formatRelativeTime('2026-02-01T15:30:00Z', now)).toBe('23h ago');
  });

  it('returns "yesterday" for timestamps from yesterday', () => {
    // Create a reference time at noon local time, and a timestamp from yesterday at noon
    // This ensures the "yesterday" detection works regardless of timezone
    const refTime = new Date();
    refTime.setHours(12, 0, 0, 0); // Today at noon local time

    const yesterdayNoon = new Date(refTime);
    yesterdayNoon.setDate(yesterdayNoon.getDate() - 1); // Yesterday at noon local time

    expect(formatRelativeTime(yesterdayNoon.toISOString(), refTime)).toBe(
      'yesterday'
    );
  });

  it('returns days ago for timestamps less than a week', () => {
    expect(formatRelativeTime('2026-01-30T14:30:00Z', now)).toBe('3d ago');
    expect(formatRelativeTime('2026-01-27T14:30:00Z', now)).toBe('6d ago');
  });

  it('returns weeks ago for timestamps more than a week old', () => {
    expect(formatRelativeTime('2026-01-26T14:30:00Z', now)).toBe('1w ago');
    expect(formatRelativeTime('2026-01-19T14:30:00Z', now)).toBe('2w ago');
    expect(formatRelativeTime('2025-12-01T14:30:00Z', now)).toBe('9w ago');
  });

  it('returns fallback for invalid timestamp', () => {
    expect(formatRelativeTime('invalid')).toBe('unknown');
  });

  it('returns fallback for empty string', () => {
    expect(formatRelativeTime('')).toBe('unknown');
  });
});

describe('formatDuration', () => {
  it('formats hours and minutes', () => {
    expect(formatDuration(5400000)).toBe('1h 30m'); // 1.5 hours
    expect(formatDuration(3600000)).toBe('1h'); // 1 hour exactly
    expect(formatDuration(7200000)).toBe('2h'); // 2 hours exactly
    expect(formatDuration(7260000)).toBe('2h 1m'); // 2 hours 1 minute
  });

  it('formats minutes and seconds', () => {
    expect(formatDuration(330000)).toBe('5m 30s'); // 5.5 minutes
    expect(formatDuration(60000)).toBe('1m'); // 1 minute exactly
    expect(formatDuration(120000)).toBe('2m'); // 2 minutes exactly
    expect(formatDuration(90000)).toBe('1m 30s'); // 1.5 minutes
  });

  it('formats seconds only', () => {
    expect(formatDuration(30000)).toBe('30s');
    expect(formatDuration(1000)).toBe('1s');
    expect(formatDuration(59000)).toBe('59s');
  });

  it('omits seconds when hours are present', () => {
    // 1 hour, 0 minutes, 30 seconds -> should show "1h" only
    expect(formatDuration(3630000)).toBe('1h');
  });

  it('returns fallback for zero', () => {
    expect(formatDuration(0)).toBe('0s');
  });

  it('returns fallback for negative values', () => {
    expect(formatDuration(-1000)).toBe('0s');
    expect(formatDuration(-5400000)).toBe('0s');
  });

  it('returns fallback for NaN', () => {
    expect(formatDuration(NaN)).toBe('0s');
  });

  it('handles large durations', () => {
    // 48 hours = 172800000ms
    expect(formatDuration(172800000)).toBe('48h');
    // 100 hours = 360000000ms
    expect(formatDuration(360000000)).toBe('100h');
  });
});

describe('formatDurationShort', () => {
  it('formats hours:minutes:seconds for durations >= 1 hour', () => {
    expect(formatDurationShort(5400000)).toBe('1:30:00'); // 1.5 hours
    expect(formatDurationShort(3600000)).toBe('1:00:00'); // 1 hour
    expect(formatDurationShort(3661000)).toBe('1:01:01'); // 1h 1m 1s
    expect(formatDurationShort(7200000)).toBe('2:00:00'); // 2 hours
  });

  it('formats minutes:seconds for durations < 1 hour', () => {
    expect(formatDurationShort(330000)).toBe('5:30'); // 5.5 minutes
    expect(formatDurationShort(60000)).toBe('1:00'); // 1 minute
    expect(formatDurationShort(90000)).toBe('1:30'); // 1.5 minutes
  });

  it('formats seconds with leading zero for durations < 1 minute', () => {
    expect(formatDurationShort(30000)).toBe('0:30'); // 30 seconds
    expect(formatDurationShort(1000)).toBe('0:01'); // 1 second
    expect(formatDurationShort(9000)).toBe('0:09'); // 9 seconds
  });

  it('returns fallback for zero', () => {
    expect(formatDurationShort(0)).toBe('0:00');
  });

  it('returns fallback for negative values', () => {
    expect(formatDurationShort(-1000)).toBe('0:00');
  });

  it('returns fallback for NaN', () => {
    expect(formatDurationShort(NaN)).toBe('0:00');
  });

  it('handles large durations', () => {
    // 48 hours
    expect(formatDurationShort(172800000)).toBe('48:00:00');
    // 100 hours, 30 minutes, 45 seconds
    expect(formatDurationShort(361845000)).toBe('100:30:45');
  });
});
