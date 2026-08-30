import { initTheme, applyAppTheme, setManualTheme } from "./theme.js";
import {
  initSettings,
  syncSettingsInputs,
  handleSettingsChange,
  handleDisplaySettingsChange,
  handleUpdateFrequencyChange,
} from "./settings.js";
import { initAppUpdates, syncAppUpdateSchedule } from "./app-update.js";
import { initHelp, renderHelpMenu, openHelp, selectHelpChapter } from "./help.js";
import { DEFAULT_HELP_CHAPTER } from "./help-chapters.js";
import { updateFrequencyOptions } from "./constants.js";
import { setupScreenshotShowcase } from "./showcase.js";
import {
  initProviders,
  initProviderIntervals,
  listenForSystemThemeMeterRefresh,
  refreshEnabledProviders,
  refreshProviderSectionsFromCache,
  removeDisabledProviderBlocks,
  scheduleSectionSlotAlignment,
  restoreNewlyEnabledProviders,
  applySharedUpdateFrequency,
  syncBackgroundRefreshSchedule,
} from "./providers.js";

const providerList = document.querySelector("#provider-list");
const statusLine = document.querySelector("#status-line");
const refreshButton = document.querySelector("#refresh-button");
const overviewView = document.querySelector("#overview-view");
const settingsView = document.querySelector("#settings-view");
const helpView = document.querySelector("#help-view");
const helpMenu = document.querySelector("#help-menu");
const helpContent = document.querySelector("#help-content");
const navTabs = document.querySelectorAll(".app-nav-tab");
const views = {
  overview: overviewView,
  settings: settingsView,
  help: helpView,
};
const settingInputs = {
  notifications: document.querySelector("#setting-notifications"),
  cursor: document.querySelector("#setting-cursor"),
  cloud: document.querySelector("#setting-cloud"),
  codex: document.querySelector("#setting-codex"),
  showLimits: document.querySelector("#setting-show-limits"),
  showPlan: document.querySelector("#setting-show-plan"),
  showSource: document.querySelector("#setting-show-source"),
  showUpdateTime: document.querySelector("#setting-show-update-time"),
  updateFrequency: document.querySelector("#setting-update-frequency"),
  autoUpdate: document.querySelector("#setting-auto-update"),
  darkTheme: document.querySelector("#setting-dark-theme"),
};

const updateBannerEls = {
  banner: document.querySelector("#update-banner"),
  bannerText: document.querySelector("#update-banner-text"),
  restartButton: document.querySelector("#update-restart"),
  dismissButton: document.querySelector("#update-dismiss"),
};

// Shows the requested top-level section and hides the other two, and marks
// the matching nav tab current. This is the single place view visibility is
// decided — help.js/settings.js only render into their sections, they never
// toggle `hidden` themselves.
function switchView(view) {
  for (const [name, section] of Object.entries(views)) {
    section.hidden = name !== view;
  }
  for (const tab of navTabs) {
    tab.setAttribute("aria-current", tab.dataset.view === view ? "page" : "false");
  }
}

function currentView() {
  return Object.keys(views).find((name) => !views[name].hidden) ?? "overview";
}

initTheme(settingInputs.darkTheme);
initProviders({ providerList, statusLine });
initSettings(settingInputs, {
  onChanged({ newlyEnabled }) {
    removeDisabledProviderBlocks();
    restoreNewlyEnabledProviders(newlyEnabled);
    syncBackgroundRefreshSchedule();
    syncAppUpdateSchedule();
  },
  onDisplayChanged() {
    refreshProviderSectionsFromCache();
  },
  onUpdateFrequencyChanged() {
    applySharedUpdateFrequency();
    syncBackgroundRefreshSchedule();
  },
});
for (const option of updateFrequencyOptions) {
  settingInputs.updateFrequency.append(new Option(option, option));
}
settingInputs.updateFrequency.addEventListener("change", () => {
  handleUpdateFrequencyChange(settingInputs.updateFrequency.value);
});
initAppUpdates(updateBannerEls);
initHelp({ helpMenu, helpContent, helpView });

for (const tab of navTabs) {
  tab.addEventListener("click", () => {
    const view = tab.dataset.view;
    switchView(view);
    if (view === "help") {
      selectHelpChapter(DEFAULT_HELP_CHAPTER);
    }
  });
}

// Bridge for the macOS native Help menu (see src-tauri/src/main.rs).
window.__openHelpFromNative = (chapterId) => {
  switchView("help");
  openHelp(chapterId);
};

// Bridge for the macOS native AI Limits > Settings… menu item (see src-tauri/src/main.rs).
window.__openSettingsFromNative = () => {
  switchView("settings");
};

const displaySettingInputs = [
  settingInputs.showLimits,
  settingInputs.showPlan,
  settingInputs.showSource,
  settingInputs.showUpdateTime,
];

for (const input of Object.values(settingInputs)) {
  if (input === settingInputs.updateFrequency) {
    continue;
  }

  if (input === settingInputs.darkTheme) {
    input.addEventListener("change", (event) => {
      setManualTheme(event.target.checked);
      refreshProviderSectionsFromCache();
    });
    continue;
  }

  if (displaySettingInputs.includes(input)) {
    input.addEventListener("change", handleDisplaySettingsChange);
    continue;
  }

  input.addEventListener("change", handleSettingsChange);
}

listenForSystemThemeMeterRefresh();

window.addEventListener("resize", scheduleSectionSlotAlignment);

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && currentView() !== "overview") {
    switchView("overview");
  }
});

renderHelpMenu();
setupScreenshotShowcase();
initProviderIntervals();
applyAppTheme();
syncSettingsInputs();
syncBackgroundRefreshSchedule();
refreshButton.addEventListener("click", () => {
  refreshEnabledProviders();
});
refreshEnabledProviders({ initial: true });
