// Academ-IA Node — Frontend JavaScript
// Communicates with Tauri backend via invoke()

const { invoke } = window.__TAURI__?.core || window.__TAURI__ || {
  invoke: async (cmd, args) => {
    console.log('[DEV] invoke:', cmd, args);
    // Mock responses for browser testing
    const mocks = {
      get_app_state: {
        ollama: { status: 'stopped', pid: null, port: 11434, version: null, error: null },
        tunnel: { status: 'disconnected', worker_id: null, worker_name: null, organization_id: null, platform_url: 'https://acad-ia-78kiixzl.manus.space', last_heartbeat: null, error: null },
        config: { worker_id: 'dev-worker-id', worker_name: '', organization_id: '', api_key: '', ollama_port: 11434, ollama_auto_start: true, app_autostart: false, minimize_to_tray: true, platform_url_override: '' },
        app_version: '1.0.0'
      },
      get_ollama_status: { status: 'stopped', pid: null, port: 11434, version: null, error: null },
      list_models: [],
      check_for_updates: { available: false, current_version: '1.0.0', latest_version: null, release_notes: null, download_url: null }
    };
    return mocks[cmd] || null;
  }
};

// ============================================
// State
// ============================================
let appState = null;
let models = [];

// ============================================
// Init
// ============================================
document.addEventListener('DOMContentLoaded', async () => {
  await loadState();
  setInterval(refreshStatus, 10000); // Refresh every 10s
});

async function loadState() {
  try {
    appState = await invoke('get_app_state');
    updateUI();
    await refreshModels();
    addLog('info', 'Application démarrée');
  } catch (e) {
    addLog('error', `Erreur au démarrage: ${e}`);
  }
}

// ============================================
// UI Updates
// ============================================
function updateUI() {
  if (!appState) return;

  const { ollama, tunnel, config, app_version } = appState;

  // Stats
  document.getElementById('stat-ollama').textContent = statusLabel(ollama.status);
  document.getElementById('stat-tunnel').textContent = tunnelLabel(tunnel.status);
  document.getElementById('stat-version').textContent = app_version || '1.0.0';
  document.getElementById('stat-models').textContent = models.length;

  // Status dots
  setDot('stat-ollama-dot', ollama.status === 'running' ? 'running' : ollama.status === 'error' ? 'error' : '');
  setDot('stat-tunnel-dot', tunnel.status === 'connected' ? 'running' : tunnel.status === 'error' ? 'error' : tunnel.status === 'connecting' ? 'connecting' : '');

  // Global status badge
  const globalDot = document.querySelector('#global-status .status-dot');
  const globalText = document.querySelector('#global-status span');
  if (ollama.status === 'running' && tunnel.status === 'connected') {
    globalDot.className = 'status-dot running';
    globalText.textContent = 'En ligne';
  } else if (ollama.status === 'running') {
    globalDot.className = 'status-dot connecting';
    globalText.textContent = 'Ollama actif';
  } else {
    globalDot.className = 'status-dot';
    globalText.textContent = 'Hors ligne';
  }

  // Buttons
  const btnStart = document.getElementById('btn-start-ollama');
  const btnStop = document.getElementById('btn-stop-ollama');
  if (ollama.status === 'running') {
    btnStart.style.display = 'none';
    btnStop.style.display = 'flex';
  } else {
    btnStart.style.display = 'flex';
    btnStop.style.display = 'none';
  }

  // Tunnel connection visual
  const connCard = document.getElementById('connection-card');
  const connLine = document.getElementById('conn-line');
  const connText = document.getElementById('conn-status-text');
  if (tunnel.status === 'connected') {
    connLine.classList.add('connected');
    connText.textContent = `Connecté — ${tunnel.worker_name || tunnel.worker_id || ''}`;
    connText.style.color = 'var(--accent-green)';
  } else {
    connLine.classList.remove('connected');
    connText.textContent = tunnelLabel(tunnel.status);
    connText.style.color = '';
  }

  // Settings
  if (config) {
    document.getElementById('setting-ollama-autostart').checked = config.ollama_auto_start;
    document.getElementById('setting-app-autostart').checked = config.app_autostart;
    document.getElementById('setting-minimize-tray').checked = config.minimize_to_tray;
    document.getElementById('org-id').value = config.organization_id || '';
    document.getElementById('api-key').value = config.api_key ? '••••••••' : '';
    document.getElementById('worker-name').value = config.worker_name || '';
    document.getElementById('ollama-port').value = config.ollama_port || 11434;
    document.getElementById('about-worker-id').textContent = config.worker_id || '—';
  }

  document.getElementById('about-version').textContent = app_version || '1.0.0';
  document.getElementById('about-platform').textContent = navigator.platform || '—';
}

