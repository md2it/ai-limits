import { colorForRemaining, escapeHtml, extractPlanNameForHeader, formatDecimal, formatSourceStatusLine, formatTimestampForDisplay, formatUpdateTimeLine } from "./provider-formatters.js";
import { isShowLimitsEnabled, isShowPlanEnabled, isShowSourceEnabled, isShowUpdateTimeEnabled } from "./settings.js";

function providerLabel(providerId) {
  return providerId.charAt(0).toUpperCase() + providerId.slice(1);
}

// Brand marks (Claude, OpenAI for codex, Cursor), traced at a shared 24x24
// viewBox so they all sit at the same visual weight next to the provider
// name. Rendered at 1em so they scale with the header's font-size (larger
// in the Main Window, smaller in the Popover) and colored via currentColor
// so they always match `.provider-name`'s text color (see
// --provider-name-text in styles/providers.css).
const PROVIDER_ICON_PATHS = {
  claude:
    "M4.709 15.955l4.72-2.647.08-.23-.08-.128H9.2l-.79-.048-2.698-.073-2.339-.097-2.266-.122-.571-.121L0 11.784l.055-.352.48-.321.686.06 1.52.103 2.278.158 1.652.097 2.449.255h.389l.055-.157-.134-.098-.103-.097-2.358-1.596-2.552-1.688-1.336-.972-.724-.491-.364-.462-.158-1.008.656-.722.881.06.225.061.893.686 1.908 1.476 2.491 1.833.365.304.145-.103.019-.073-.164-.274-1.355-2.446-1.446-2.49-.644-1.032-.17-.619a2.97 2.97 0 01-.104-.729L6.283.134 6.696 0l.996.134.42.364.62 1.414 1.002 2.229 1.555 3.03.456.898.243.832.091.255h.158V9.01l.128-1.706.237-2.095.23-2.695.08-.76.376-.91.747-.492.584.28.48.685-.067.444-.286 1.851-.559 2.903-.364 1.942h.212l.243-.242.985-1.306 1.652-2.064.73-.82.85-.904.547-.431h1.033l.76 1.129-.34 1.166-1.064 1.347-.881 1.142-1.264 1.7-.79 1.36.073.11.188-.02 2.856-.606 1.543-.28 1.841-.315.833.388.091.395-.328.807-1.969.486-2.309.462-3.439.813-.042.03.049.061 1.549.146.662.036h1.622l3.02.225.79.522.474.638-.079.485-1.215.62-1.64-.389-3.829-.91-1.312-.329h-.182v.11l1.093 1.068 2.006 1.81 2.509 2.33.127.578-.322.455-.34-.049-2.205-1.657-.851-.747-1.926-1.62h-.128v.17l.444.649 2.345 3.521.122 1.08-.17.353-.608.213-.668-.122-1.374-1.925-1.415-2.167-1.143-1.943-.14.08-.674 7.254-.316.37-.729.28-.607-.461-.322-.747.322-1.476.389-1.924.315-1.53.286-1.9.17-.632-.012-.042-.14.018-1.434 1.967-2.18 2.945-1.726 1.845-.414.164-.717-.37.067-.662.401-.589 2.388-3.036 1.44-1.882.93-1.086-.006-.158h-.055L4.132 18.56l-1.13.146-.487-.456.061-.746.231-.243 1.908-1.312-.006.006z",
  codex:
    "M9.205 8.658v-2.26c0-.19.072-.333.238-.428l4.543-2.616c.619-.357 1.356-.523 2.117-.523 2.854 0 4.662 2.212 4.662 4.566 0 .167 0 .357-.024.547l-4.71-2.759a.797.797 0 00-.856 0l-5.97 3.473zm10.609 8.8V12.06c0-.333-.143-.57-.429-.737l-5.97-3.473 1.95-1.118a.433.433 0 01.476 0l4.543 2.617c1.309.76 2.189 2.378 2.189 3.948 0 1.808-1.07 3.473-2.76 4.163zM7.802 12.703l-1.95-1.142c-.167-.095-.239-.238-.239-.428V5.899c0-2.545 1.95-4.472 4.591-4.472 1 0 1.927.333 2.712.928L8.23 5.067c-.285.166-.428.404-.428.737v6.898zM12 15.128l-2.795-1.57v-3.33L12 8.658l2.795 1.57v3.33L12 15.128zm1.796 7.23c-1 0-1.927-.332-2.712-.927l4.686-2.712c.285-.166.428-.404.428-.737v-6.898l1.974 1.142c.167.095.238.238.238.428v5.233c0 2.545-1.974 4.472-4.614 4.472zm-5.637-5.303l-4.544-2.617c-1.308-.761-2.188-2.378-2.188-3.948A4.482 4.482 0 014.21 6.327v5.423c0 .333.143.571.428.738l5.947 3.449-1.95 1.118a.432.432 0 01-.476 0zm-.262 3.9c-2.688 0-4.662-2.021-4.662-4.519 0-.19.024-.38.047-.57l4.686 2.71c.286.167.571.167.856 0l5.97-3.448v2.26c0 .19-.07.333-.237.428l-4.543 2.616c-.619.357-1.356.523-2.117.523zm5.899 2.83a5.947 5.947 0 005.827-4.756C22.287 18.339 24 15.84 24 13.296c0-1.665-.713-3.282-1.998-4.448.119-.5.19-.999.19-1.498 0-3.401-2.759-5.947-5.946-5.947-.642 0-1.26.095-1.88.31A5.962 5.962 0 0010.205 0a5.947 5.947 0 00-5.827 4.757C1.713 5.447 0 7.945 0 10.49c0 1.666.713 3.283 1.998 4.448-.119.5-.19 1-.19 1.499 0 3.401 2.759 5.946 5.946 5.946.642 0 1.26-.095 1.88-.309a5.96 5.96 0 004.162 1.713z",
  cursor:
    "M22.106 5.68L12.5.135a.998.998 0 00-.998 0L1.893 5.68a.84.84 0 00-.419.726v11.186c0 .3.16.577.42.727l9.607 5.547a.999.999 0 00.998 0l9.608-5.547a.84.84 0 00.42-.727V6.407a.84.84 0 00-.42-.726zm-.603 1.176L12.228 22.92c-.063.108-.228.064-.228-.061V12.34a.59.59 0 00-.295-.51l-9.11-5.26c-.107-.062-.063-.228.062-.228h18.55c.264 0 .428.286.296.514z",
};

