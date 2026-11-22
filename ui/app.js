/**
 * Daqiq License Generator Studio v1.1.0
 * Modern Vanilla JS Frontend for Tauri v2 Native Windows Desktop
 */

// Helper to check if running inside Tauri
const isTauri = () => typeof window.__TAURI_INTERNALS__ !== 'undefined' || typeof window.__TAURI__ !== 'undefined';

// Safe Tauri invoke wrapper with web fallback
async function invokeTauri(cmd, args = {}) {
  if (isTauri()) {
    try {
      const invoke = window.__TAURI__.core.invoke;
      return await invoke(cmd, args);
    } catch (e) {
      if (typeof window.__TAURI_INTERNALS__ !== 'undefined' && window.__TAURI_INTERNALS__.invoke) {
        return await window.__TAURI_INTERNALS__.invoke(cmd, args);
      }
      throw e;
    }
  } else {
    console.warn(`[MockTauri] ${cmd}`, args);
    return mockInvoke(cmd, args);
  }
}

// Global UI State
const state = {
  activeTab: 'tab-generator',
  theme: localStorage.getItem('daqiq_theme') || 'dark',
  vaultUnlocked: false,
  activeKeyId: 'daqiq-prod-v1',
  activePublicKey: 'tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=',
  currentGeneratedJson: null,
};

// Toast Notifications
function showToast(message, type = 'success') {
  const container = document.getElementById('toast-container');
  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  toast.textContent = message;
  container.appendChild(toast);
  setTimeout(() => {
    toast.style.opacity = '0';
    toast.style.transform = 'translateY(10px)';
    setTimeout(() => toast.remove(), 300);
  }, 3500);
}

