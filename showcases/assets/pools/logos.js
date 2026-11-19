// The pool logos in this folder, for every page that draws a pool: a page asks only for a logo there is, the default one
// for a pool without (or its own stand-in, its initials). The two lists are written from the files here by
// crates/bitview_primitives/pools.sh, and checked by showcases/home.mjs.

/** The pools with a logo (`<slug>.svg`). */
export const LOGOS = new Set([
  "altpool", "coinminers", "cyberpool", "default", "kriptokyng", "minorpool", "official", "pep", "trrxitte",
  "unknown",
]);
/** Those with one of their own for a light ground (`<slug>.light.svg`), a dark ground's not showing on it. */
export const LIGHT = new Set([
  "default", "unknown",
]);

/**
 * A pool's logo for the ground it's drawn on, its file's address: the default one for a pool without.
 * @param {string} slug
 * @param {boolean} light
 */
export const logo = (slug, light) => {
  const file = LOGOS.has(slug) ? slug : "default";
  return import.meta.resolve(`/assets/pools/${file}${light && LIGHT.has(file) ? ".light" : ""}.svg`);
};
