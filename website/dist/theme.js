(() => {
  const root = document.documentElement;
  try { const t=localStorage.getItem('theme'); if(t==='light'||t==='dark')root.dataset.theme=t; } catch {}
  addEventListener('DOMContentLoaded', () => {
    document.querySelector('#lang').addEventListener('change', e => location.assign(e.target.value+location.hash));
    document.querySelector('#theme').addEventListener('click', () => {
      const dark = root.dataset.theme ? root.dataset.theme==='dark' : matchMedia('(prefers-color-scheme: dark)').matches;
      root.dataset.theme=dark?'light':'dark';
      try {localStorage.setItem('theme',root.dataset.theme);} catch {}
    });
  });
})();