function buildProviderBrandIconSvg(providerId) {
  const path = PROVIDER_ICON_PATHS[providerId];
  if (!path) {
    return "";
  }

  return `<svg class="provider-brand-icon" xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24" fill="currentColor" fill-rule="evenodd" aria-hidden="true"><path d="${path}"/></svg>`;
}

export function createEmptyProvider(providerId) {
  return {
    id: providerId,
    label: providerLabel(providerId),
    limits: [],
    availableLimitResets: null,
    plan: { lines: [], links: [] },
    sourceId: null,
    dataTimestamp: null,
    errorMessage: null,
    noFreshData: false,
    authorizationRequired: null,
    pending: true,
  };
}

// Key-value lines are emitted as three spans — label, separator, value —
// instead of one flat string. The Main Window concatenates them back into
// exactly the "Available credits: 23.8" line it always showed (the spans are
// inline and carry no styling of their own there), while the Popover lays
// them out as a label-left/value-right menu row and hides the separator, the
// way a system panel writes a key-value pair. See
// docs/desktop/mac-popover.md#visual-layer.
function buildKeyValueHtml(label, value) {
  return `<span class="kv-label">${escapeHtml(label)}</span><span class="kv-separator">: </span><span class="kv-value">${escapeHtml(value)}</span>`;
}

function formatCreditsValue(provider) {
  if (provider.creditsRemaining == null) {
    return "";
  }

  return formatDecimal(provider.creditsRemaining);
}

function buildLimitResetsHtml(provider) {
  if (Number(provider.availableLimitResets) <= 0) {
    return "";
  }

  return `
    <p class="credits-info">${buildKeyValueHtml("Available resets", String(provider.availableLimitResets))}</p>
  `;
}

function cliAuthorizationCopy(providerKey) {
  if (providerKey === "claude") {
    return { message: "You\u2019re not signed in to Claude CLI.", signInLabel: "Sign in to Claude", loginCommand: "claude login" };
  }

  return { message: "You\u2019re not signed in to Codex CLI.", signInLabel: "Sign in to Codex", loginCommand: "codex login" };
}

// "Fix access" only appears on the Main Window (surface === "main"): the
// Popover has no room for a Help chapter to open into, and its rendered HTML
// stays byte-for-byte the same as before this addition. See
// docs/desktop/mac-popover.md#card-content.
function buildCliAuthorizationHtml(providerKey, surface) {
  const copy = cliAuthorizationCopy(providerKey);
  const fixAccessHtml = surface === "main"
    ? `<button type="button" class="provider-link" data-open-help="permissions">Fix access</button>`
    : "";
  return `
    <div class="cli-authorization">
      <p class="provider-message">${escapeHtml(copy.message)}</p>
      <button type="button" class="provider-link provider-link--external" data-provider-cli-login="${escapeHtml(providerKey)}">
        ${escapeHtml(copy.signInLabel)}
      </button>
      ${fixAccessHtml}
      <p class="cli-authorization-manual">Or run manually: <code>${escapeHtml(copy.loginCommand)}</code></p>
    </div>
  `;
}