// Generate Random License ID (e.g. DQL-1405-XXXX)
function generateRandomLicenseId() {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
  let rand = '';
  for (let i = 0; i < 4; i++) {
    rand += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return `DQL-1405-${rand}`;
}

// Jalali Date Helpers
function getTodayJalaliString() {
  // Approximate today's Jalali string based on current date
  const now = new Date();
  const gy = now.getFullYear();
  const gm = now.getMonth() + 1;
  const gd = now.getDate();
  
  // Standard conversion
  const g_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
  const j_days = [31, 31, 31, 31, 31, 31, 30, 30, 30, 30, 30, 29];
  let gy2 = gy - 1600;
  let gm2 = gm - 1;
  let j_day_no = 365 * gy2 + Math.floor((gy2 + 3) / 4) - Math.floor((gy2 + 99) / 100) + Math.floor((gy2 + 399) / 400) + gd - 1 - 79;
  for (let i = 0; i < gm2; ++i) j_day_no += g_days[i];
  if (gm2 > 1 && ((gy2 % 4 === 0 && gy2 % 100 !== 0) || (gy2 % 400 === 0))) j_day_no++;
  let j_np = Math.floor(j_day_no / 12053);
  j_day_no %= 12053;
  let jy = 979 + 33 * j_np + 4 * Math.floor(j_day_no / 1461);
  j_day_no %= 1461;
  if (j_day_no >= 366) {
    j_day_no--;
    jy += Math.floor(j_day_no / 365);
    j_day_no %= 365;
  }
  let jm = 0;
  for (let i = 0; i < 11; i++) {
    if (j_day_no < j_days[i]) break;
    j_day_no -= j_days[i];
    jm = i + 1;
  }
  const jd = j_day_no + 1;
  const pad = (n) => String(n).padStart(2, '0');
  return `${jy}/${pad(jm + 1)}/${pad(jd)}`;
}

function addJalaliMonths(jalaliStr, monthsToAdd) {
  const parts = jalaliStr.split('/').map(Number);
  if (parts.length !== 3) return jalaliStr;
  let [y, m, d] = parts;
  m += monthsToAdd;
  while (m > 12) {
    y += 1;
    m -= 12;
  }
  const pad = (n) => String(n).padStart(2, '0');
  return `${y}/${pad(m)}/${pad(d)}`;
}

// Initialize Application
document.addEventListener('DOMContentLoaded', async () => {
  setupNavigation();
  setupTheme();
  setupFormDefaults();
  setupHwidValidation();
  setupGeneratorActions();
  setupLedger();
  setupInspector();
  setupVault();

  await refreshVaultStatus();
});

// Setup Navigation Tabs
function setupNavigation() {
  const tabs = document.querySelectorAll('.nav-tab');
  tabs.forEach((tab) => {
    tab.addEventListener('click', () => {
      tabs.forEach((t) => t.classList.remove('active'));
      document.querySelectorAll('.tab-pane').forEach((p) => p.classList.remove('active'));
      
      tab.classList.add('active');
      const targetId = tab.dataset.tab;
      document.getElementById(targetId).classList.add('active');
      state.activeTab = targetId;

      if (targetId === 'tab-ledger') {
        loadLedgerRecords();
      }
    });
  });
}

// Setup Theme Toggle
function setupTheme() {
  const btn = document.getElementById('btn-theme-toggle');
  const sunIcon = document.getElementById('icon-sun');
  const moonIcon = document.getElementById('icon-moon');

  const applyTheme = (theme) => {
    if (theme === 'light') {
      document.body.classList.remove('theme-dark');
      document.body.classList.add('theme-light');
      sunIcon.classList.add('hidden');
      moonIcon.classList.remove('hidden');
    } else {
      document.body.classList.remove('theme-light');
      document.body.classList.add('theme-dark');
      moonIcon.classList.add('hidden');
      sunIcon.classList.remove('hidden');
    }
    localStorage.setItem('daqiq_theme', theme);
  };

  applyTheme(state.theme);

  btn.addEventListener('click', () => {
    state.theme = state.theme === 'dark' ? 'light' : 'dark';
    applyTheme(state.theme);
  });
}

// Setup Form Defaults
function setupFormDefaults() {
  const todayJ = getTodayJalaliString();
  const nextYearJ = addJalaliMonths(todayJ, 12);

  document.getElementById('lic-id').value = generateRandomLicenseId();
  document.getElementById('date-issued').value = todayJ;
  document.getElementById('date-not-before').value = todayJ;
  document.getElementById('date-expires').value = nextYearJ;
  document.getElementById('sim-date').value = todayJ;

  document.getElementById('btn-regen-lic-id').addEventListener('click', () => {
    document.getElementById('lic-id').value = generateRandomLicenseId();
  });

  // Radio button for license type toggle
  const radios = document.querySelectorAll('input[name="lic-type"]');
  radios.forEach((r) => {
    r.addEventListener('change', () => {
      const isSub = document.querySelector('input[name="lic-type"]:checked').value === 'subscription';
      const groupExp = document.getElementById('group-expires');
      const quickDur = document.getElementById('quick-durations');
      if (isSub) {
        groupExp.classList.remove('hidden');
        quickDur.classList.remove('hidden');
      } else {
        groupExp.classList.add('hidden');
        quickDur.classList.add('hidden');
      }
    });
  });

  // Quick duration buttons
  document.querySelectorAll('#quick-durations button').forEach((btn) => {
    btn.addEventListener('click', () => {
      const months = parseInt(btn.dataset.months, 10);
      const start = document.getElementById('date-not-before').value || todayJ;
      document.getElementById('date-expires').value = addJalaliMonths(start, months);
    });
  });

  // Feature Select All / Deselect All
  document.getElementById('btn-select-all-features').addEventListener('click', () => {
    document.querySelectorAll('#feature-grid input[type="checkbox"]').forEach((c) => (c.checked = true));
  });
  document.getElementById('btn-deselect-all-features').addEventListener('click', () => {
    document.querySelectorAll('#feature-grid input[type="checkbox"]').forEach((c) => (c.checked = false));
  });
}

// Live HWID Validation
function setupHwidValidation() {
  const txtHwids = document.getElementById('txt-hwids');
  const badgeCount = document.getElementById('badge-hwid-count');
  const msgBox = document.getElementById('hwid-status-message');

  const updateHwidState = async () => {
    const text = txtHwids.value;
    if (!text.trim()) {
      badgeCount.textContent = '۰ دستگاه وارد شده';
      badgeCount.className = 'badge badge-neutral';
      msgBox.classList.add('hidden');
      return;
    }

    try {
      const res = await invokeTauri('clean_and_parse_hwids', { text });
      badgeCount.textContent = `${res.total_valid} شناسه معتبر`;
      badgeCount.className = res.total_valid > 0 ? 'badge badge-success' : 'badge badge-neutral';

      if (res.invalid_entries.length > 0) {
        msgBox.textContent = `توجه: ${res.invalid_entries.length} ورودی نامعتبر نادیده گرفته شد (فرمت صحیح: fp_v1:<64 نویسه هگز>).`;
        msgBox.classList.remove('hidden');
        msgBox.className = 'validation-msg warning';
      } else {
        msgBox.classList.add('hidden');
      }

      // Auto update max devices if count exceeds
      const maxDevInput = document.getElementById('max-devices');
      if (res.total_valid > parseInt(maxDevInput.value, 10)) {
        maxDevInput.value = res.total_valid;
      }
    } catch (e) {
      console.error(e);
    }
  };

  txtHwids.addEventListener('input', updateHwidState);

  // Import .daqreq
  document.getElementById('btn-import-daqreq').addEventListener('click', () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.daqreq,.json';
    input.onchange = async (e) => {
      const file = e.target.files[0];
      if (!file) return;
      const content = await file.text();
      try {
        const req = await invokeTauri('parse_daqreq_file', { jsonStr: content });
        const existing = txtHwids.value.trim();
        txtHwids.value = existing ? `${existing}\n${req.device_fingerprint}` : req.device_fingerprint;
        updateHwidState();
        showToast('شناسه سخت‌افزاری از فایل .daqreq با موفقیت وارد شد');
      } catch (err) {
        showToast(err.toString(), 'error');
      }
    };
    input.click();
  });
}

