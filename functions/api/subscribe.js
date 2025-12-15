// Cloudflare Pages Function: /api/subscribe
// Proxies newsletter subscriptions to Web3Forms using a secret access key.

export const onRequestOptions = ({ request, env }) => {
  const headers = corsHeaders(request, env);
  return new Response(null, { status: 204, headers });
};

export const onRequestPost = async ({ request, env }) => {
  const headers = corsHeaders(request, env);
  try {
    const ct = request.headers.get('content-type') || '';
    let payload;
    if (ct.includes('application/json')) {
      payload = await request.json();
    } else {
      const form = await request.formData();
      payload = Object.fromEntries(form.entries());
    }

    const email = (payload.email || '').trim();
    const honeypot = (payload.bot_field || '').trim();
    const source = (payload.source || 'website').slice(0, 64);

    if (honeypot) return json({ ok: true }, 200, headers);
    if (!/^[^\s@]+@[^\s@]+\.[A-Za-z0-9-]{2,}$/.test(email)) {
      return json({ ok: false, error: 'Invalid email' }, 400, headers);
    }

    const body = new URLSearchParams({
      access_key: env.WEB3FORMS_ACCESS_KEY,
      subject: 'Apex SDK: New newsletter signup',
      from_name: 'Apex SDK Website',
      email,
      source
    });

    const res = await fetch('https://api.web3forms.com/submit', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body
    });

    const data = await res.json().catch(() => ({}));
    const ok = res.ok && data && data.success !== false;
    return json({ ok, message: data.message || null }, ok ? 201 : (res.status || 400), headers);
  } catch (e) {
    return json({ ok: false, error: 'Server error' }, 500, headers);
  }
};

function corsHeaders(request, env) {
  const origin = request.headers.get('Origin') || '';
  const allow = (env.ALLOWED_ORIGINS || '').split(',').map(s => s.trim()).filter(Boolean);
  const h = new Headers();
  h.set('Vary', 'Origin');
  h.set('Access-Control-Allow-Methods', 'POST,OPTIONS');
  h.set('Access-Control-Allow-Headers', 'Content-Type');
  if (allow.length && allow.includes(origin)) {
    h.set('Access-Control-Allow-Origin', origin);
  } else if (!allow.length) {
    h.set('Access-Control-Allow-Origin', '*'); // relax if not configured
  }
  h.set('Content-Type', 'application/json; charset=utf-8');
  return h;
}

function json(body, status, headers) {
  return new Response(JSON.stringify(body), { status, headers });
}
