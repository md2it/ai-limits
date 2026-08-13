# Tauri UI Controls

## Update Frequency

The Settings page Other section contains one Update frequency dropdown. It applies to every enabled provider card; individual cards have no refresh-frequency control.

Default value: `10 min`.

Changing the setting applies immediately to every enabled provider and updates each source line's next-update time (see [provider-block-content.md](provider-block-content.md#source-line)). The next automatic refresh is always one shared interval after that provider's last update, whether the update came from the automatic timer or `UPDATE ALL DATA NOW`. Selecting a new frequency does not reset the countdown to a fresh interval from that moment; it recomputes the target using the new interval length. Two edge cases both resolve to fetching immediately instead of scheduling a wait:

- the provider has no known last update yet (nothing fetched this session), or
- the recomputed target is already in the past — for example the frequency was just changed to something shorter than the time already elapsed since the last update.
