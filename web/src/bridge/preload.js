(function () {
  if (window.__gpuiBridgeInstalled) return;
  window.__gpuiBridgeInstalled = true;

  const script = document.createElement('script');
  script.src = window.location.origin + '/__gpui_bridge/electron-api.js';
  script.async = false;
  document.documentElement.appendChild(script);
})();
