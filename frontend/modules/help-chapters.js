/** Help chapter content for the in-app Help view.
 *
 * Each chapter is one left-menu entry and, on macOS, one native Help menu item
 * (kept in sync in src-tauri/src/main.rs). Adding a chapter here adds it to the
 * in-app menu automatically.
 *
 * Keep this content in sync with the app: when a change affects what a section
 * describes, update the matching chapter in the same change.
 */
export const HELP_CHAPTERS = [
  {
    id: "about",
    label: "About",
    title: "About AI-Limits",
    render() {
      return `
        <p>
          AI-Limits shows how much of your Codex, Claude, and Cursor subscription
          limits you have left, in one place, without opening each provider's site.
        </p>
        <dl class="info-terms">
          <div>
            <dt>Free</dt>
            <dd>Open source.</dd>
          </div>
          <div>
            <dt>Private</dt>
            <dd>No AI-Limits sign-up or service between you and your providers.</dd>
          </div>
          <div>
            <dt>Cross-platform</dt>
            <dd>Runs on macOS, Windows, and Linux.</dd>
          </div>
          <div>
            <dt>Proactive</dt>
            <dd>Can notify you before you run out, so limits don't catch you mid-task.</dd>
          </div>
          <div>
            <dt>Local</dt>
            <dd>All data comes from your own setup or account.</dd>
          </div>
        </dl>
        <p class="app-version" data-app-version></p>
      `;
    },
  },
  {
    id: "providers",
    label: "Providers",
    title: "Providers",
    render() {
      return `
        <dl class="info-terms">
          <div>
            <dt>Codex</dt>
            <dd>Queries the Codex CLI over RPC, falling back to local Codex data if that isn't available.</dd>
          </div>
          <div>
            <dt>Claude</dt>
            <dd>Queries the Claude CLI over RPC, falling back to local Claude data if that isn't available.</dd>
          </div>
          <div>
            <dt>Cursor</dt>
            <dd>Uses the Cursor access token from Keychain to query the Cursor API.</dd>
          </div>
        </dl>
        <p>
          Each provider can be shown or hidden in settings. Hiding a provider also
          excludes it from the next refresh.
        </p>
      `;
    },
  },
  {
    id: "source",
    label: "Source",
    title: "Source",
    render() {
      return `
        <p>
          The <b>Source</b> toggle in Display settings adds a line under each
          provider's card showing exactly where its data came from, for
          example <code>RPC, as of today 10:10.</code> or <code>Local files, as of Aug 3, 10:10.</code>
        </p>
        <p>
          It's off by default; turn it on when you want to see, at a glance,
          whether a provider answered over RPC or fell back to local data.
        </p>
        <div class="info-links info-links--two-up" aria-label="Setup guides">
          <button type="button" class="info-link info-link--external" data-open-external="claude">
            Claude&nbsp;Code&nbsp;setup
          </button>
          <button type="button" class="info-link info-link--external" data-open-external="codex">
            Codex&nbsp;CLI
          </button>
        </div>
      `;
    },
  },
  {
    id: "data-errors",
    label: "Data availability",
    title: "When data is unavailable",
    render() {
      return `
        <p>
          AI-Limits can show limits only when a provider has current data available.
          If it cannot, the provider block explains what to do next.
        </p>
        <dl class="info-terms">
          <div>
            <dt>No fresh limits' data</dt>
            <dd>Neither RPC nor local data had a current reading for this provider.</dd>
          </div>
          <div>
            <dt>RPC access</dt>
            <dd>Codex and Claude are queried over RPC first. Install the CLI and sign in when the app asks.</dd>
          </div>
          <div>
            <dt>Local data</dt>
            <dd>Used when RPC isn't available. It may be missing or out of date.</dd>
          </div>
          <div>
            <dt>Cursor</dt>
            <dd>Cursor needs a valid local access token. Sign in to Cursor again if its token is missing or rejected.</dd>
          </div>
        </dl>
        <p class="info-note">Try a manual refresh after fixing access.</p>
        <div class="info-links">
          <button type="button" class="info-link info-link--internal" data-open-help="source">
            Source
          </button>
        </div>
      `;
    },
  },
  {
    id: "notifications",
    label: "Notifications",
    title: "Notifications",
    render() {
      return `
        <p>
          When enabled, AI-Limits sends a system notification for a
          provider's remaining limit, so you don't have to keep the app open
          to notice.
        </p>
        <dl class="info-terms">
          <div>
            <dt>Low remaining</dt>
            <dd>Fires when a remaining limit crosses a low threshold.</dd>
          </div>
          <div>
            <dt>100% again</dt>
            <dd>
              Fires on an exact return to 100% after a stored lower reading;
              first readings and partial rises below 100% don't notify.
            </dd>
          </div>
          <div>
            <dt>Control</dt>
            <dd>One toggle in settings covers every notification type.</dd>
          </div>
          <div>
            <dt>Platform</dt>
            <dd>macOS only for now; Windows and Linux are next.</dd>
          </div>
        </dl>
      `;
    },
  },
  {
    id: "updates",
    label: "Updates",
    title: "Updates",
    render() {
      return `
        <p>
          When automatic updates are on, AI-Limits looks for a new release and
          downloads it in the background. It never restarts on its own: the
          version you are using keeps running until you choose to restart.
        </p>
        <dl class="info-terms">
          <div>
            <dt>Schedule</dt>
            <dd>Once at startup, then every 12 hours while the app stays open.</dd>
          </div>
          <div>
            <dt>Applying</dt>
            <dd>A ready update shows a notice with a Restart now action.</dd>
          </div>
          <div>
            <dt>Control</dt>
            <dd>
              Turn automatic updates off in settings; the app then makes no
              update requests at all.
            </dd>
          </div>
          <div>
            <dt>Source</dt>
            <dd>
              Signed releases from the project's GitHub repository; an update
              that fails its signature check is discarded.
            </dd>
          </div>
          <div>
            <dt>Platform</dt>
            <dd>macOS only for now; Windows and Linux are next.</dd>
          </div>
        </dl>
      `;
    },
  },
  {
    id: "permissions",
    label: "Permissions",
    title: "Permissions",
    render() {
      return `
        <dl class="info-terms">
          <div>
            <dt>Network</dt>
            <dd>To query the Cursor API and check for available data.</dd>
          </div>
          <div>
            <dt>Keychain</dt>
            <dd>Read-only access to the Cursor access token.</dd>
          </div>
          <div>
            <dt>Local files</dt>
            <dd>Read-only access to local Codex and Claude data folders.</dd>
          </div>
          <div>
            <dt>Notifications</dt>
            <dd>To alert you when a limit runs low.</dd>
          </div>
          <div>
            <dt>Updates</dt>
            <dd>To download a new version when automatic updates are on.</dd>
          </div>
          <div>
            <dt>RPC</dt>
            <dd>Runs the <code>claude</code> or <code>codex</code> CLI over its RPC interface.</dd>
          </div>
        </dl>
        <p class="info-note">
          External links only open from a fixed allowlist. Nothing else is read,
          written, or sent anywhere.
        </p>
      `;
    },
  },
  {
    id: "cli-mode",
    label: "CLI mode",
    title: "CLI mode",
    render() {
      return `
        <p>
          The terminal interface is stateless: one query per run, no saved settings,
          no background refresh or notifications.
        </p>
        <dl class="info-terms">
          <div>
            <dt>Good for</dt>
            <dd>Scripts, CI, and quick checks, with plain, structured, or raw output.</dd>
          </div>
          <div>
            <dt>Not for</dt>
            <dd>Ongoing monitoring; use the desktop app for that.</dd>
          </div>
        </dl>
        <div class="cli-command-row" aria-label="Terminal command">
          <code data-cli-command>Loading command…</code>
          <button type="button" class="info-link cli-command-action" data-copy-cli-command disabled>Copy</button>
          <button type="button" class="info-link info-link--external cli-command-action" data-run-cli-command disabled>Run</button>
        </div>
      `;
    },
  },
  {
    id: "limitations",
    label: "Limitations",
    title: "Limitations",
    render() {
      return `
        <p>The desktop app is in beta and improving. Current known limits:</p>
        <dl class="info-terms">
          <div>
            <dt>macOS</dt>
            <dd>Signed and notarized; no DMG installer yet.</dd>
          </div>
          <div>
            <dt>Windows / Linux</dt>
            <dd>Builds are unsigned and still being tested with real users.</dd>
          </div>
          <div>
            <dt>Notifications</dt>
            <dd>macOS only for now.</dd>
          </div>
          <div>
            <dt>Local sources</dt>
            <dd>Some Codex and Claude local sources aren't available yet outside macOS.</dd>
          </div>
        </dl>
        <p class="info-note">RPC-backed sources are the most reliable option across platforms today.</p>
      `;
    },
  },
  {
    id: "for-developers",
    label: "For developers",
    title: "For developers",
    render() {
      return `
        <p>AI-Limits is open source under the MIT License.</p>
        <dl class="info-terms">
          <div>
            <dt>Core</dt>
            <dd>A shared Rust core powers both interfaces.</dd>
          </div>
          <div>
            <dt>Desktop</dt>
            <dd>A Tauri app with a web-technology UI.</dd>
          </div>
          <div>
            <dt>CLI</dt>
            <dd>A stateless command-line interface for scripts and CI.</dd>
          </div>
        </dl>
        <div class="info-links info-links--two-up">
          <button type="button" class="info-link info-link--external" data-open-external="github">
            Source&nbsp;on&nbsp;GitHub
          </button>
          <button type="button" class="info-link info-link--external" data-open-external="license">
            MIT&nbsp;License
          </button>
        </div>
      `;
    },
  },
];
export const DEFAULT_HELP_CHAPTER = HELP_CHAPTERS[0].id;
