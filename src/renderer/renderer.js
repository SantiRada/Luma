const searchInput = document.querySelector('#search');
const list = document.querySelector('#list');
const message = document.querySelector('#message');
const installButton = document.querySelector('#install');
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const appWindow = window.__TAURI__.window.getCurrentWindow();

let tools = [];
let selectedIndex = 0;
let lastMessage = '';

function searchableText(tool) {
  return [tool.name, tool.description, ...(tool.tags || [])].join(' ').toLowerCase();
}

function filteredTools() {
  const query = searchInput.value.trim().toLowerCase();
  if (!query) return tools;
  return tools.filter((tool) => searchableText(tool).includes(query));
}

function iconFor(tool) {
  if (tool.icon) return tool.icon;
  return tool.name.split(/\s+/).slice(0, 2).map((part) => part[0]).join('').toUpperCase();
}

function render() {
  const visibleTools = filteredTools();
  selectedIndex = Math.min(selectedIndex, Math.max(visibleTools.length - 1, 0));
  list.innerHTML = '';

  if (!visibleTools.length) {
    const empty = document.createElement('div');
    empty.className = 'empty';
    empty.textContent = tools.length ? 'No hay resultados.' : 'Todavia no hay Actions instaladas.';
    list.appendChild(empty);
    return;
  }

  visibleTools.forEach((tool, index) => {
    const item = document.createElement('button');
    item.className = 'tool';
    item.type = 'button';
    item.dataset.actionId = tool.id;
    item.setAttribute('role', 'option');
    item.setAttribute('aria-selected', String(index === selectedIndex));
    item.addEventListener('mouseenter', () => {
      selectedIndex = index;
      updateSelection();
    });
    item.addEventListener('pointerdown', (event) => {
      event.preventDefault();
      runTool(tool);
    });

    const icon = document.createElement('span');
    icon.className = 'tool-icon';
    icon.textContent = iconFor(tool);

    const copy = document.createElement('span');
    const name = document.createElement('p');
    name.className = 'tool-name';
    name.textContent = tool.name;
    const description = document.createElement('p');
    description.className = 'tool-description';
    description.textContent = tool.description;
    copy.append(name, description);

    const status = document.createElement('span');
    status.className = 'status';
    status.textContent = tool.status === 'concept' ? 'Concepto' : tool.version;

    item.append(icon, copy, status);
    list.appendChild(item);
  });
}

function updateSelection() {
  const items = list.querySelectorAll('.tool');
  items.forEach((item, index) => {
    item.setAttribute('aria-selected', String(index === selectedIndex));
  });
}

async function runTool(tool) {
  if (!tool) return;

  message.textContent = '';
  lastMessage = '';

  if (tool.id === 'luma.action.copy-color') {
    appWindow.hide();
  }

  if (tool.runtime?.type === 'overlay') {
    lastMessage = `Preparando ${tool.name}...`;
    message.textContent = lastMessage;
  }

  try {
    const result = await invoke('run_tool', { toolId: tool.id });
    lastMessage = result.message || '';
    message.textContent = lastMessage;

    if (tool.runtime?.type === 'overlay') {
      appWindow.hide();
    }
  } catch (error) {
    lastMessage = String(error || 'No se pudo abrir la Action.');
    message.textContent = lastMessage;
  }
}

async function runSelected() {
  const tool = filteredTools()[selectedIndex];
  await runTool(tool);
}

async function boot() {
  tools = await invoke('list_tools');
  render();
  searchInput.focus();
}

window.__workspaceBackSetTools = (nextTools) => {
  tools = nextTools;
  selectedIndex = 0;
  searchInput.value = '';
  message.textContent = lastMessage;
  render();
  requestAnimationFrame(() => searchInput.focus());
  return true;
};

searchInput.addEventListener('input', () => {
  selectedIndex = 0;
  message.textContent = '';
  render();
});

installButton.addEventListener('click', async () => {
  const installed = await invoke('install_tool');
  if (installed) {
    tools = await invoke('list_tools');
    message.textContent = `${installed.name} instalada.`;
    render();
  }
});

window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    event.preventDefault();
    event.stopPropagation();
    appWindow.hide();
    invoke('hide_launcher');
    return;
  }
}, true);

window.addEventListener('keydown', (event) => {
  const visibleTools = filteredTools();

  if (event.key === 'ArrowDown') {
    selectedIndex = Math.min(selectedIndex + 1, visibleTools.length - 1);
    updateSelection();
  }

  if (event.key === 'ArrowUp') {
    selectedIndex = Math.max(selectedIndex - 1, 0);
    updateSelection();
  }

  if (event.key === 'Enter') {
    runSelected();
  }
});

listen('tools-updated', (event) => {
  window.__workspaceBackSetTools(event.payload);
});

boot();
