// BFG Repo Cleaner Native GUI Logic
document.addEventListener('DOMContentLoaded', () => {
  // DOM Elements
  const repoPathInput = document.getElementById('repo-path-input');
  const btnBrowse = document.getElementById('btn-browse');
  const repoStatus = document.getElementById('repo-status');
  const repoStatusText = document.getElementById('repo-status-text');

  const toggleMaxSize = document.getElementById('toggle-max-size');
  const groupMaxSize = document.getElementById('group-max-size');
  const maxSizeInput = document.getElementById('max-size-input');
  const pillButtons = document.querySelectorAll('.pill');

  const deleteFilesInput = document.getElementById('delete-files-input');
  const deleteFoldersInput = document.getElementById('delete-folders-input');
  const regexInput = document.getElementById('regex-input');
  const presetButtons = document.querySelectorAll('.btn-preset');

  const checkProtectHead = document.getElementById('check-protect-head');
  const protectRefsInput = document.getElementById('protect-refs-input');

  const btnExecute = document.getElementById('btn-execute');
  const confirmModal = document.getElementById('confirm-modal');
  const modalRepoPath = document.getElementById('modal-repo-path');
  const btnCancelModal = document.getElementById('btn-cancel-modal');
  const btnConfirmExecute = document.getElementById('btn-confirm-execute');

  const loadingOverlay = document.getElementById('loading-overlay');
  const resultsSection = document.getElementById('results-section');
  const btnCloseResults = document.getElementById('btn-close-results');

  const resBlobsScanned = document.getElementById('res-blobs-scanned');
  const resBlobsRemoved = document.getElementById('res-blobs-removed');
  const resSecretsRedacted = document.getElementById('res-secrets-redacted');
  const resCommitsRewritten = document.getElementById('res-commits-rewritten');
  const resTreesRewritten = document.getElementById('res-trees-rewritten');
  const resRefsUpdated = document.getElementById('res-refs-updated');
  const resExecTime = document.getElementById('res-exec-time');

  let isRepoValid = false;

  // Helper for Tauri invoke
  async function invokeCommand(cmd, args = {}) {
    if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === 'function') {
      return await window.__TAURI__.core.invoke(cmd, args);
    } else if (window.__TAURI__ && typeof window.__TAURI__.invoke === 'function') {
      return await window.__TAURI__.invoke(cmd, args);
    } else if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function') {
      return await window.__TAURI_INTERNALS__.invoke(cmd, args);
    } else {
      console.warn(`[Tauri API] Not detected in browser context. Command '${cmd}' skipped.`);
      return null;
    }
  }

  // Repository Folder Browse
  btnBrowse.addEventListener('click', async () => {
    try {
      const folderPath = await invokeCommand('select_repository_folder');
      if (folderPath) {
        repoPathInput.value = folderPath;
        validateRepoPath(folderPath);
      }
    } catch (err) {
      console.error('Folder selection error:', err);
    }
  });

  // Repo Path Input Event
  repoPathInput.addEventListener('input', (e) => {
    validateRepoPath(e.target.value.trim());
  });

  // Validate Repository
  async function validateRepoPath(path) {
    if (!path) {
      updateRepoStatus(false, 'Select a repository directory to analyze.');
      return;
    }

    try {
      const res = await invokeCommand('validate_repository', { repoPath: path });
      if (res && res.valid) {
        let msg = res.message;
        if (res.head_ref) {
          msg += ` (Branch: ${res.head_ref})`;
        }
        updateRepoStatus(true, msg);
      } else {
        updateRepoStatus(false, res ? res.message : 'Invalid repository.');
      }
    } catch (err) {
      updateRepoStatus(false, `Error checking repository: ${err}`);
    }
  }

  function updateRepoStatus(valid, message) {
    isRepoValid = valid;
    repoStatus.className = 'repo-status ' + (valid ? 'status-valid' : 'status-error');
    repoStatus.querySelector('.status-icon').textContent = valid ? '✔' : '✖';
    repoStatusText.textContent = message;
    btnExecute.disabled = !valid;
  }

  // Max Size Toggle & Presets
  toggleMaxSize.addEventListener('change', (e) => {
    maxSizeInput.disabled = !e.target.checked;
    pillButtons.forEach(p => p.style.opacity = e.target.checked ? '1' : '0.4');
  });

  pillButtons.forEach(pill => {
    pill.addEventListener('click', () => {
      pillButtons.forEach(p => p.classList.remove('active'));
      pill.classList.add('active');
      maxSizeInput.value = pill.dataset.size;
      toggleMaxSize.checked = true;
      maxSizeInput.disabled = false;
    });
  });

  // Regex Presets
  presetButtons.forEach(btn => {
    btn.addEventListener('click', () => {
      regexInput.value = btn.dataset.regex;
      regexInput.focus();
    });
  });

  // Protect HEAD Sync
  checkProtectHead.addEventListener('change', (e) => {
    let currentRefs = protectRefsInput.value.split(',').map(s => s.trim()).filter(Boolean);
    if (e.target.checked) {
      if (!currentRefs.includes('HEAD')) {
        currentRefs.unshift('HEAD');
      }
    } else {
      currentRefs = currentRefs.filter(r => r !== 'HEAD');
    }
    protectRefsInput.value = currentRefs.join(', ');
  });

  // Execute Action -> Open Confirmation Modal
  btnExecute.addEventListener('click', () => {
    if (!isRepoValid) return;
    modalRepoPath.textContent = repoPathInput.value;
    confirmModal.classList.remove('hidden');
  });

  btnCancelModal.addEventListener('click', () => {
    confirmModal.classList.add('hidden');
  });

  // Confirm Execution
  btnConfirmExecute.addEventListener('click', async () => {
    confirmModal.classList.add('hidden');
    loadingOverlay.classList.remove('hidden');

    const maxSizeBytes = toggleMaxSize.checked && maxSizeInput.value
      ? parseInt(maxSizeInput.value, 10) * 1024 * 1024
      : null;

    const protectRefs = protectRefsInput.value
      .split(',')
      .map(s => s.trim())
      .filter(Boolean);

    const payload = {
      repo_path: repoPathInput.value.trim(),
      max_file_size_bytes: maxSizeBytes,
      delete_files: deleteFilesInput.value.trim() || null,
      delete_folders: deleteFoldersInput.value.trim() || null,
      regex_pattern: regexInput.value.trim() || null,
      protect_blobs_from: protectRefs.length > 0 ? protectRefs : null,
      no_blob_protection: !checkProtectHead.checked && protectRefs.length === 0,
    };

    try {
      const summary = await invokeCommand('execute_cleaner', { payload });
      loadingOverlay.classList.add('hidden');

      if (summary) {
        displayResults(summary);
      }
    } catch (err) {
      loadingOverlay.classList.add('hidden');
      alert(`Execution Error / Erro de Execução:\n${err}`);
    }
  });

  // Display Results
  function displayResults(summary) {
    resBlobsScanned.textContent = summary.total_blobs_scanned.toLocaleString();
    resBlobsRemoved.textContent = summary.blobs_removed.toLocaleString();
    resSecretsRedacted.textContent = summary.secrets_redacted.toLocaleString();
    resCommitsRewritten.textContent = summary.total_commits_rewritten.toLocaleString();
    resTreesRewritten.textContent = summary.total_trees_rewritten.toLocaleString();
    resRefsUpdated.textContent = summary.total_refs_updated.toLocaleString();
    resExecTime.textContent = `${summary.execution_time_ms} ms`;

    resultsSection.classList.remove('hidden');
    resultsSection.scrollIntoView({ behavior: 'smooth' });
  }

  btnCloseResults.addEventListener('click', () => {
    resultsSection.classList.add('hidden');
  });
});
