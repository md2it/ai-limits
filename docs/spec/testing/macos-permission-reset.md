# macOS Permission Reset

Use this procedure before a clean macOS permission check.

1. Install a signed and notarized GitHub build in a stable location, preferably:

```text
/Applications/AI Limits.app
```

2. Remove older app copies if they can affect which app macOS opens.

3. Reset TCC permissions for the desktop bundle:

```text
tccutil reset All com.ai-limits.desktop
```

4. Launch the app and run the permission scenario being checked.

5. For notifications, also check the system notification permission manually:

```text
System Settings -> Notifications -> AI Limits
```

6. For provider permission diagnostics, test one provider at a time and record which action triggers each macOS prompt.

7. If permission behavior remains inconsistent, log out and back in, or reboot macOS.