// The "Retry" button reuses data-manual-refresh (see providers.js), surfaced
// directly on the card for recovery states. Main Window only, same reasoning
// as buildCliAuthorizationHtml.
function buildNoFreshDataHtml(surface) {
  const retryHtml = surface === "main"
    ? `<button type="button" class="provider-link" data-manual-refresh>Retry</button>`
    : "";
  return `
    <div class="no-fresh-data">
      <p>No fresh limits' data.</p>
      <button type="button" class="provider-link" data-open-data-errors>
        More details
      </button>
      ${retryHtml}
    </div>
  `;
}

function buildLimitRowsHtml(provider, surface) {
  if (!provider.limits.length) {
    if (provider.pending || provider.availableLimitResets != null) {
      return "";
    }

    if (provider.authorizationRequired) {
      return buildCliAuthorizationHtml(provider.authorizationRequired, surface);
    }

    if (provider.noFreshData) {
      return buildNoFreshDataHtml(surface);
    }

    const message = escapeHtml(provider.errorMessage || "No usable limit records from this source");
    const cursorDetails = provider.id === "cursor" && provider.errorMessage === "Choose which Cursor account to use." ? `<button type="button" class="provider-link" data-cursor-settings>Choose account in Settings</button>` : "";
    const cursorLogin = provider.id === "cursor" && /Sign in to Cursor/.test(provider.errorMessage || "") ? `<p>Sign in through Cursor IDE or run <code>cursor agent login</code>, then retry.</p>` : "";
    const details = provider.errorMessage === "Local provider data is outdated" ? `<button type="button" class="provider-link" data-open-data-errors>More details</button>` : "";
    const retryHtml = surface === "main" ? `<button type="button" class="provider-link" data-manual-refresh>Retry</button>` : "";
    return `<div><p class="provider-message">${message}</p>${details}${cursorDetails}${cursorLogin}${retryHtml}</div>`;
  }

  const warning = provider.id === "cursor" && provider.errorMessage
    ? `<p class="provider-message" role="status">${escapeHtml(provider.errorMessage)}</p><button type="button" class="provider-link" data-manual-refresh>Retry</button>` : "";
  return warning + provider.limits.map((limit) => {
    const remaining = Number(limit.remainingPercentage) || 0;
    const percent = remaining.toFixed(1);
    const width = Math.max(0, Math.min(100, remaining));
    const formattedResetTime = formatTimestampForDisplay(limit.resetTime);
    const resetText = formattedResetTime ? `reset ${escapeHtml(formattedResetTime)}` : "";

    return `
      <div class="limit-row">
        <div class="limit-top"><span class="limit-label">${escapeHtml(limit.label)}</span><span class="limit-separator"> | </span><span class="limit-value">${percent}% left</span></div>
        <progress class="meter" data-remaining="${width}" value="${width}" max="100" aria-label="${escapeHtml(provider.label)} ${escapeHtml(limit.label)} ${percent}% left"></progress>
        ${resetText ? `<div class="limit-reset">${resetText}</div>` : ""}
      </div>
    `;
  }).join("");
}

// CSP forbids `style="..."` attributes (no `unsafe-inline`), so the
// interpolated fill color can't be written into the HTML string above.
// Setting individual CSSOM properties via `style.setProperty` is not an
// inline-style attribute and is unaffected by that restriction.
function applyMeterFillColors(root) {
  root.querySelectorAll(".meter[data-remaining]").forEach((meter) => {
    meter.style.setProperty("--meter-fill", colorForRemaining(Number(meter.dataset.remaining)));
  });
}

const SECTION_HEADINGS = {
  limits: "LIMITS",
  plan: "PLAN",
};

// A section slot is always rendered when its display toggle is on. Content
// adds the divider heading and body; an empty slot stays blank and is later
// sized to match the tallest visible slot of the same kind. While the
// provider is still pending (cold start / not yet fetched), the heading is
// known from the toggle alone, so it renders immediately with a placeholder
// body reserving roughly the space real content will need — this is what
// keeps the card from collapsing to nothing before the first fetch resolves.
function buildProviderSectionHtml(kind, bodyHtml, pending) {
  const heading = `<h3 class="section-heading"><span class="section-heading-label">${escapeHtml(SECTION_HEADINGS[kind])}</span></h3>`;

  if (bodyHtml) {
    return `
      <div class="provider-section provider-section--${kind}" data-section-slot="${kind}">
        ${heading}<div class="provider-section-body">${bodyHtml}</div>
      </div>
    `;
  }

  if (pending) {
    return `
      <div class="provider-section provider-section--${kind}" data-section-slot="${kind}">
        ${heading}<div class="provider-section-body provider-section-body--placeholder" aria-hidden="true"></div>
      </div>
    `;
  }

  return `
    <div class="provider-section provider-section--${kind}" data-section-slot="${kind}"></div>
  `;
}

