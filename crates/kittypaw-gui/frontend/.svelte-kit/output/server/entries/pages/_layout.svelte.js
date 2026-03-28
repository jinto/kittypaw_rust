import { s as ssr_context, a as store_get, e as escape_html, u as unsubscribe_stores, b as slot } from "../../chunks/index2.js";
import "clsx";
import { w as writable } from "../../chunks/index.js";
import "@tauri-apps/api/core";
import "@tauri-apps/api/event";
function onDestroy(fn) {
  /** @type {SSRContext} */
  ssr_context.r.on_destroy(fn);
}
const pendingPermissionRequest = writable(null);
function PermissionPopup($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let req;
    onDestroy(() => {
    });
    req = store_get($$store_subs ??= {}, "$pendingPermissionRequest", pendingPermissionRequest);
    if (req) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="overlay svelte-1dsr8qz" role="dialog" aria-modal="true" aria-label="Permission Request"><div class="popup svelte-1dsr8qz"><div class="popup-header svelte-1dsr8qz"><span class="icon svelte-1dsr8qz">${escape_html(req.resource_kind === "file" ? "📄" : "🌐")}</span> <h3 class="svelte-1dsr8qz">Permission Required</h3></div> <div class="detail svelte-1dsr8qz"><div class="row svelte-1dsr8qz"><span class="label svelte-1dsr8qz">Resource</span> <span class="value mono svelte-1dsr8qz">${escape_html(req.resource_path)}</span></div> <div class="row svelte-1dsr8qz"><span class="label svelte-1dsr8qz">Action</span> <span class="value svelte-1dsr8qz">${escape_html(req.action)}</span></div> <div class="row svelte-1dsr8qz"><span class="label svelte-1dsr8qz">Workspace</span> <span class="value mono svelte-1dsr8qz">${escape_html(req.workspace_id)}</span></div></div> <div class="actions svelte-1dsr8qz"><button class="btn deny svelte-1dsr8qz">거부</button> <button class="btn once svelte-1dsr8qz">이번만 허용</button> <button class="btn permanent svelte-1dsr8qz">영구 허용</button></div></div></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]-->`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
function _layout($$renderer, $$props) {
  $$renderer.push(`<!--[-->`);
  slot($$renderer, $$props, "default", {});
  $$renderer.push(`<!--]--> `);
  PermissionPopup($$renderer);
  $$renderer.push(`<!---->`);
}
export {
  _layout as default
};
