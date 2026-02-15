/**
 * Enhanced SPA Router with Proper Script Lifecycle Management
 *
 * Features:
 * - Automatic script loading/unloading per page
 * - Clean page-scoped execution contexts
 * - Page-scoped and global state management
 * - State watchers for reactive updates
 * - Proper cleanup on navigation
 * - Fast page transitions with caching
 * - No need for IIFEs or window prefixes
 *
 * Usage in Page Scripts:
 *
 * <script data-page>
 *   // Your code runs in a clean scope
 *   const button = document.querySelector('#myButton');
 *   button.addEventListener('click', handleClick);
 *
 *   function handleClick() {
 *     console.log('clicked');
 *   }
 *
 *   // Page-scoped state
 *   Router.setState('formData', { name: 'John' });
 *
 *   // Global state (accessible from any page)
 *   Router.setGlobalState('user', { id: 123, name: 'John' });
 *
 *   // Watch global state changes
 *   const unwatch = Router.watchGlobalState('user', (newUser, oldUser) => {
 *     console.log('User changed:', newUser);
 *   });
 *
 *   // Register cleanup (optional)
 *   Router.onCleanup(() => {
 *     button.removeEventListener('click', handleClick);
 *     unwatch();
 *   });
 * </script>
 *
 * For shared/global scripts that persist across pages:
 * <script data-global>
 *   // Runs once, never cleaned up
 * </script>
 */

