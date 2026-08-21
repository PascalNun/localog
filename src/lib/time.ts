/**
 * A moment in a recording, or a length of one, as a clock.
 *
 * Four components wrote this out themselves, and they did not agree: two floored
 * the milliseconds and one rounded them, so the same instant could read 1:23 in
 * the transcript and 1:24 in the recording review. Flooring is what a playhead
 * does — a moment shows the second it is inside, not the nearest one.
 *
 * Two other formats in the transcript are deliberately not this one: segment
 * timestamps are always hh:mm:ss so a column of them lines up, and the player's
 * own readout is always mm:ss.
 */

const pad = (value: number) => String(value).padStart(2, '0');

/** The hour is dropped when there is none: a meeting is usually minutes. */
export function clock(seconds: number): string {
  const whole = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(whole / 3600);
  const minutes = Math.floor((whole % 3600) / 60);
  const rest = whole % 60;
  return hours > 0 ? `${hours}:${pad(minutes)}:${pad(rest)}` : `${minutes}:${pad(rest)}`;
}

/** The same, from the milliseconds transcripts and recordings are measured in. */
export function clockFromMillis(milliseconds: number): string {
  return clock(milliseconds / 1000);
}
