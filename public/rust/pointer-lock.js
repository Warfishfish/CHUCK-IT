// Keep browser capture separate from requested engine cursor state.
export function installPointerLock(doc, win) {
  const canvas = doc.getElementById('bbq');
  const resume = doc.getElementById('mouse-resume');
  let pending = false;
  const playing = () => canvas.getAttribute('data-playing') === 'true';
  const locked = () => doc.pointerLockElement === canvas;
  const sync = () => {
    if (!playing() && locked()) doc.exitPointerLock();
    resume.hidden = !playing() || locked();
    resume.disabled = pending;
  };
  const failed = () => {
    pending = false;
    resume.textContent = 'Mouse capture was blocked. Click to retry (Esc opens the menu).';
    sync();
  };
  resume.addEventListener('click', (event) => {
    event.preventDefault();
    event.stopPropagation();
    if (!playing() || locked() || pending) return;
    pending = true;
    resume.disabled = true;
    canvas.focus();
    try {
      // Must run directly in this click handler, not in a later animation frame.
      const request = canvas.requestPointerLock();
      if (request?.catch) request.catch(failed);
    } catch { failed(); }
  });
  doc.addEventListener('pointerlockchange', () => {
    pending = false;
    resume.textContent = 'Click to resume — capture mouse (Esc releases it)';
    sync();
  });
  doc.addEventListener('pointerlockerror', failed);
  win.addEventListener('blur', () => { if (locked()) doc.exitPointerLock(); });
  doc.addEventListener('visibilitychange', () => {
    if (doc.hidden && locked()) doc.exitPointerLock();
  });
  new win.MutationObserver(sync).observe(canvas, { attributes:true, attributeFilter:['data-playing'] });
  sync();
}