function setDot(id, cls) {
  const el = document.getElementById(id);
  if (el) el.className = 'stat-status' + (cls ? ' ' + cls : '');
}

function statusLabel(status) {
  const labels = { stopped: 'Arrêté', starting: 'Démarrage...', running: 'En cours', error: 'Erreur' };
  return labels[status] || status;
}

function tunnelLabel(status) {
  const labels = { disconnected: 'Déconnecté', connecting: 'Connexion...', connected: 'Connecté', error: 'Erreur' };
  return labels[status] || status;
}

// ============================================
// Navigation
// ============================================
function showPage(name) {
  document.querySelectorAll('.page').forEach(p => p.classList.remove('active'));
  document.querySelectorAll('.nav-item').forEach(n => n.classList.remove('active'));
  document.getElementById('page-' + name)?.classList.add('active');
  document.querySelector(`[data-page="${name}"]`)?.classList.add('active');

  if (name === 'ollama') refreshModels();
}

// ============================================
// Ollama Actions
// ============================================
async function startOllama() {
  addLog('info', 'Démarrage de Ollama...');
  try {
    const state = await invoke('start_ollama');
    appState.ollama = state;
    updateUI();
    addLog('success', `Ollama démarré${state.version ? ' — v' + state.version : ''}`);
  } catch (e) {
    addLog('error', `Impossible de démarrer Ollama: ${e}`);
  }
}

async function stopOllama() {
  try {
    const state = await invoke('stop_ollama');
    appState.ollama = state;
    updateUI();
    addLog('info', 'Ollama arrêté');
  } catch (e) {
    addLog('error', `Erreur: ${e}`);
  }
}

async function refreshModels() {
  try {
    models = await invoke('list_models');
    document.getElementById('stat-models').textContent = models.length;
    renderModels();
  } catch (e) {
    // Ollama might not be running
    models = [];
    renderModels();
  }
}

function renderModels() {
  const list = document.getElementById('models-list');
  if (models.length === 0) {
    list.innerHTML = `
      <div class="empty-state">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.3">
          <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
        </svg>
        <p>Aucun modèle installé</p>
        <p class="empty-hint">Démarrez Ollama puis téléchargez un modèle ci-dessus</p>
      </div>`;
    return;
  }

  list.innerHTML = models.map(m => {
    const sizeGB = (m.size / 1e9).toFixed(1);
    const date = m.modified_at ? new Date(m.modified_at).toLocaleDateString('fr-FR') : '—';
    return `
      <div class="model-item">
        <div class="model-item-info">
          <div class="model-item-name">${m.name}</div>
          <div class="model-item-meta">${sizeGB} GB · ${m.parameter_size || '—'} · ${m.quantization_level || '—'} · Modifié le ${date}</div>
        </div>
        <div class="model-item-actions">
          <button class="btn-icon" title="Supprimer" onclick="deleteModel('${m.name}')">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6"/><path d="M10 11v6M14 11v6"/><path d="M9 6V4h6v2"/>
            </svg>
          </button>
        </div>
      </div>`;
  }).join('');
}

function setModelSearch(name) {
  document.getElementById('model-search').value = name;
  showPage('ollama');
}

async function pullModel() {
  const modelName = document.getElementById('model-search').value.trim();
  if (!modelName) {
    addLog('warning', 'Entrez un nom de modèle');
    return;
  }

  const progress = document.getElementById('pull-progress');
  const bar = document.getElementById('pull-bar');
  const pct = document.getElementById('pull-percent');
  const pName = document.getElementById('pull-model-name');

  progress.style.display = 'block';
  pName.textContent = `Téléchargement de ${modelName}...`;
  bar.style.width = '0%';
  pct.textContent = '0%';

  // Animate progress bar (Tauri doesn't stream progress easily without events)
  let fakeProgress = 0;
  const interval = setInterval(() => {
    fakeProgress = Math.min(fakeProgress + Math.random() * 3, 90);
    bar.style.width = fakeProgress + '%';
    pct.textContent = Math.round(fakeProgress) + '%';
  }, 500);

  addLog('info', `Téléchargement de ${modelName}...`);

  try {
    const result = await invoke('pull_model', { model: modelName });
    clearInterval(interval);
    bar.style.width = '100%';
    pct.textContent = '100%';
    addLog('success', result);
    setTimeout(() => { progress.style.display = 'none'; }, 2000);
    await refreshModels();
  } catch (e) {
    clearInterval(interval);
    progress.style.display = 'none';
    addLog('error', `Erreur: ${e}`);
  }
}

