const { invoke } = window.__TAURI__.core;

const setupView =
  document.querySelector("#setup-view");

const chatView =
  document.querySelector("#chat-view");

const modelPathInput =
  document.querySelector("#model-path");

const modelError =
  document.querySelector("#model-error");

const loadModelButton =
  document.querySelector("#load-model-button");

const changeModelButton =
  document.querySelector("#change-model-button");

const currentModel =
  document.querySelector("#current-model");

const clearButton =
  document.querySelector("#clear-button");

let history = [];

let chatEl;
let inputEl;
let modeEl;
let sendButtonEl;
let emptyStateEl;

function showSetup(path = "") {
  setupView.classList.remove("hidden");
  chatView.classList.add("hidden");

  modelPathInput.value = path;
}

function showChat(path) {
  setupView.classList.add("hidden");
  chatView.classList.remove("hidden");

  currentModel.textContent = path;
}

async function initialize() {
  try {
    const status =
      await invoke("initialize_model");

    if (status.loaded) {
      showChat(status.path);
    } else {
      showSetup(status.path ?? "");

      if (status.error) {
        modelError.textContent =
          status.error;
      }
    }
  } catch (error) {
    showSetup();

    modelError.textContent =
      `Error: ${error}`;
  }
}

function addMessage(role, text) {
  emptyStateEl?.remove();
  emptyStateEl = null;

  const messageEl = document.createElement("div");
  messageEl.className = `message ${role}`;

  const labelEl = document.createElement("span");
  labelEl.className = "message-label";
  labelEl.textContent = role === "user" ? "You" : "Model";

  const contentEl = document.createElement("div");
  contentEl.className = "message-content";
  contentEl.textContent = text;

  messageEl.append(labelEl, contentEl);

  chatEl.appendChild(messageEl);
  chatEl.scrollTop = chatEl.scrollHeight;

  return contentEl;
}

function buildPrompt() {
  return (
    history
      .map(({ role, content }) => {
        const name =
          role === "user"
            ? "User"
            : "Assistant";

        return `${name}: ${content}`;
      })
      .join("\n\n") + "\n\nAssistant:"
  );
}

function clearChat() {
  history = [];
  chatEl.innerHTML = "";
}


async function sendMessage() {
  const text = inputEl.value.trim();

  if (!text) {
    return;
  }

  const mode = modeEl.value;

  // New mode means forget previous conversation
  if (mode === "new") {
    clearChat();
  }

  history.push({
    role: "user",
    content: text,
  });

  addMessage("user", text);

  inputEl.value = "";
  inputEl.style.height = "auto";

  sendButtonEl.disabled = true;

  const responseEl =
    addMessage("assistant", "Thinking...");

  responseEl.classList.add("pending");

  try {
    let prompt;

    if (mode === "add") {
      prompt = buildPrompt();
    } else {
      prompt = text;
    }

    const response = await invoke("greet", {
      prompt: prompt,
    });

    responseEl.textContent = response;
    responseEl.classList.remove("pending");

    history.push({
      role: "assistant",
      content: response,
    });
  } catch (error) {
    responseEl.textContent = `Error: ${error}`;

    responseEl.classList.remove("pending");
    responseEl.classList.add("error");
  } finally {
    sendButtonEl.disabled = false;

    inputEl.focus();

    chatEl.scrollTop =
      chatEl.scrollHeight;
  }
}

window.addEventListener(
  "DOMContentLoaded",
  initialize
);

window.addEventListener("DOMContentLoaded", () => {
  chatEl =
    document.querySelector("#chat");

  inputEl =
    document.querySelector("#chat-input");

  modeEl =
    document.querySelector("#mode-select");

  sendButtonEl =
    document.querySelector("#send-button");

  emptyStateEl =
    document.querySelector("#empty-state");

  document
    .querySelector("#chat-form")
    .addEventListener("submit", (event) => {
      event.preventDefault();

      sendMessage();
    });

  // Enter sends message
  // Shift + Enter creates new line
  inputEl.addEventListener("keydown", (event) => {
    if (
      event.key === "Enter" &&
      !event.shiftKey
    ) {
      event.preventDefault();

      sendMessage();
    }
  });

  // Automatically grow textarea
  inputEl.addEventListener("input", () => {
    inputEl.style.height = "auto";

    inputEl.style.height =
      `${Math.min(inputEl.scrollHeight, 160)}px`;
  });
});

loadModelButton.addEventListener(
  "click",
  async () => {
    const path =
      modelPathInput.value.trim();

    if (!path) {
      modelError.textContent =
        "Enter a model path.";

      return;
    }

    modelError.textContent =
      "Loading model...";

    loadModelButton.disabled = true;

    if (currentModel.textContent === path) {
      loadModelButton.disabled = false;
      modelError.textContent =
        "Loading model...";
      showChat(currentModel.textContent);
    }

    try {
      const status =
        await invoke(
          "set_model_path",
          {
            modelPath: path,
          }
        );

      modelError.textContent = "";

      showChat(status.path);
    } catch (error) {
      modelError.textContent =
        `Could not load model: ${error}`;
    } finally {
      loadModelButton.disabled = false;
    }
  }
);

clearButton.addEventListener("click", () => { clearChat() });

changeModelButton.addEventListener(
  "click",
  () => {
    showSetup(
      currentModel.textContent
    );
  }
);