// Generator Tab Actions
function setupGeneratorActions() {
  const btnGenerate = document.getElementById('btn-generate-license');
  const txtOutput = document.getElementById('txt-output-json');
  const outputBadge = document.getElementById('output-badge');
  const outputDigest = document.getElementById('output-digest');
  const btnCopy = document.getElementById('btn-copy-license');
  const btnSave = document.getElementById('btn-save-license');

  btnGenerate.addEventListener('click', async () => {
    const custName = document.getElementById('cust-name').value.trim();
    if (!custName) {
      showToast('لطفاً نام مشتری یا فروشگاه را وارد نمایید', 'error');
      document.getElementById('cust-name').focus();
      return;
    }

    const hwidText = document.getElementById('txt-hwids').value.trim();
    if (!hwidText) {
      showToast('لطفاً حداقل یک شناسه سخت‌افزاری وارد نمایید', 'error');
      document.getElementById('txt-hwids').focus();
      return;
    }

    const parsedHwids = await invokeTauri('clean_and_parse_hwids', { text: hwidText });
    if (parsedHwids.total_valid === 0) {
      showToast('هیچ شناسه سخت‌افزاری معتبری یافت نشد', 'error');
      return;
    }

    const licType = document.querySelector('input[name="lic-type"]:checked').value;
    const dateIssuedJ = document.getElementById('date-issued').value;
    const dateNotBeforeJ = document.getElementById('date-not-before').value;
    const dateExpiresJ = document.getElementById('date-expires').value;

    let issuedAtIso = '', notBeforeIso = '', expiresAtIso = null;
    try {
      issuedAtIso = await invokeTauri('convert_jalali_to_gregorian', { jalaliStr: dateIssuedJ, endOfDay: false });
      notBeforeIso = await invokeTauri('convert_jalali_to_gregorian', { jalaliStr: dateNotBeforeJ, endOfDay: false });
      if (licType === 'subscription') {
        expiresAtIso = await invokeTauri('convert_jalali_to_gregorian', { jalaliStr: dateExpiresJ, endOfDay: true });
      }
    } catch (err) {
      showToast(`خطا در تاریخ‌های شمسی: ${err}`, 'error');
      return;
    }

    // Features
    const features = {};
    document.querySelectorAll('#feature-grid input[type="checkbox"]').forEach((c) => {
      features[c.value] = c.checked;
    });

    const spec = {
      license_id: document.getElementById('lic-id').value.trim(),
      key_id: state.activeKeyId,
      customer_name: custName,
      customer_organization: document.getElementById('cust-org').value.trim() || null,
      customer_contact: document.getElementById('cust-contact').value.trim() || null,
      license_type: licType,
      issued_at: issuedAtIso,
      not_before: notBeforeIso,
      expires_at: expiresAtIso,
      max_devices: parseInt(document.getElementById('max-devices').value, 10),
      device_fingerprints: parsedHwids.valid_hwids,
      features: features,
      min_app_version: document.getElementById('min-app-ver').value.trim() || null,
      max_app_version: document.getElementById('max-app-ver').value.trim() || null,
    };

    btnGenerate.disabled = true;
    btnGenerate.innerHTML = '<span>در حال پردازش و امضای رمزنگاری...</span>';

    try {
      const res = await invokeTauri('generate_license', { spec });
      state.currentGeneratedJson = res.json_text;
      txtOutput.value = res.json_text;
      outputDigest.textContent = res.sha256_digest;
      outputBadge.textContent = 'امضا شده و معتبر (Ed25519)';
      outputBadge.className = 'badge badge-success';
      btnCopy.disabled = false;
      btnSave.disabled = false;

      showToast(`لایسنس ${spec.license_id} با موفقیت صادر و ثبت شد`);
    } catch (err) {
      showToast(err.toString(), 'error');
    } finally {
      btnGenerate.disabled = false;
      btnGenerate.innerHTML = `
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
          <polyline points="9 12 11 14 15 10"/>
        </svg>
        <span>تولید و امضای لایسنس دیجیتال (.daqlic)</span>
      `;
    }
  });

  // Copy License
  btnCopy.addEventListener('click', () => {
    if (!state.currentGeneratedJson) return;
    navigator.clipboard.writeText(state.currentGeneratedJson);
    showToast('محتوای لایسنس در حافظه کلیپ‌بورد کپی شد');
  });

  // Save .daqlic File
  btnSave.addEventListener('click', () => {
    if (!state.currentGeneratedJson) return;
    const blob = new Blob([state.currentGeneratedJson], { type: 'application/json;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    const licId = document.getElementById('lic-id').value.trim();
    a.href = url;
    a.download = `${licId}.daqlic`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    showToast(`فایل ${licId}.daqlic ذخیره گردید`);
  });
}

// Sales Ledger Tab
async function loadLedgerRecords(query = '') {
  const tbody = document.getElementById('tbody-ledger');
  try {
    const records = query
      ? await invokeTauri('search_registry_records', { query })
      : await invokeTauri('list_registry_records', { limit: 100, offset: 0 });

    if (records.length === 0) {
      tbody.innerHTML = '<tr><td colspan="7" class="text-center py-4 text-muted">هیچ لایسنسی یافت نشد.</td></tr>';
      return;
    }

    tbody.innerHTML = records
      .map(
        (r) => `
        <tr>
          <td class="code-font fw-bold" dir="ltr">${escapeHtml(r.license_id)}</td>
          <td>${escapeHtml(r.customer_name)}</td>
          <td><span class="badge ${r.license_type === 'perpetual' ? 'badge-info' : 'badge-neutral'}">${r.license_type === 'perpetual' ? 'مادام‌العمر' : 'اشتراکی'}</span></td>
          <td class="code-font" dir="ltr">${escapeHtml(r.issued_at.slice(0, 10))}</td>
          <td class="code-font" dir="ltr">${r.expires_at ? escapeHtml(r.expires_at.slice(0, 10)) : 'دائمی'}</td>
          <td>${r.max_devices}</td>
          <td>
            <button class="btn btn-xs btn-outline btn-ledger-inspect" data-json="${escapeAttr(r.full_envelope_json)}">
              🔍 بازرسی
            </button>
          </td>
        </tr>
      `
      )
      .join('');

    // Attach inspect buttons
    tbody.querySelectorAll('.btn-ledger-inspect').forEach((b) => {
      b.addEventListener('click', () => {
        const json = b.dataset.json;
        document.getElementById('txt-inspect-json').value = json;
        document.querySelector('.nav-tab[data-tab="tab-inspector"]').click();
        document.getElementById('btn-verify-license').click();
      });
    });
  } catch (err) {
    tbody.innerHTML = `<tr><td colspan="7" class="text-center py-4 text-danger">${escapeHtml(err.toString())}</td></tr>`;
  }
}

function setupLedger() {
  const searchInput = document.getElementById('txt-ledger-search');
  let debounceTimer;
  searchInput.addEventListener('input', () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      loadLedgerRecords(searchInput.value.trim());
    }, 250);
  });

  document.getElementById('btn-export-csv').addEventListener('click', async () => {
    try {
      const csv = await invokeTauri('export_registry_csv');
      const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `daqiq-licenses-${Date.now()}.csv`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      showToast('خروجی اکسل / CSV با موفقیت صادر شد');
    } catch (err) {
      showToast(err.toString(), 'error');
    }
  });
}