function buildLimitsBodyHtml(provider, surface) {
  const limitRowsHtml = buildLimitRowsHtml(provider, surface);
  const creditsValue = formatCreditsValue(provider);
  const limitResetsHtml = buildLimitResetsHtml(provider);

  if (!limitRowsHtml && !creditsValue && !limitResetsHtml) {
    return "";
  }

  return `
    <div class="limits">${limitRowsHtml}</div>
    <p class="credits-info" ${creditsValue ? "" : "hidden"}>${buildKeyValueHtml("Available credits", creditsValue)}</p>
    <div class="limit-resets-slot">${limitResetsHtml}</div>
  `;
}

function buildPlanLinesHtml(plan) {
  return (plan?.lines ?? []).map((line) => `<p class="section-line">${escapeHtml(line)}</p>`).join("");
}

// The management links share one line, joined by a middot, per
// docs/desktop/ui/provider-block-content.md. A single link is rendered
// without a separator. Each link stays a button so the click keeps going
// through openExternalUrl rather than navigating the app window.
function buildPlanLinksHtml(plan) {
  const links = (plan?.links ?? []).filter((link) => link?.url && link?.label);

  if (!links.length) {
    return "";
  }

  const linksHtml = links.map((link) => `<button type="button" class="provider-link provider-link--external" data-plan-link-url="${escapeHtml(link.url)}">${escapeHtml(link.label)}</button>`)
    .join(`<span class="plan-links-separator" aria-hidden="true">·</span>`);

  return `<p class="plan-links">${linksHtml}</p>`;
}

function buildPlanBodyHtml(provider) {
  return `${buildPlanLinesHtml(provider.plan)}${buildPlanLinksHtml(provider.plan)}`;
}

function buildProviderSectionsHtml(provider, surface) {
  return [
    isShowLimitsEnabled() ? buildProviderSectionHtml("limits", buildLimitsBodyHtml(provider, surface), provider.pending) : "",
    isShowPlanEnabled() ? buildProviderSectionHtml("plan", buildPlanBodyHtml(provider), provider.pending) : "",
  ].join("");
}

// The provider label, plus its plan name where a surface wants it folded
// into the header instead of shown as its own section (see
// `.provider-plan-name` in styles/popover.css) — hidden by default
// (styles/providers.css) so the Main Window's own Subscription section stays
// the only place the plan name appears there.
function buildProviderTitleHtml(provider) {
  const planName = extractPlanNameForHeader(provider.plan);
  const planNameHtml = planName ? ` <span class="provider-plan-name">${escapeHtml(planName)}</span>` : "";
  return `${buildProviderBrandIconSvg(provider.id)}<span class="provider-name">${escapeHtml(provider.label)}</span>${planNameHtml}`;
}

function buildSourceLineHtml(provider) {
  return `<p class="source-line" ${isShowSourceEnabled() ? "" : "hidden"}>${escapeHtml(formatSourceStatusLine(provider))}</p>`;
}

function buildUpdateTimeLineHtml(provider, nextRefreshAt) {
  return `<p class="update-time-line" ${isShowUpdateTimeEnabled() ? "" : "hidden"}>${escapeHtml(formatUpdateTimeLine(provider, nextRefreshAt))}</p>`;
}

export function renderProvider(provider, nextRefreshAt, surface = "main") {
  const block = document.createElement("article");
  block.className = "provider-block";
  block.dataset.providerId = provider.id;
  block.dataset.surface = surface;

  block.innerHTML = `
    <div class="provider-refresh-glare" aria-hidden="true"></div>
    <div class="provider-content">
      <div class="provider-header">
        <h2>${buildProviderTitleHtml(provider)}</h2>
      </div>
      <div class="provider-sections">${buildProviderSectionsHtml(provider, surface)}</div>
    </div>
    <div class="provider-footer">
      ${buildUpdateTimeLineHtml(provider, nextRefreshAt)}
      ${buildSourceLineHtml(provider)}
    </div>
  `;
  applyMeterFillColors(block);

  return block;
}

export function updateProviderBlockData(block, provider, nextRefreshAt) {
  block.querySelector(".provider-header h2").innerHTML = buildProviderTitleHtml(provider);
  const sections = block.querySelector(".provider-sections");
  sections.innerHTML = buildProviderSectionsHtml(provider, block.dataset.surface || "main");
  applyMeterFillColors(sections);
  block.querySelector(".source-line").outerHTML = buildSourceLineHtml(provider);
  block.querySelector(".update-time-line").outerHTML = buildUpdateTimeLineHtml(provider, nextRefreshAt);
}

export function updateProviderUpdateTimeText(block, provider, nextRefreshAt) {
  const el = block.querySelector(".update-time-line");
  if (el) {
    el.textContent = formatUpdateTimeLine(provider, nextRefreshAt);
  }
}
