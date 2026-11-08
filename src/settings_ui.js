// Include once on pages using dns_provider_form. Event delegation also handles rerenders.
(() => {
  const manualValues = new WeakMap();
  const cloudnsValues = new WeakMap();
  const previousProviders = new WeakMap();
  const initialValues = new WeakMap();
  const planChecks = new WeakMap();
  const cloudnsTtls = [60, 300, 900, 1800, 3600, 21600, 43200, 86400, 172800, 259200, 604800, 1209600, 2592000];
  function snapshot(form) {
    const value = name => form.querySelector(`[name="${name}"]`).value;
    const provider = value('provider');
    return JSON.stringify([provider, value('zone'), value('ttl'), ...(provider === 'cloudns' ? [value('cloudns_auth_type'), value('cloudns_auth_id')] : [])]);
  }
  function updateSave(form) {
    if (!initialValues.has(form)) initialValues.set(form, snapshot(form));
    const provider = form.querySelector('[name="provider"]').value;
    const zone = form.querySelector('[name="zone"]');
    const name = zone.value.trim().replace(/\.$/, '');
    const validZone = name.length > 0 && name.length <= 253 && name.split('.').every(label => label.length > 0 && label.length <= 63 && /^[a-z0-9](?:[a-z0-9-]*[a-z0-9])?$/i.test(label));
    zone.setCustomValidity(validZone ? '' : 'Enter a valid ASCII DNS zone name.');
    if (!validZone) zone.setAttribute('aria-invalid', 'true'); else zone.removeAttribute('aria-invalid');
    const credential = form.querySelector('[name="credential"]');
    const auth = form.querySelector('[name="cloudns_auth_id"]');
    const ttl = form.querySelector('[name="ttl"]');
    const validTtl = provider === 'cloudns' ? cloudnsTtls.includes(ttl.valueAsNumber) : (ttl.readOnly || ttl.validity.valid);
    const shortTtl = provider === 'cloudflare' && !ttl.readOnly && Number.isInteger(ttl.valueAsNumber) && ttl.valueAsNumber < 60;
    const plan = checkPlan(form, shortTtl && validZone && (!credential.required || credential.value.trim() !== ''));
    const valid = validZone && validTtl && (!shortTtl || plan === 'enterprise') && (!credential.required || credential.value.trim() !== '') && (provider !== 'cloudns' || auth.value.trim() !== '');
    const changed = snapshot(form) !== initialValues.get(form) || credential.value.trim() !== '';
    form.querySelector('button[type="submit"]').disabled = form.getAttribute('aria-busy') === 'true' || !valid || !changed;
  }
  function checkPlan(form, needed) {
    const zone = form.querySelector('[name="zone"]').value.trim();
    const credential = form.querySelector('[name="credential"]').value;
    const endpoint = form.dataset.cloudflarePlanUrl;
    const note = form.querySelector('.dns-provider-ttl-enterprise');
    let state = planChecks.get(form);
    if (!needed) {
      if (state) { clearTimeout(state.timer); state.controller.abort(); planChecks.delete(form); }
      note.hidden = true;
      return 'unchecked';
    }
    if (!state || state.zone !== zone || state.credential !== credential || state.endpoint !== endpoint) {
      if (state) { clearTimeout(state.timer); state.controller.abort(); }
      state = { zone, credential, endpoint, status: 'checking', controller: new AbortController() };
      planChecks.set(form, state);
      state.timer = setTimeout(async () => {
        try {
          if (!endpoint || new URL(endpoint, location.href).origin !== location.origin) throw new Error();
          const response = await fetch(endpoint, { method: 'POST', credentials: 'same-origin', redirect: 'error', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ zone, credential }), signal: state.controller.signal });
          if (!response.ok) throw new Error();
          const data = await response.json();
          if (typeof data.enterprise !== 'boolean') throw new Error();
          state.status = data.enterprise ? 'enterprise' : 'not-enterprise';
        } catch (_) { state.status = 'unverified'; }
        if (planChecks.get(form) === state) {
          const ttl = form.querySelector('[name="ttl"]');
          const minimum = state.status === 'enterprise' ? 30 : 60;
          if (form.querySelector('[name="provider"]').value === 'cloudflare' && !ttl.readOnly && Number.isInteger(ttl.valueAsNumber) && ttl.valueAsNumber < minimum) {
            ttl.value = String(minimum);
            validateTtl(form);
            updateSave(form);
            note.textContent = state.status === 'unverified'
              ? 'Could not verify the Cloudflare zone subscription. TTL set to 60 seconds.'
              : `TTL set to ${minimum} seconds, the minimum for this zone based on the subscription.`;
            note.dataset.planStatus = state.status;
            note.setAttribute('aria-busy', 'false');
            note.hidden = false;
          } else updateSave(form);
        }
      }, 400);
    }
    note.hidden = false;
    note.dataset.planStatus = state.status;
    note.setAttribute('aria-busy', state.status === 'checking' ? 'true' : 'false');
    note.textContent = ({ checking: 'Checking Cloudflare subscription…', enterprise: 'Cloudflare Enterprise subscription confirmed.', 'not-enterprise': 'This zone requires a TTL of at least 60 seconds.', unverified: 'Could not verify the Cloudflare zone subscription.' })[state.status];
    return state.status;
  }
  function updateAuthLabel(form) {
    const type = form.querySelector('[name="cloudns_auth_type"]');
    form.querySelector('.dns-provider-auth-id-label').textContent = type.value === 'sub_auth_id' ? 'Sub-account ID' : 'Account ID';
  }
  function updateRequired(form) {
    const provider = form.querySelector('[name="provider"]').value;
    const credential = form.querySelector('[name="credential"]');
    const authId = form.querySelector('[name="cloudns_auth_id"]');
    credential.required = provider !== form.dataset.savedProvider;
    authId.required = provider === 'cloudns';
    for (const input of [form.querySelector('[name="zone"]'), credential, authId]) {
      if (input.required && !input.value.trim()) input.setAttribute('aria-invalid', 'true');
      else if (input.validity.valid) input.removeAttribute('aria-invalid');
    }
  }
  function validateTtl(form) {
    const ttl = form.querySelector('[name="ttl"]');
    const error = form.querySelector('.dns-provider-ttl-error');
    const enterprise = form.querySelector('.dns-provider-ttl-enterprise');
    const custom = form.querySelector('[name="provider"]').value === 'cloudflare' && !ttl.readOnly;
    let message = '';
    if (form.querySelector('[name="provider"]').value === 'cloudns' && !cloudnsTtls.includes(ttl.valueAsNumber)) {
      message = 'Choose a TTL supported by ClouDNS.';
    } else if (custom && (ttl.validity.badInput || ttl.value === '' || !Number.isInteger(ttl.valueAsNumber) || ttl.valueAsNumber < 30 || ttl.valueAsNumber > 86400)) {
      message = 'Enter a whole number between 30 and 86,400 seconds.';
    }
    ttl.setCustomValidity(message);
    error.textContent = message;
    error.hidden = !message;
    enterprise.hidden = !(custom && !message && ttl.valueAsNumber < 60);
    if (message) ttl.setAttribute('aria-invalid', 'true');
    else ttl.removeAttribute('aria-invalid');
  }
  document.addEventListener('input', event => {
    const form = event.target.closest('.dns-provider-form');
    if (!form) return;
    if (event.target.matches('[name="ttl"]')) validateTtl(form);
    else if (event.target.matches('input, select')) updateRequired(form);
    updateSave(form);
  });
  document.addEventListener('invalid', event => {
    if (event.target.closest('.dns-provider-form')) event.target.setAttribute('aria-invalid', 'true');
  }, true);
  document.addEventListener('click', event => {
    const button = event.target.closest('.dns-provider-form [data-ttl-step]');
    if (!button || button.disabled) return;
    const ttl = button.closest('.dns-provider-ttl').querySelector('input');
    const up = button.dataset.ttlStep === 'up';
    const current = ttl.valueAsNumber;
    if (button.closest('.dns-provider-form').querySelector('[name="provider"]').value === 'cloudns') {
      const next = up ? cloudnsTtls.find(value => value > current) : [...cloudnsTtls].reverse().find(value => value < current);
      ttl.value = next ?? (up && Number.isFinite(current) ? cloudnsTtls.at(-1) : cloudnsTtls[0]);
    } else {
      const next = Number.isFinite(current) ? (up ? Math.floor(current) + 1 : Math.ceil(current) - 1) : 30;
      ttl.value = Math.min(86400, Math.max(30, next));
    }
    ttl.dispatchEvent(new Event('input', { bubbles: true }));
    ttl.dispatchEvent(new Event('change', { bubbles: true }));
  });
  document.addEventListener('change', event => {
    const form = event.target.closest('.dns-provider-form');
    if (!form) return;
    if (event.target.matches('[name="cloudns_auth_type"]')) updateAuthLabel(form);
    if (!event.target.matches('[name="provider"], .dns-provider-ttl-mode select')) { updateSave(form); return; }
    const provider = form.querySelector('[name="provider"]');
    const mode = form.querySelector('.dns-provider-ttl-mode select');
    const ttl = form.querySelector('[name="ttl"]');
    const field = form.querySelector('.dns-provider-ttl');
    if (event.target.matches('[name="provider"]')) {
      const previous = previousProviders.get(form);
      if (previous === 'cloudns') cloudnsValues.set(form, ttl.value);
      if (provider.value === 'cloudns' && previous !== 'cloudns') {
        if (!ttl.readOnly) manualValues.set(form, ttl.value);
        ttl.value = cloudnsValues.get(form) ?? '3600';
        ttl.readOnly = false;
      }
      previousProviders.set(form, provider.value);
    }
    const auto = provider.value === 'cloudflare' && mode.value === 'auto';
    if (auto && !ttl.readOnly) {
      manualValues.set(form, ttl.value);
      ttl.value = '1';
    } else if (!auto && ttl.readOnly) {
      ttl.value = manualValues.get(form) || '3600';
    }
    if (provider.value !== 'cloudflare') mode.value = 'seconds';
    ttl.readOnly = auto;
    field.hidden = auto;
    validateTtl(form);
    updateRequired(form);
    updateSave(form);
  });
  document.addEventListener('dns-provider:refresh', event => { if (event.target.matches('.dns-provider-form')) { if (!previousProviders.has(event.target)) previousProviders.set(event.target, event.target.querySelector('[name="provider"]').value); updateAuthLabel(event.target); updateRequired(event.target); validateTtl(event.target); updateSave(event.target); } });
  const initialize = () => document.querySelectorAll('.dns-provider-form').forEach(form => { previousProviders.set(form, form.querySelector('[name="provider"]').value); updateAuthLabel(form); validateTtl(form); updateRequired(form); updateSave(form); });
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', initialize);
  else initialize();
})();
