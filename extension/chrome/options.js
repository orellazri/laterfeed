const baseUrlInput = document.getElementById("baseUrl");
const authTokenInput = document.getElementById("authToken");
const saveBtn = document.getElementById("saveBtn");
const statusEl = document.getElementById("status");

function showStatus(message, type) {
  statusEl.textContent = message;
  statusEl.className = "status " + type;
  setTimeout(() => {
    statusEl.className = "status";
  }, 3000);
}

// Load saved settings
chrome.storage.sync.get(["baseUrl", "authToken"], (result) => {
  if (result.baseUrl) baseUrlInput.value = result.baseUrl;
  if (result.authToken) authTokenInput.value = result.authToken;
});

// Save settings
saveBtn.addEventListener("click", () => {
  const baseUrl = baseUrlInput.value.trim().replace(/\/+$/, "");
  const authToken = authTokenInput.value.trim();

  if (!baseUrl) {
    showStatus("Base URL is required.", "error");
    return;
  }

  if (!authToken) {
    showStatus("Auth Token is required.", "error");
    return;
  }

  try {
    new URL(baseUrl);
  } catch {
    showStatus("Please enter a valid URL.", "error");
    return;
  }

  chrome.storage.sync.set({ baseUrl, authToken }, () => {
    showStatus("Settings saved.", "success");
  });
});
