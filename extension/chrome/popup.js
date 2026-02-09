const setupPrompt = document.getElementById("setupPrompt");
const mainForm = document.getElementById("mainForm");
const urlInput = document.getElementById("url");
const titleInput = document.getElementById("title");
const sourceTypeSelect = document.getElementById("sourceType");
const saveBtn = document.getElementById("saveBtn");
const statusEl = document.getElementById("status");
const openOptionsLink = document.getElementById("openOptions");

let config = {};

function showStatus(message, type) {
  statusEl.textContent = message;
  statusEl.className = "status " + type;
}

function clearStatus() {
  statusEl.className = "status";
}

// Open the options page when the link is clicked
openOptionsLink.addEventListener("click", (e) => {
  e.preventDefault();
  chrome.runtime.openOptionsPage();
});

// Initialize popup
chrome.storage.sync.get(["baseUrl", "authToken"], (result) => {
  if (!result.baseUrl || !result.authToken) {
    setupPrompt.style.display = "block";
    return;
  }

  config = result;
  mainForm.style.display = "block";

  // Fill URL from active tab
  chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
    if (tabs[0] && tabs[0].url) {
      urlInput.value = tabs[0].url;
    }
  });
});

// Handle form submission
saveBtn.addEventListener("click", async () => {
  clearStatus();

  const url = urlInput.value.trim();
  if (!url) {
    showStatus("URL is required.", "error");
    return;
  }

  const title = titleInput.value.trim() || null;
  const sourceType = sourceTypeSelect.value;

  const body = {
    url,
    title,
    source_type: sourceType,
  };

  saveBtn.disabled = true;
  saveBtn.textContent = "Saving...";

  try {
    const response = await fetch(`${config.baseUrl}/entries`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${config.authToken}`,
      },
      body: JSON.stringify(body),
    });

    if (response.ok) {
      showStatus("Saved!", "success");
      saveBtn.textContent = "Saved";
      setTimeout(() => window.close(), 1000);
    } else if (response.status === 401) {
      showStatus("Unauthorized. Check your auth token in settings.", "error");
      saveBtn.disabled = false;
      saveBtn.textContent = "Save";
    } else {
      const data = await response.json().catch(() => null);
      const msg = data?.message || `Error: ${response.status}`;
      showStatus(msg, "error");
      saveBtn.disabled = false;
      saveBtn.textContent = "Save";
    }
  } catch (err) {
    console.error("Laterfeed request failed:", err);
    showStatus(`Connection failed: ${err.message}`, "error");
    saveBtn.disabled = false;
    saveBtn.textContent = "Save";
  }
});