// Inspector & Simulator Tab
function setupInspector() {
  const btnVerify = document.getElementById('btn-verify-license');
  const btnPicker = document.getElementById('btn-inspect-file-picker');
  const txtJson = document.getElementById('txt-inspect-json');
  const resBox = document.getElementById('inspect-result-box');
  const statusPill = document.getElementById('inspect-status-pill');
  const msgText = document.getElementById('inspect-message');
  const digestBox = document.getElementById('inspect-digest');

  btnPicker.addEventListener('click', () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.daqlic,.json';
    input.onchange = async (e) => {
      const file = e.target.files[0];
      if (!file) return;
      txtJson.value = await file.text();
      btnVerify.click();
    };
    input.click();
  });

  btnVerify.addEventListener('click', async () => {
    const text = txtJson.value.trim();
    if (!text) {
      showToast('لطفاً محتوای لایسنس را قرار دهید', 'error');
      return;
    }

    try {
      const res = await invokeTauri('verify_license', {
        jsonText: text,
        publicKeyB64: state.activePublicKey,
      });

      resBox.classList.remove('hidden');
      if (res.is_valid) {
        statusPill.className = 'status-pill valid';
        statusPill.textContent = 'امضای معتبر و رسمی (Ed25519 Verified)';
      } else {
        statusPill.className = 'status-pill locked';
        statusPill.textContent = `نامعتبر: ${res.status}`;
      }
      msgText.textContent = res.message;
      digestBox.textContent = `هش پیام: ${res.sha256_digest}`;
    } catch (err) {
      resBox.classList.remove('hidden');
      statusPill.className = 'status-pill locked';
      statusPill.textContent = 'خطا در ساختار لایسنس';
      msgText.textContent = err.toString();
      digestBox.textContent = '';
    }
  });

  // Simulator
  const btnSim = document.getElementById('btn-run-simulation');
  const simHwid = document.getElementById('sim-hwid');
  const simDate = document.getElementById('sim-date');
  const simVer = document.getElementById('sim-version');
  const simBox = document.getElementById('sim-result-box');
  const simPill = document.getElementById('sim-status-pill');
  const simMsg = document.getElementById('sim-message');

  btnSim.addEventListener('click', async () => {
    const text = txtJson.value.trim();
    if (!text) {
      showToast('ابتدا لایسنس مورد نظر را در کادر بررسی قرار دهید', 'error');
      return;
    }
    const hwid = simHwid.value.trim();
    if (!hwid) {
      showToast('لطفاً شناسه سخت‌افزاری فرضی را وارد کنید', 'error');
      simHwid.focus();
      return;
    }

    let isoDate = null;
    if (simDate.value.trim()) {
      try {
        isoDate = await invokeTauri('convert_jalali_to_gregorian', {
          jalaliStr: simDate.value.trim(),
          endOfDay: false,
        });
      } catch (err) {
        showToast(`خطای تاریخ فرضی: ${err}`, 'error');
        return;
      }
    }

    try {
      const sim = await invokeTauri('simulate_validation', {
        jsonText: text,
        currentHwid: hwid,
        currentTimeIso: isoDate,
        currentAppVersion: simVer.value.trim() || '4.9.0',
        publicKeyB64: state.activePublicKey,
      });

      simBox.classList.remove('hidden');
      if (sim.is_valid) {
        simPill.className = 'status-pill valid';
        simPill.textContent = `نتیجه نرم‌افزار دقیق: ${sim.status} (${sim.status_fa})`;
      } else {
        simPill.className = 'status-pill locked';
        simPill.textContent = `رد شده توسط نرم‌افزار دقیق: ${sim.status} (${sim.status_fa})`;
      }
      simMsg.textContent = sim.detail;
    } catch (err) {
      showToast(err.toString(), 'error');
    }
  });
}

