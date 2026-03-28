import { a6 as attr_class, a7 as stringify, e as escape_html, a8 as bind_props, a9 as fallback, aa as ensure_array_like, ab as attr, ac as attr_style, a as store_get, u as unsubscribe_stores } from "../../chunks/index2.js";
import { w as writable } from "../../chunks/index.js";
import "@tauri-apps/api/core";
import "@tauri-apps/api/event";
import "@tauri-apps/plugin-dialog";
function createChatStore() {
  const { subscribe, update } = writable([]);
  return {
    subscribe,
    addMessage(role, content) {
      const id = crypto.randomUUID();
      update((msgs) => [
        ...msgs,
        { id, role, content, timestamp: /* @__PURE__ */ new Date() }
      ]);
      return id;
    },
    appendToMessage(id, token) {
      update(
        (msgs) => msgs.map((m) => m.id === id ? { ...m, content: m.content + token } : m)
      );
    },
    updateMessage(id, content) {
      update((msgs) => msgs.map((m) => m.id === id ? { ...m, content } : m));
    },
    clear() {
      update(() => []);
    }
  };
}
const chatStore = createChatStore();
const isStreaming = writable(false);
const currentWorkspace = writable(null);
const fileTree = writable([]);
const selectedFile = writable(null);
const fileContent = writable("");
const pendingChanges = writable([]);
function ChatMessage($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let message = $$props["message"];
    function formatTime(date) {
      return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    }
    $$renderer2.push(`<div${attr_class(`message ${stringify(message.role)}`, "svelte-1ebqwt")}><div class="bubble svelte-1ebqwt"><div class="content svelte-1ebqwt">${escape_html(message.content)}</div> <div class="timestamp svelte-1ebqwt">${escape_html(formatTime(message.timestamp))}</div></div></div>`);
    bind_props($$props, { message });
  });
}
function ChatInput($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let mentions;
    let disabled = fallback($$props["disabled"], false);
    let value = "";
    mentions = [...value.matchAll(/@([\w./\-]+)/g)].map((m) => m[1]);
    $$renderer2.push(`<div class="input-wrap svelte-5wsbgm">`);
    if (mentions.length > 0) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="mentions-bar svelte-5wsbgm"><!--[-->`);
      const each_array = ensure_array_like(mentions);
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let m = each_array[$$index];
        $$renderer2.push(`<span class="chip svelte-5wsbgm">@${escape_html(m)}</span>`);
      }
      $$renderer2.push(`<!--]--></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <div class="input-row svelte-5wsbgm">`);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <textarea placeholder="Message Oochy… (@ to mention a file, Enter to send)"${attr("disabled", disabled, true)}${attr("rows", 1)}${attr_class("svelte-5wsbgm", void 0, { "disabled": disabled })}>`);
    const $$body = escape_html(value);
    if ($$body) {
      $$renderer2.push(`${$$body}`);
    }
    $$renderer2.push(`</textarea> <button${attr("disabled", disabled, true)} aria-label="Send"${attr_class("svelte-5wsbgm", void 0, { "disabled": disabled })}><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="20" height="20"><path d="M3.478 2.405a.75.75 0 0 0-.926.94l2.432 7.905H13.5a.75.75 0 0 1 0 1.5H4.984l-2.432 7.905a.75.75 0 0 0 .926.94 60.519 60.519 0 0 0 18.445-8.986.75.75 0 0 0 0-1.218A60.517 60.517 0 0 0 3.478 2.405Z"></path></svg></button></div></div>`);
    bind_props($$props, { disabled });
  });
}
function TreeNode($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let node = $$props["node"];
    let depth = fallback($$props["depth"], 0);
    let expandedDirs = $$props["expandedDirs"];
    let selectedFile2 = $$props["selectedFile"];
    function getFileName(path) {
      return path.split("/").pop() ?? path;
    }
    $$renderer2.push(`<div class="tree-item svelte-1971dpc"${attr_style(`padding-left: ${stringify(depth * 14 + 8)}px`)}>`);
    if (node.entry.is_dir) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<button class="tree-btn dir svelte-1971dpc"${attr("title", node.entry.path)}><span${attr_class("arrow svelte-1971dpc", void 0, { "open": expandedDirs.has(node.entry.path) })}>▶</span> <span class="icon svelte-1971dpc">📁</span> <span class="name svelte-1971dpc">${escape_html(getFileName(node.entry.path))}</span></button> `);
      if (expandedDirs.has(node.entry.path)) {
        $$renderer2.push("<!--[0-->");
        $$renderer2.push(`<!--[-->`);
        const each_array = ensure_array_like(node.children);
        for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
          let child = each_array[$$index];
          TreeNode($$renderer2, { node: child, depth: depth + 1, expandedDirs, selectedFile: selectedFile2 });
          $$renderer2.push(`<!---->`);
        }
        $$renderer2.push(`<!--]-->`);
      } else {
        $$renderer2.push("<!--[-1-->");
      }
      $$renderer2.push(`<!--]-->`);
    } else {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<button${attr_class("tree-btn file svelte-1971dpc", void 0, { "active": selectedFile2 === node.entry.path })}${attr("title", node.entry.path)}><span class="icon svelte-1971dpc">📄</span> <span class="name svelte-1971dpc">${escape_html(getFileName(node.entry.path))}</span></button>`);
    }
    $$renderer2.push(`<!--]--></div>`);
    bind_props($$props, { node, depth, expandedDirs, selectedFile: selectedFile2 });
  });
}
function FileTree($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let tree;
    let searchQuery = "";
    let expandedDirs = /* @__PURE__ */ new Set();
    function buildTree(entries, query) {
      const filtered = entries;
      const nodeMap = /* @__PURE__ */ new Map();
      const root = [];
      const sorted = [...filtered].sort((a, b) => {
        if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
        return a.path.localeCompare(b.path);
      });
      for (const entry of sorted) {
        const node = { entry, children: [] };
        nodeMap.set(entry.path, node);
        const parts = entry.path.split("/");
        if (parts.length === 1) {
          root.push(node);
        } else {
          const parentPath = parts.slice(0, -1).join("/");
          const parent = nodeMap.get(parentPath);
          if (parent) {
            parent.children.push(node);
          } else {
            root.push(node);
          }
        }
      }
      return root;
    }
    tree = buildTree(store_get($$store_subs ??= {}, "$fileTree", fileTree));
    $$renderer2.push(`<div class="file-tree svelte-124nk1e"><div class="search-wrap svelte-124nk1e"><input class="search svelte-124nk1e" type="text" placeholder="Filter files…"${attr("value", searchQuery)}/></div> `);
    if (store_get($$store_subs ??= {}, "$currentWorkspace", currentWorkspace)) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="workspace-name svelte-124nk1e"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="13" height="13"><path d="M19.5 21a3 3 0 0 0 3-3v-4.5a3 3 0 0 0-3-3h-15a3 3 0 0 0-3 3V18a3 3 0 0 0 3 3h15zM1.5 10.146V6a3 3 0 0 1 3-3h5.379a2.25 2.25 0 0 1 1.59.659l2.122 2.121c.14.141.331.22.53.22H19.5a3 3 0 0 1 3 3v1.146A4.483 4.483 0 0 0 19.5 12h-15a4.483 4.483 0 0 0-3 1.146z"></path></svg> ${escape_html(store_get($$store_subs ??= {}, "$currentWorkspace", currentWorkspace).name)}</div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <div class="tree-body svelte-124nk1e">`);
    if (tree.length === 0) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="empty svelte-124nk1e">No files found</div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<!--[-->`);
      const each_array = ensure_array_like(tree);
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let node = each_array[$$index];
        TreeNode($$renderer2, {
          node,
          depth: 0,
          expandedDirs,
          selectedFile: store_get($$store_subs ??= {}, "$selectedFile", selectedFile)
        });
      }
      $$renderer2.push(`<!--]-->`);
    }
    $$renderer2.push(`<!--]--></div></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
function SearchBar($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let workspaceId = fallback($$props["workspaceId"], "");
    let query = "";
    let mode = "keyword";
    let results = [];
    let loading = false;
    let onFileSelect = fallback($$props["onFileSelect"], () => {
    });
    $$renderer2.push(`<div class="search-bar svelte-yyldap"><div class="mode-toggle svelte-yyldap"><button${attr_class("mode-btn svelte-yyldap", void 0, { "active": mode === "keyword" })}>Keyword</button> <button${attr_class("mode-btn svelte-yyldap", void 0, { "active": mode === "semantic" })}>Semantic</button></div> <div class="input-row svelte-yyldap"><input type="text" class="search-input svelte-yyldap"${attr("placeholder", "Search files...")}${attr("value", query)}/> <button class="search-btn svelte-yyldap"${attr("disabled", loading, true)}>`);
    {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>`);
    }
    $$renderer2.push(`<!--]--></button></div> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> `);
    if (results.length > 0) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<ul class="results-list svelte-yyldap"><!--[-->`);
      const each_array = ensure_array_like(results);
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let result = each_array[$$index];
        $$renderer2.push(`<li class="svelte-yyldap"><button class="result-item svelte-yyldap"><span class="result-path svelte-yyldap">${escape_html(result.path)}</span> `);
        if (result.detail) {
          $$renderer2.push("<!--[0-->");
          $$renderer2.push(`<span class="result-detail svelte-yyldap">${escape_html(result.detail)}</span>`);
        } else {
          $$renderer2.push("<!--[-1-->");
        }
        $$renderer2.push(`<!--]--></button></li>`);
      }
      $$renderer2.push(`<!--]--></ul>`);
    } else if (query.trim()) {
      $$renderer2.push("<!--[1-->");
      $$renderer2.push(`<div class="no-results svelte-yyldap">No results found.</div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--></div>`);
    bind_props($$props, { workspaceId, onFileSelect });
  });
}
function Sidebar($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let showSettings = fallback($$props["showSettings"], false);
    $$renderer2.push(`<aside class="sidebar svelte-129hoe0"><div class="logo svelte-129hoe0"><span class="logo-icon svelte-129hoe0">◉</span> <span class="logo-text svelte-129hoe0">Oochy</span></div> <nav class="nav svelte-129hoe0"><button class="nav-item active svelte-129hoe0"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg> New Chat</button> <button class="nav-item workspace-btn svelte-129hoe0"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg> Open Workspace</button></nav> `);
    if (store_get($$store_subs ??= {}, "$currentWorkspace", currentWorkspace)) {
      $$renderer2.push("<!--[0-->");
      SearchBar($$renderer2, {
        workspaceId: store_get($$store_subs ??= {}, "$currentWorkspace", currentWorkspace).id,
        onFileSelect: (path) => selectedFile.set(path)
      });
      $$renderer2.push(`<!----> <div class="file-tree-wrap svelte-129hoe0">`);
      FileTree($$renderer2);
      $$renderer2.push(`<!----></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<div class="spacer svelte-129hoe0"></div>`);
    }
    $$renderer2.push(`<!--]--> <button${attr_class("nav-item settings-btn svelte-129hoe0", void 0, { "active": showSettings })}><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg> Settings</button></aside>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
    bind_props($$props, { showSettings });
  });
}
function FilePreview($$renderer) {
  var $$store_subs;
  if (store_get($$store_subs ??= {}, "$selectedFile", selectedFile)) {
    $$renderer.push("<!--[0-->");
    $$renderer.push(`<div class="preview svelte-fi7x8m"><div class="header svelte-fi7x8m"><span class="icon">📄</span> <span class="path svelte-fi7x8m">${escape_html(store_get($$store_subs ??= {}, "$selectedFile", selectedFile))}</span></div> <div class="code-wrap svelte-fi7x8m"><pre class="code svelte-fi7x8m"><code>${escape_html(store_get($$store_subs ??= {}, "$fileContent", fileContent))}</code></pre></div></div>`);
  } else {
    $$renderer.push("<!--[-1-->");
    $$renderer.push(`<div class="empty svelte-fi7x8m"><div class="empty-icon svelte-fi7x8m">📄</div> <p class="svelte-fi7x8m">Select a file to preview</p></div>`);
  }
  $$renderer.push(`<!--]-->`);
  if ($$store_subs) unsubscribe_stores($$store_subs);
}
function DiffView($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let lines;
    let change = $$props["change"];
    let loading = false;
    function parseDiff(diff) {
      return diff.split("\n").map((line) => {
        if (line.startsWith("+++") || line.startsWith("---") || line.startsWith("@@")) {
          return { type: "header", text: line };
        } else if (line.startsWith("+")) {
          return { type: "add", text: line };
        } else if (line.startsWith("-")) {
          return { type: "remove", text: line };
        } else {
          return { type: "context", text: line };
        }
      });
    }
    const statusColors = {
      Pending: "#f59e0b",
      Approved: "#22c55e",
      Rejected: "#ef4444",
      Applied: "#3b82f6"
    };
    lines = parseDiff(change.diff);
    $$renderer2.push(`<div class="diff-view svelte-1fghi9m"><div class="diff-header svelte-1fghi9m"><div class="file-info svelte-1fghi9m"><span class="change-type svelte-1fghi9m"${attr("data-type", change.change_type.toLowerCase())}>${escape_html(change.change_type)}</span> <span class="file-path svelte-1fghi9m">${escape_html(change.path)}</span></div> <div class="status svelte-1fghi9m"${attr_style(`color: ${stringify(statusColors[change.status])}`)}>${escape_html(change.status)}</div></div> <div class="diff-body svelte-1fghi9m"><pre class="diff-code svelte-1fghi9m"><code><!--[-->`);
    const each_array = ensure_array_like(lines);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let line = each_array[$$index];
      $$renderer2.push(`<span${attr_class(`line ${stringify(line.type)}`, "svelte-1fghi9m")}>${escape_html(line.text)}
</span>`);
    }
    $$renderer2.push(`<!--]--></code></pre></div> `);
    if (change.status === "Pending") {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="diff-actions svelte-1fghi9m"><button class="btn reject svelte-1fghi9m"${attr("disabled", loading, true)}><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg> Reject</button> <button class="btn approve svelte-1fghi9m"${attr("disabled", loading, true)}><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><polyline points="20 6 9 17 4 12"></polyline></svg> Approve</button></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--></div>`);
    bind_props($$props, { change });
  });
}
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let pendingCount;
    let showSettings = false;
    let activePanel = "chat";
    if (store_get($$store_subs ??= {}, "$selectedFile", selectedFile)) {
      activePanel = "preview";
    }
    if (store_get($$store_subs ??= {}, "$pendingChanges", pendingChanges).length > 0 && activePanel === "chat" && store_get($$store_subs ??= {}, "$chatStore", chatStore).length === 0) {
      activePanel = "changes";
    }
    pendingCount = store_get($$store_subs ??= {}, "$pendingChanges", pendingChanges).filter((c) => c.status === "Pending").length;
    $$renderer2.push(`<div class="app svelte-1uha8ag">`);
    Sidebar($$renderer2, { showSettings });
    $$renderer2.push(`<!----> <div class="main svelte-1uha8ag"><div class="tab-bar svelte-1uha8ag"><button${attr_class("tab svelte-1uha8ag", void 0, { "active": activePanel === "chat" })}><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg> Chat</button> <button${attr_class("tab svelte-1uha8ag", void 0, { "active": activePanel === "preview" })}><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg> File Preview</button> <button${attr_class("tab svelte-1uha8ag", void 0, { "active": activePanel === "changes" })}><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13"><polyline points="16 3 21 3 21 8"></polyline><line x1="4" y1="20" x2="21" y2="3"></line><polyline points="21 16 21 21 16 21"></polyline><line x1="15" y1="15" x2="21" y2="21"></line></svg> Changes `);
    if (pendingCount > 0) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<span class="badge svelte-1uha8ag">${escape_html(pendingCount)}</span>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--></button></div> <div${attr_class("panel svelte-1uha8ag", void 0, { "visible": activePanel === "chat" })}>`);
    if (store_get($$store_subs ??= {}, "$chatStore", chatStore).length === 0) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="empty-state svelte-1uha8ag"><div class="empty-icon svelte-1uha8ag">◉</div> <h1 class="svelte-1uha8ag">How can I help you?</h1> <p class="svelte-1uha8ag">I'm Oochy, your AI agent. I can run code, automate tasks, and answer questions.</p> `);
      {
        $$renderer2.push("<!--[-1-->");
      }
      $$renderer2.push(`<!--]--></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<div class="messages svelte-1uha8ag"><!--[-->`);
      const each_array = ensure_array_like(store_get($$store_subs ??= {}, "$chatStore", chatStore));
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let message = each_array[$$index];
        ChatMessage($$renderer2, { message });
      }
      $$renderer2.push(`<!--]--></div>`);
    }
    $$renderer2.push(`<!--]--> `);
    ChatInput($$renderer2, {
      disabled: store_get($$store_subs ??= {}, "$isStreaming", isStreaming)
    });
    $$renderer2.push(`<!----></div> <div${attr_class("panel svelte-1uha8ag", void 0, { "visible": activePanel === "preview" })}>`);
    FilePreview($$renderer2);
    $$renderer2.push(`<!----></div> <div${attr_class("panel changes-panel svelte-1uha8ag", void 0, { "visible": activePanel === "changes" })}>`);
    if (store_get($$store_subs ??= {}, "$pendingChanges", pendingChanges).length === 0) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="empty-state svelte-1uha8ag"><div class="empty-icon svelte-1uha8ag">✓</div> <h1 class="svelte-1uha8ag">No pending changes</h1> <p class="svelte-1uha8ag">File changes proposed by Oochy will appear here for review.</p></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<div class="changes-list svelte-1uha8ag"><!--[-->`);
      const each_array_1 = ensure_array_like(store_get($$store_subs ??= {}, "$pendingChanges", pendingChanges));
      for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
        let change = each_array_1[$$index_1];
        DiffView($$renderer2, { change });
      }
      $$renderer2.push(`<!--]--></div>`);
    }
    $$renderer2.push(`<!--]--></div></div></div> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]-->`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _page as default
};
