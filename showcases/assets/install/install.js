// @ts-check
// "Get the app": the window DebitMyData's other sites have (the pool's, id.debitmydata.io's), loaded on every page by
// assets/launch/launch.js. The whole site is one app (launch.js names it and links its icon), so the window offers the
// same one on every page: "Get the DebitMyData Analytics app".
//
// - It offers itself on phones and tablets a moment after a page opens (2.5 seconds, while nothing is being typed), and
//   on any device from a link ending in #get-the-app (the home page's footer has one).
// - Never by itself inside the app, nor when the browser says the app is installed. Only "Not now" is remembered, in
//   this browser, for 30 days, for every page.
// - Where the browser offers its own one-tap install (Chrome, Edge, Samsung Internet: once the site has had a tap and
//   some use on that device), the window shows the Install button, even if it was already open with the steps; the
//   browser's own banner is kept away. Elsewhere it says how: Share then Add to Home Screen on an iPhone or iPad, the
//   browser's menu on Android, the address bar's install icon on a computer.
// - Its look the pool's: a card, the app's icon, red buttons, in the site's light or dark theme. In a shadow root, the
//   pages' own styles kept out of it.
(() => {
  const DECLINED = "dmd-analytics-app-not-now", DECLINE_DAYS = 30;
  const inApp = matchMedia("(display-mode: standalone)").matches || /** @type {any} */ (navigator).standalone === true;
  if (inApp) document.documentElement.classList.add("dmd-in-app");
  const touch = matchMedia("(pointer: coarse)").matches;
  const iphone = /iPad|iPhone|iPod/.test(navigator.userAgent) || (navigator.platform === "MacIntel" && navigator.maxTouchPoints > 1);
  const device = iphone ? "iphone" : touch ? "android" : "computer";
  const canTell = "getInstalledRelatedApps" in navigator;

  const SHARE = '<svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true" focusable="false"><path d="M12 3v12M8 7l4-4 4 4M7 10H6a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-7a2 2 0 0 0-2-2h-1" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>';
  const ADD = '<svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true" focusable="false"><rect x="4" y="4" width="16" height="16" rx="4" fill="none" stroke="currentColor" stroke-width="2"/><path d="M12 8v8M8 12h8" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>';
  const STYLE = `
    :host { all: initial; color-scheme: inherit; font-family: Inter, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
      --red: #c80815; --red-hover: #e12330;
      --card: light-dark(#ffffff, #202027); --text: light-dark(#211f1f, #ececf1); --muted: light-dark(#7d8797, #9a9ba6);
      --grey: light-dark(#f5f5f5, #2a2a33); --line: light-dark(#ececf0, #303039); --red-text: light-dark(#c80815, #ff5a64);
      --pop-shadow: light-dark(0 30px 70px rgba(17, 17, 26, 0.25), 0 30px 70px rgba(0, 0, 0, 0.6)) }
    * { box-sizing: border-box }
    dialog { width: min(calc(100vw - 32px), 420px); max-height: calc(100vh - 32px); margin: auto; padding: 0; border: 0; border-radius: 24px;
      background: var(--card); color: var(--text); text-align: center; box-shadow: var(--pop-shadow); overflow: auto; font: inherit }
    dialog::backdrop { background: rgba(24, 20, 20, 0.55) }
    dialog[open] { animation: in 0.28s ease-out }
    @keyframes in { from { opacity: 0; transform: translateY(24px) scale(0.97) } }
    .in { position: relative; padding: 34px 24px 22px }
    img { display: block; width: 112px; height: 112px; margin: 0 auto 16px }
    h2 { margin: 0 0 10px; font-family: inherit; font-size: 1.5rem; font-weight: 800; line-height: 1.2; color: var(--text); letter-spacing: -0.01em }
    h2:focus { outline: none }
    p { margin: 0 0 18px; color: var(--muted); font-size: 1.05rem; line-height: 1.5 }
    .steps { margin: 0 0 20px; padding: 0; list-style: none; counter-reset: step; text-align: left; display: grid; gap: 10px }
    .steps li { counter-increment: step; display: flex; align-items: center; gap: 10px; padding: 12px 14px; border-radius: 14px;
      background: var(--grey); font-size: 1rem; color: var(--text) }
    .steps li::before { content: counter(step); flex: none; display: grid; place-items: center; width: 28px; height: 28px;
      border-radius: 50%; background: var(--red); color: #fff; font-weight: 700; font-size: 0.9rem }
    .steps svg { flex: none; margin-left: auto; color: var(--red-text) }
    .steps[hidden], p.note[hidden], button[hidden] { display: none }
    p.note { margin: -6px 0 18px; font-size: 0.95rem }
    .actions { display: grid; gap: 10px }
    button { font: inherit; cursor: pointer; -webkit-tap-highlight-color: transparent }
    button:focus-visible { outline: 3px solid var(--red-text); outline-offset: 2px }
    .go { width: 100%; margin: 0; padding: 15px 20px; border: 1px solid var(--red); border-radius: 999px; background: var(--red);
      color: #fff; font-size: 1.05rem; font-weight: 700; line-height: 1.2 }
    .go:hover { background: var(--red-hover); border-color: var(--red-hover) }
    .later { width: 100%; margin: 0; padding: 13px 20px; border: 1px solid var(--line); border-radius: 999px; background: var(--card);
      color: var(--text); font-size: 1rem; font-weight: 600; line-height: 1.2 }
    .later:hover { background: var(--grey) }
    .close { position: absolute; top: 10px; right: 10px; width: 40px; height: 40px; margin: 0; padding: 0; border: 0; border-radius: 50%;
      background: none; color: var(--muted); font-size: 1.6rem; line-height: 1 }
    .close:hover { background: var(--grey); color: var(--text) }
    @media (prefers-reduced-motion: reduce) { dialog[open] { animation: none } }`;

  // The browser's own install dialog, when it offers one: its banner kept away from the first moment, the window told
  // once it's made.
  /** @type {any} */
  let prompt = null;
  let onPrompt = () => {};
  addEventListener("beforeinstallprompt", (event) => {
    event.preventDefault();
    prompt = event;
    onPrompt();
  });

  const start = () => {
    // (The page's app, as launch.js named it and linked its icon.)
    const name = /** @type {HTMLMetaElement | null} */ (document.querySelector('meta[name="application-name"]'))?.content || "DebitMyData Analytics";
    const icon = /** @type {HTMLLinkElement | null} */ (document.querySelector('link[rel="apple-touch-icon"]'))?.href || "/assets/favicon/apps/bitview.png";
    const host = document.createElement("dmd-install");
    const shadow = host.attachShadow({ mode: "open" });
    shadow.innerHTML = `<style>${STYLE}</style>
      <dialog aria-labelledby="title"><div class="in">
        <button type="button" class="close" aria-label="Close">&times;</button>
        <img alt="" width="112" height="112" decoding="async">
        <h2 id="title" tabindex="-1" autofocus></h2>
        <p class="how"></p>
        <ol class="steps iphone" hidden><li><span>Tap <b>Share</b></span>${SHARE}</li><li><span>Choose <b>Add to Home Screen</b></span>${ADD}</li></ol>
        <ol class="steps android" hidden><li><span>Open the browser's menu <b>&#8942;</b></span></li><li><span>Tap <b>Install app</b> or <b>Add to Home screen</b></span></li></ol>
        <p class="note" hidden>Already added it? Open it from your home screen.</p>
        <div class="actions"><button type="button" class="go" hidden>Install the app</button><button type="button" class="later">Not now</button></div>
      </div></dialog>`;
    document.body.append(host);
    const $ = (/** @type {string} */ selector) => /** @type {HTMLElement} */ (shadow.querySelector(selector));
    const box = /** @type {HTMLDialogElement} */ ($("dialog")), title = $("h2"), how = $(".how"), go = $(".go"), later = $(".later");
    const offerTitle = `Get the ${name} app`;
    /** @type {HTMLImageElement} */ ($("img")).src = icon;
    title.textContent = offerTitle;
    let installed = false, ready = false, closed = false;

    const declinedLately = () => {
      try {
        const at = Number(localStorage.getItem(DECLINED));
        return at > 0 && Date.now() - at < DECLINE_DAYS * 24 * 60 * 60 * 1000;
      } catch {
        return false;
      }
    };
    const isInstalled = () => installed || inApp;
    const fill = () => {
      const done = isInstalled();
      title.textContent = done ? (inApp ? "You're using the app" : "The app is already installed") : offerTitle;
      if (inApp) how.textContent = `This is the ${name} app.`;
      else if (done) how.textContent = `Open ${name} from your ${device === "computer" ? "computer's apps." : "home screen."}`;
      else if (prompt) how.textContent = "Install it: it opens full screen, like any app. Free, no app store.";
      else if (device === "computer") how.textContent = "In Chrome or Edge, click the install icon at the right of the address bar. In Safari on a Mac, choose File, then Add to Dock.";
      else how.textContent = "Add it to your home screen in two taps:";
      const byHand = !done && !prompt;
      $(".iphone").hidden = !(byHand && device === "iphone");
      $(".android").hidden = !(byHand && device === "android");
      $(".note").hidden = done || canTell;
      go.hidden = done || !prompt;
      later.textContent = done ? "OK" : "Not now";
    };
    const open = () => {
      fill();
      if (box.open) return;
      if (typeof box.showModal === "function") box.showModal();
      else box.setAttribute("open", "");
    };
    const close = () => {
      closed = true;
      if (!box.open) return;
      if (typeof box.close === "function") box.close();
      else box.removeAttribute("open");
    };
    // By itself: phones and tablets, a moment after the page opens, while nothing is being typed.
    const offer = () => {
      if (!ready || closed || !touch || isInstalled() || declinedLately()) return;
      const active = document.activeElement;
      if (active && (active.tagName === "INPUT" || active.tagName === "TEXTAREA" || /** @type {HTMLElement} */ (active).isContentEditable)) return;
      open();
    };
    const check = (canTell
      ? /** @type {any} */ (navigator).getInstalledRelatedApps().then((/** @type {any[]} */ apps) => apps.some((app) => app.platform === "webapp" || app.platform === "play"), () => false)
      : Promise.resolve(false)
    ).then((/** @type {boolean} */ yes) => (installed = yes));
    // Asked for: always opens, and forgets an earlier "Not now".
    const ask = () => {
      closed = false;
      try {
        localStorage.removeItem(DECLINED);
      } catch {}
      check.then(open);
    };

    onPrompt = () => (box.open ? fill() : offer());
    go.addEventListener("click", () => {
      if (!prompt) return;
      const asked = prompt;
      prompt = null;
      asked.prompt();
      asked.userChoice.then((/** @type {{ outcome: string }} */ choice) => {
        if (choice.outcome === "accepted") installed = true;
        // ("Cancel" in the browser's dialog only closes ours.)
        close();
      });
    });
    addEventListener("appinstalled", () => {
      installed = true;
      close();
    });
    // "Not now": the only answer remembered.
    later.addEventListener("click", () => {
      if (!isInstalled())
        try {
          localStorage.setItem(DECLINED, String(Date.now()));
        } catch {}
      close();
    });
    $(".close").addEventListener("click", close);
    // (Escape, or the phone's Back; a tap outside the window.)
    box.addEventListener("cancel", (event) => (event.preventDefault(), close()));
    box.addEventListener("click", (event) => event.target === box && close());
    // A link to #get-the-app opens it here.
    document.addEventListener("click", (event) => {
      const link = /** @type {HTMLElement | null} */ (event.target instanceof Element ? event.target.closest('a[href$="#get-the-app"]') : null);
      if (!link) return;
      event.preventDefault();
      ask();
    });
    if (location.hash === "#get-the-app") {
      history.replaceState(history.state, "", location.pathname + location.search);
      ask();
    }
    check.then(() => setTimeout(() => ((ready = true), offer()), 2500));
  };
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", start, { once: true });
  else start();
})();