(function () {
  "use strict";

  // ============================================================================
  // ROUTER CORE
  // ============================================================================

  const Router = {
    // Configuration
    config: {
      cacheSize: 20,
      timeout: 10000,
      scrollToTop: true,
    },

    // Internal state
    _state: {
      isNavigating: false,
      currentPath: window.location.pathname,
      pageCache: new Map(),
      cleanupHandlers: [],
      pageScripts: new Set(),
      globalScripts: new Set(),
      dynamicStyles: new Map(),
      dynamicLinks: new Map(),
      globalStateWatchers: new Map(),
    },

    // Page-specific state storage (survives navigation, path-scoped)
    pageState: new Map(),

    // Global state storage (survives navigation, accessible from all pages)
    globalState: new Map(),
  };

  // ============================================================================
  // PAGE-SCOPED STATE MANAGEMENT
  // ============================================================================

  /**
   * Store state that persists across navigation (page-scoped)
   * Useful for maintaining form data, scroll positions, etc.
   *
   * @param {string} key - Unique identifier for this state
   * @param {any} value - Value to store
   * @param {boolean} override - If true, always set. If false, only set if not already set
   */
  Router.setState = function (key, value, override = false) {
    const pageKey = `${Router._state.currentPath}:${key}`;
    if (override || !Router.pageState.has(pageKey)) {
      Router.pageState.set(pageKey, value);
    }
  };

  /**
   * Retrieve stored state for current page
   *
   * @param {string} key - State identifier
   * @param {any} defaultValue - Default if not found
   * @returns {any} Stored value or default
   */
  Router.getState = function (key, defaultValue = null) {
    const pageKey = `${Router._state.currentPath}:${key}`;
    return Router.pageState.has(pageKey)
      ? Router.pageState.get(pageKey)
      : defaultValue;
  };

  /**
   * Clear state for current page
   *
   * @param {string} key - Optional specific key to clear
   */
  Router.clearState = function (key = null) {
    if (key) {
      const pageKey = `${Router._state.currentPath}:${key}`;
      Router.pageState.delete(pageKey);
    } else {
      // Clear all state for current page
      const prefix = `${Router._state.currentPath}:`;
      for (const stateKey of Router.pageState.keys()) {
        if (stateKey.startsWith(prefix)) {
          Router.pageState.delete(stateKey);
        }
      }
    }
  };

  // ============================================================================
  // GLOBAL STATE MANAGEMENT
  // ============================================================================

  /**
   * Set global state accessible from any page
   * Triggers watchers if any are registered for this key
   *
   * @param {string} key - State identifier
   * @param {any} value - Value to store
   * @param {boolean} override - If true, always set. If false, only set if not already set
   */
  Router.setGlobalState = function (key, value, override = false) {
    if (override || !Router.globalState.has(key)) {
      const oldValue = Router.globalState.get(key);
      Router.globalState.set(key, value);

      // Notify watchers
      const watchKey = `watch:${key}`;
      const watchers = Router._state.globalStateWatchers.get(watchKey);
      if (watchers && watchers.length > 0) {
        watchers.forEach((callback) => {
          try {
            callback(value, oldValue);
          } catch (error) {
            console.error("Global state watcher error:", error);
          }
        });
      }
    }
  };

  /**
   * Get global state
   *
   * @param {string} key - State identifier
   * @param {any} defaultValue - Default if not found
   * @returns {any} Stored value or default
   */
  Router.getGlobalState = function (key, defaultValue = null) {
    return Router.globalState.has(key)
      ? Router.globalState.get(key)
      : defaultValue;
  };

  /**
   * Clear specific or all global state
   *
   * @param {string} key - Optional specific key to clear
   */
  Router.clearGlobalState = function (key = null) {
    if (key) {
      Router.globalState.delete(key);
    } else {
      Router.globalState.clear();
    }
  };

  /**
   * Watch for changes to global state
   * Returns an unwatch function to stop watching
   *
   * @param {string} key - State key to watch
   * @param {Function} callback - Function to call on changes (newValue, oldValue)
   * @returns {Function} Unwatch function
   */
  Router.watchGlobalState = function (key, callback) {
    if (typeof callback !== "function") {
      console.error("watchGlobalState callback must be a function");
      return () => {};
    }

    const watchKey = `watch:${key}`;

    if (!Router._state.globalStateWatchers.has(watchKey)) {
      Router._state.globalStateWatchers.set(watchKey, []);
    }

    Router._state.globalStateWatchers.get(watchKey).push(callback);

    // Return unwatch function
    return () => {
      const watchers = Router._state.globalStateWatchers.get(watchKey);
      if (watchers) {
        const index = watchers.indexOf(callback);
        if (index > -1) {
          watchers.splice(index, 1);
        }
      }
    };
  };

  // ============================================================================
  // CLEANUP & LIFECYCLE MANAGEMENT
  // ============================================================================

  /**
   * Register a cleanup function to run when leaving the page
   * This is the primary way to clean up event listeners, intervals, watchers, etc.
   *
   * @param {Function} handler - Cleanup function to execute
   */
  Router.onCleanup = function (handler) {
    if (typeof handler === "function") {
      Router._state.cleanupHandlers.push(handler);
    }
  };

  /**
   * Register a callback for when the page is fully loaded via router
   *
   * @param {Function} callback - Function to call when page loads
   */
  Router.onPageLoad = function (callback) {
    document.addEventListener("routerPageLoaded", callback, { once: true });
  };

  /**
   * Execute all registered cleanup handlers and clear page scripts
   */
  async function cleanupCurrentPage() {
    // Run all cleanup handlers
    for (const handler of Router._state.cleanupHandlers) {
      try {
        await handler();
      } catch (error) {
        console.error("Cleanup handler error:", error);
      }
    }
    Router._state.cleanupHandlers = [];

    // Remove all page-specific scripts
    for (const script of Router._state.pageScripts) {
      if (script.parentNode) {
        script.remove();
      }
    }
    Router._state.pageScripts.clear();
  }

  // ============================================================================
  // SCRIPT EXECUTION
  // ============================================================================

  /**
   * Execute a script element in a clean context
   *
   * @param {HTMLScriptElement} scriptNode - Script to execute
   * @param {boolean} isGlobal - Whether this is a global script
   * @returns {Promise<HTMLScriptElement|null>}
   */
  function executeScript(scriptNode, isGlobal = false) {
    return new Promise((resolve, reject) => {
      const newScript = document.createElement("script");

      // Copy attributes
      Array.from(scriptNode.attributes).forEach((attr) => {
        newScript.setAttribute(attr.name, attr.value);
      });

      const executeScriptContent = () => {
        if (scriptNode.src) {
          // External script
          newScript.src = scriptNode.src;
          newScript.async = false;

          newScript.onload = () => resolve(newScript);
          newScript.onerror = () =>
            reject(new Error(`Failed to load: ${scriptNode.src}`));

          document.head.appendChild(newScript);
        } else {
          // Inline script - wrap in function scope for clean execution
          const content = scriptNode.textContent.trim();
          if (content) {
            try {
              // Create isolated scope
              newScript.textContent = `
(function() {
  'use strict';
  ${content}
})();
              `;

              document.body.appendChild(newScript);
              resolve(newScript);
            } catch (error) {
              console.error("Script execution error:", error);
              reject(error);
            }
          } else {
            resolve(null);
          }
        }
      };

      executeScriptContent();
    });
  }

  /**
   * Load and execute all scripts from a document
   *
   * @param {Document} doc - Document containing scripts to load
   */
  async function loadPageScripts(doc) {
    const scripts = Array.from(
      doc.querySelectorAll("script[data-page], script[data-global]"),
    );

    for (const scriptNode of scripts) {
      const isGlobal = scriptNode.hasAttribute("data-global");

      // Skip global scripts that have already been loaded
      if (isGlobal) {
        const scriptId = getScriptId(scriptNode);
        if (Router._state.globalScripts.has(scriptId)) {
          continue;
        }
      }

      try {
        const scriptElement = await executeScript(scriptNode, isGlobal);

        if (scriptElement) {
          if (isGlobal) {
            const scriptId = getScriptId(scriptNode);
            Router._state.globalScripts.add(scriptId);
          } else {
            Router._state.pageScripts.add(scriptElement);
          }
        }
      } catch (error) {
        console.error("Failed to execute script:", error);
      }
    }
  }

  /**
   * Generate a unique ID for a script
   */
  function getScriptId(script) {
    if (script.src) {
      return `src:${script.src}`;
    }

    // Hash the content
    const content = script.textContent.trim();
    let hash = 0;
    for (let i = 0; i < content.length; i++) {
      const char = content.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash;
    }
    return `hash:${hash}`;
  }

  // ============================================================================
  // DYNAMIC STYLES MANAGEMENT
  // ============================================================================

  function getStyleId(style) {
    const content = style.textContent.trim();
    let hash = 0;
    for (let i = 0; i < content.length; i++) {
      const char = content.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash;
    }
    return `style:${hash}`;
  }

  function updateDynamicStyles(newDoc) {
    const newStyles = Array.from(
      newDoc.querySelectorAll("style[data-dynamic]"),
    );
    const newStyleIds = new Set(newStyles.map(getStyleId));

    // Remove old styles
    for (const [styleId, styleElement] of Router._state.dynamicStyles) {
      if (!newStyleIds.has(styleId)) {
        styleElement.remove();
        Router._state.dynamicStyles.delete(styleId);
      }
    }

    // Add new styles
    newStyles.forEach((styleNode) => {
      const styleId = getStyleId(styleNode);
      if (!Router._state.dynamicStyles.has(styleId)) {
        const clone = styleNode.cloneNode(true);
        document.head.appendChild(clone);
        Router._state.dynamicStyles.set(styleId, clone);
      }
    });
  }

  // ============================================================================
  // DYNAMIC LINKS MANAGEMENT
  // ============================================================================

  function getLinkId(link) {
    return `${link.rel}:${link.href}:${link.type || ""}`;
  }

  function updateDynamicLinks(newDoc) {
    const newLinks = Array.from(newDoc.querySelectorAll("link[data-dynamic]"));
    const newLinkIds = new Set(newLinks.map(getLinkId));

    // Remove old links
    for (const [linkId, linkElement] of Router._state.dynamicLinks) {
      if (!newLinkIds.has(linkId)) {
        linkElement.remove();
        Router._state.dynamicLinks.delete(linkId);
      }
    }

    // Add new links
    newLinks.forEach((linkNode) => {
      const linkId = getLinkId(linkNode);
      if (!Router._state.dynamicLinks.has(linkId)) {
        const clone = linkNode.cloneNode(true);
        document.head.appendChild(clone);
        Router._state.dynamicLinks.set(linkId, clone);
      }
    });
  }

  // ============================================================================
  // PAGE CONTENT UPDATE
  // ============================================================================

  /**
   * Update the page content
   */
  function updateContent(newDoc) {
    const newContent = newDoc.querySelector("#app");
    const currentApp = document.querySelector("#app");

    if (!newContent || !currentApp) {
      throw new Error("App container not found");
    }

    // Clear current content
    while (currentApp.firstChild) {
      currentApp.removeChild(currentApp.firstChild);
    }

    // Clone and append new content
    Array.from(newContent.childNodes).forEach((node) => {
      currentApp.appendChild(node.cloneNode(true));
    });
  }

  /**
   * Update page title
   */
  function updateTitle(newDoc) {
    const newTitle = newDoc.querySelector("title");
    if (newTitle) {
      document.title = newTitle.textContent;
    }
  }

  // ============================================================================
  // CACHE MANAGEMENT
  // ============================================================================

  /**
   * Invalidate cache for a specific path or all paths
   *
   * @param {string} path - Optional path to invalidate. If null, clears all cache.
   */
  Router.invalidateCache = function (path = null) {
    if (path) {
      Router._state.pageCache.delete(path);
    } else {
      Router._state.pageCache.clear();
    }
  };

  /**
   * Manage cache size (LRU)
   */
  function manageCacheSize() {
    if (Router._state.pageCache.size > Router.config.cacheSize) {
      // Remove oldest entry
      const firstKey = Router._state.pageCache.keys().next().value;
      Router._state.pageCache.delete(firstKey);
    }
  }

  // ============================================================================
  // NAVIGATION
  // ============================================================================

  /**
   * Navigate to a new path
   *
   * @param {string} path - Path to navigate to
   * @param {Object} options - Navigation options
   * @returns {Promise<boolean>} Success status
   */
  Router.navigate = async function (
    path,
    { pushState = true, replaceState = false } = {},
  ) {
    // Prevent concurrent navigation
    if (Router._state.isNavigating) {
      return false;
    }

    // Normalize path
    if (!path.startsWith("/")) {
      path = "/" + path;
    }

    // Check if already on this page
    if (path === Router._state.currentPath) {
      return false;
    }

    Router._state.isNavigating = true;
    const fromPath = Router._state.currentPath;

    try {
      // Dispatch before navigation event
      document.dispatchEvent(
        new CustomEvent("routerBeforeNavigate", {
          detail: { fromPath, toPath: path },
        }),
      );

      // Cleanup current page
      await cleanupCurrentPage();

      // Fetch new page
      let newDoc;
      if (Router._state.pageCache.has(path)) {
        newDoc = Router._state.pageCache.get(path).cloneNode(true);
      } else {
        const controller = new AbortController();
        const timeoutId = setTimeout(
          () => controller.abort(),
          Router.config.timeout,
        );

        try {
          const response = await fetch(path, {
            signal: controller.signal,
            headers: { "X-Requested-With": "XMLHttpRequest" },
          });

          clearTimeout(timeoutId);

          if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
          }

          // Handle redirects
          if (response.redirected) {
            path = new URL(response.url).pathname;
          }

          const html = await response.text();
          newDoc = new DOMParser().parseFromString(html, "text/html");

          // Cache the page
          Router._state.pageCache.set(path, newDoc.cloneNode(true));
          manageCacheSize();
        } catch (error) {
          if (error.name === "AbortError") {
            throw new Error("Navigation timeout");
          }
          throw error;
        }
      }

      // Update page elements
      updateTitle(newDoc);
      updateDynamicLinks(newDoc);
      updateDynamicStyles(newDoc);
      updateContent(newDoc);

      // Update history
      const state = {
        path,
        title: document.title,
        timestamp: Date.now(),
      };

      if (pushState) {
        window.history.pushState(state, document.title, path);
      } else if (replaceState) {
        window.history.replaceState(state, document.title, path);
      }

      // Update current path
      Router._state.currentPath = path;

      // Load new page scripts
      await loadPageScripts(newDoc);

      // Scroll to top if configured
      if (Router.config.scrollToTop) {
        window.scrollTo(0, 0);
      }

      // Dispatch page loaded event
      document.dispatchEvent(
        new CustomEvent("routerPageLoaded", {
          detail: { path },
        }),
      );

      // Dispatch navigation success
      document.dispatchEvent(
        new CustomEvent("routerNavigate", {
          detail: { path, success: true, state },
        }),
      );

      return true;
    } catch (error) {
      console.error("Navigation error:", error);

      document.dispatchEvent(
        new CustomEvent("routerNavigate", {
          detail: { path, success: false, error },
        }),
      );

      return false;
    } finally {
      Router._state.isNavigating = false;
    }
  };

  // ============================================================================
  // EVENT HANDLERS
  // ============================================================================

  /**
   * Intercept link clicks
   */
  document.addEventListener("click", (e) => {
    const link = e.target.closest("a");
    if (!link || !link.href) return;

    // Check if it's a same-origin link
    if (link.origin !== window.location.origin) return;

    // Allow modified clicks (new tab, etc)
    if (e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;

    // Allow download and external targets
    if (link.hasAttribute("download") || link.target) return;

    // Allow data-no-router attribute to opt out
    if (link.hasAttribute("data-no-router")) return;

    const targetPath = new URL(link.href).pathname;

    // Prevent navigation to same page
    if (targetPath === Router._state.currentPath) {
      e.preventDefault();
      return;
    }

    e.preventDefault();
    Router.navigate(targetPath);
  });

  /**
   * Handle browser back/forward
   */
  window.addEventListener("popstate", (e) => {
    const state = e.state;
    const targetPath =
      state && state.path ? state.path : window.location.pathname;
    Router.navigate(targetPath, { pushState: false });
  });

  // ============================================================================
  // INITIALIZATION
  // ============================================================================

  /**
   * Initialize the router
   */
  function initializeRouter() {
    const initialPath = window.location.pathname;
    const initialSearch = window.location.search;

    // Set initial history state
    const initialState = {
      path: initialPath,
      title: document.title,
      timestamp: Date.now(),
    };

    window.history.replaceState(
      initialState,
      document.title,
      initialPath + initialSearch,
    );

    // Cache initial page
    Router._state.pageCache.set(initialPath, document.cloneNode(true));
    Router._state.currentPath = initialPath;

    // Track initial dynamic resources
    document.querySelectorAll("link[data-dynamic]").forEach((link) => {
      const linkId = getLinkId(link);
      Router._state.dynamicLinks.set(linkId, link);
    });

    document.querySelectorAll("style[data-dynamic]").forEach((style) => {
      const styleId = getStyleId(style);
      Router._state.dynamicStyles.set(styleId, style);
    });

    // Track initial global scripts
    document.querySelectorAll("script[data-global]").forEach((script) => {
      const scriptId = getScriptId(script);
      Router._state.globalScripts.add(scriptId);
    });
  }

  // Initialize on DOM ready
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initializeRouter);
  } else {
    initializeRouter();
  }

  // Expose Router globally
  window.Router = Router;
})();