// Vault Tab
async function refreshVaultStatus() {
  try {
    const info = await invokeTauri('get_vault_status');
    state.vaultUnlocked = info.is_unlocked;
    state.activeKeyId = info.key_id;
    state.activePublicKey = info.public_key_b64;

    const ind = document.getElementById('vault-status-indicator');
    const text = document.getElementById('vault-status-text');
    const keyIdElem = document.getElementById('vault-key-id');
    const pubKeyElem = document.getElementById('vault-public-key');

    if (info.is_unlocked) {
      ind.className = 'status-pill unlocked';
      text.textContent = `مخزن کلید: فعال (${info.key_id})`;
    } else {
      ind.className = 'status-pill locked';
      text.textContent = 'مخزن کلید: قفل';
    }

    keyIdElem.textContent = info.key_id;
    pubKeyElem.textContent = info.public_key_b64;
  } catch (e) {
    console.error(e);
  }
}

function setupVault() {
  // Unlock Vault
  document.getElementById('btn-unlock-vault-file').addEventListener('click', () => {
    const pass = document.getElementById('vault-passphrase-unlock').value;
    if (!pass) {
      showToast('لطفاً ابتدا رمز عبور مخزن را وارد نمایید', 'error');
      document.getElementById('vault-passphrase-unlock').focus();
      return;
    }

    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.dqvault,.json';
    input.onchange = async (e) => {
      const file = e.target.files[0];
      if (!file) return;
      const content = await file.text();
      try {
        const info = await invokeTauri('unlock_vault', {
          envelopeJson: content,
          passphrase: pass,
        });
        await refreshVaultStatus();
        document.getElementById('vault-passphrase-unlock').value = '';
        showToast(`مخزن کلید ${info.key_id} با موفقیت بازگشایی شد`);
      } catch (err) {
        showToast(err.toString(), 'error');
      }
    };
    input.click();
  });

  // Lock Vault
  document.getElementById('btn-lock-vault').addEventListener('click', async () => {
    try {
      await invokeTauri('lock_vault');
      await refreshVaultStatus();
      showToast('مخزن کلید با موفقیت قفل گردید');
    } catch (err) {
      showToast(err.toString(), 'error');
    }
  });

  // Create New Vault
  document.getElementById('btn-create-new-vault').addEventListener('click', async () => {
    const keyId = document.getElementById('new-key-id').value.trim();
    const pass = document.getElementById('new-vault-passphrase').value;
    if (!keyId) {
      showToast('لطفاً شناسه کلید را مشخص کنید', 'error');
      return;
    }
    if (!pass || pass.length < 6) {
      showToast('رمز عبور باید حداقل ۶ نویسه باشد', 'error');
      document.getElementById('new-vault-passphrase').focus();
      return;
    }

    try {
      const [info, envelope] = await invokeTauri('create_vault', {
        keyId,
        passphrase: pass,
      });

      // Save .dqvault file to user disk
      const jsonStr = JSON.stringify(envelope, null, 2);
      const blob = new Blob([jsonStr], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${keyId}.dqvault`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      document.getElementById('new-vault-passphrase').value = '';
      await refreshVaultStatus();
      showToast(`جفت کلید جدید با شناسه ${keyId} ساخته و فایل .dqvault دانلود شد`);
    } catch (err) {
      showToast(err.toString(), 'error');
    }
  });
}

// Security Escape Helpers
function escapeHtml(str) {
  if (!str) return '';
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function escapeAttr(str) {
  if (!str) return '';
  return String(str).replace(/"/g, '&quot;');
}

// Fallback mock invoke for previewing in standard browser
function mockInvoke(cmd, args) {
  switch (cmd) {
    case 'get_vault_status':
      return { key_id: 'daqiq-prod-v1', public_key_b64: 'tZwFxuCGOF+WZqVF+PeKwpoIpOZovHLFRVSQxGaXJXA=', is_unlocked: true };
    case 'clean_and_parse_hwids':
      const lines = (args.text || '').split('\n').map((l) => l.trim()).filter((l) => l.startsWith('fp_v1:'));
      return { valid_hwids: lines, invalid_entries: [], total_valid: lines.length };
    case 'convert_jalali_to_gregorian':
      return '2026-11-22T00:00:00Z';
    case 'list_registry_records':
      return [];
    default:
      return {};
  }
}
