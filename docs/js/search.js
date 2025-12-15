/* Lightweight docs search powered by Lunr.js */
(function() {
  const LUNR_CDN = 'https://cdnjs.cloudflare.com/ajax/libs/lunr.js/2.3.9/lunr.min.js';
  let lunrLoaded = false;
  let idx = null;
  let documents = [];

  function loadScript(src) {
    return new Promise((resolve, reject) => {
      const s = document.createElement('script');
      s.src = src;
      s.async = true;
      s.onload = resolve;
      s.onerror = reject;
      document.head.appendChild(s);
    });
  }

  async function fetchText(path) {
    const res = await fetch(path);
    if (!res.ok) throw new Error('Failed to fetch ' + path);
    return await res.text();
  }

  function stripMd(md) {
    return md
      .replace(/```[\s\S]*?```/g, ' ') // code blocks
      .replace(/`[^`]*`/g, ' ') // inline code
      .replace(/\!\[[^\]]*\]\([^\)]*\)/g, ' ') // images
      .replace(/\[[^\]]*\]\([^\)]*\)/g, ' ') // links
      .replace(/[#>*_\-]+/g, ' ') // md symbols
      .replace(/\s+/g, ' ') // compress
      .trim();
  }

  async function buildIndex(manifestPath) {
    if (!lunrLoaded) {
      await loadScript(LUNR_CDN);
      lunrLoaded = true;
    }

    const manifest = await fetch(manifestPath).then(r => r.json());
    documents = await Promise.all(manifest.map(async (m) => {
      const raw = await fetchText(m.path);
      const content = stripMd(raw);
      return {
        id: m.path,
        title: m.title || m.path,
        category: m.category || '',
        body: content,
        path: m.path
      };
    }));

    idx = lunr(function () {
      this.ref('id');
      this.field('title', { boost: 5 });
      this.field('category', { boost: 3 });
      this.field('body');
      documents.forEach(doc => this.add(doc));
    });
  }

  function renderResults(results, container) {
    const el = typeof container === 'string' ? document.querySelector(container) : container;
    if (!el) return;
    el.innerHTML = '';
    el.classList.add('search-results');
    if (!results.length) {
      el.innerHTML = '<div class="search-empty">No results</div>';
      return;
    }
    const ul = document.createElement('ul');
    ul.className = 'search-list';
    results.slice(0, 10).forEach(r => {
      const doc = documents.find(d => d.id === r.ref);
      const li = document.createElement('li');
      li.className = 'search-item';
      const url = `viewer.html?doc=${encodeURIComponent(doc.path)}&q=${encodeURIComponent(currentQuery)}`;
      li.innerHTML = `
        <a href="${url}">
          <div class="search-item-title">${doc.title}</div>
          <div class="search-item-meta">${doc.category}</div>
          <div class="search-item-snippet">${(doc.body || '').slice(0, 160)}...</div>
        </a>
      `;
      ul.appendChild(li);
    });
    el.appendChild(ul);
  }

  let currentQuery = '';

  function search(q) {
    if (!idx) return [];
    try {
      return idx.search(q);
    } catch (e) {
      return [];
    }
  }

  async function setupSearch({ input, results, manifest = 'search-manifest.json' }) {
    const inputEl = typeof input === 'string' ? document.querySelector(input) : input;
    const resEl = typeof results === 'string' ? document.querySelector(results) : results;
    if (!inputEl) return;

    await buildIndex(manifest);

    function handle(q) {
      currentQuery = q;
      if (!q || q.length < 2) {
        if (resEl) resEl.innerHTML = '';
        return;
      }
      const results = search(q);
      if (resEl) renderResults(results, resEl);
    }

    inputEl.addEventListener('input', (e) => handle(e.target.value));
    inputEl.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        const val = inputEl.value.trim();
        if (val) {
          window.location.href = `viewer.html?q=${encodeURIComponent(val)}`;
        }
      }
    });

    // if URL had ?q=, hydrate
    const urlQ = new URLSearchParams(window.location.search).get('q');
    if (urlQ) {
      inputEl.value = urlQ;
      handle(urlQ);
    }

    // keyboard shortcut
    window.addEventListener('keydown', (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        inputEl.focus();
      }
    });
  }

  // expose
  window.ApexSearch = { setupSearch };
})();