async function deleteModel(name) {
  if (!confirm(`Supprimer le modèle "${name}" ?`)) return;
  try {
    await invoke('delete_model', { model: name });
    addLog('info', `Modèle ${name} supprimé`);
    await refreshModels();
  } catch (e) {
    addLog('error', `Erreur: ${e}`);
  }
}

// ============================================
// Tunnel Actions
// ============================================
async function connectTunnel() {
  showPage('tunnel');
}

async function saveAndConnect() {
  const orgId = document.getElementById('org-id').value.trim();
  const apiKey = document.getElementById('api-key').value.trim();
  const workerName = document.getElementById('worker-name').value.trim();
  const ollamaPort = parseInt(document.getElementById('ollama-port').value) || 11434;

  if (!orgId || !apiKey) {
    addLog('warning', 'ID Organisation et Clé API sont obligatoires');
    return;
  }

  // Save config
  const newConfig = {
    ...appState.config,
    organization_id: orgId,
    api_key: apiKey !== '••••••••' ? apiKey : appState.config.api_key,
    worker_name: workerName,
    ollama_port: ollamaPort,
  };

  try {
    await invoke('save_config', { config: newConfig });
    appState.config = newConfig;
    addLog('info', 'Configuration sauvegardée');

    // Connect
    addLog('info', 'Connexion à la plateforme...');
    const tunnelState = await invoke('start_tunnel');
    appState.tunnel = tunnelState;
    updateUI();
    addLog('success', 'Connecté à Academ-IA !');
  } catch (e) {
    addLog('error', `Connexion échouée: ${e}`);
  }
}

async function disconnectTunnel() {
  try {
    const tunnelState = await invoke('stop_tunnel');
    appState.tunnel = tunnelState;
    updateUI();
    addLog('info', 'Déconnecté de la plateforme');
  } catch (e) {
    addLog('error', `Erreur: ${e}`);
  }
}

// ============================================
// Settings
// ============================================
async function saveSetting(key, value) {
  if (!appState?.config) return;
  const keyMap = { ollamaAutoStart: 'ollama_auto_start', appAutostart: 'app_autostart', minimizeToTray: 'minimize_to_tray' };
  const configKey = keyMap[key];
  if (!configKey) return;

  appState.config[configKey] = value;
  try {
    await invoke('save_config', { config: appState.config });
  } catch (e) {
    addLog('error', `Erreur de sauvegarde: ${e}`);
  }
}

// ============================================
// Updates
// ============================================
async function checkUpdates() {
  addLog('info', 'Vérification des mises à jour...');
  try {
    const info = await invoke('check_for_updates');
    if (info.available) {
      addLog('success', `Mise à jour disponible: v${info.latest_version}`);
      if (confirm(`Mise à jour disponible: v${info.latest_version}\n\nVoulez-vous télécharger la mise à jour ?`)) {
        if (info.download_url) window.open(info.download_url, '_blank');
      }
    } else {
      addLog('info', `Vous utilisez la dernière version (v${info.current_version})`);
    }
  } catch (e) {
    addLog('error', `Erreur: ${e}`);
  }
}

// ============================================
// Refresh Status
// ============================================
async function refreshStatus() {
  try {
    const ollama = await invoke('get_ollama_status');
    if (appState) {
      appState.ollama = ollama;
      updateUI();
    }
  } catch (e) {
    // Silent fail
  }
}

// ============================================
// Activity Log
// ============================================
function addLog(type, message) {
  const log = document.getElementById('activity-log');
  const now = new Date().toLocaleTimeString('fr-FR');
  const entry = document.createElement('div');
  entry.className = `log-entry ${type}`;
  entry.innerHTML = `<span class="log-time">${now}</span><span class="log-msg">${message}</span>`;
  log.appendChild(entry);
  log.scrollTop = log.scrollHeight;

  // Keep only last 50 entries
  while (log.children.length > 50) {
    log.removeChild(log.firstChild);
  }
}
