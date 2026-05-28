(function() {
  "use strict";

  var Rendra = {};

  // ---------------------------------------------------------------------------
  // Modal
  // ---------------------------------------------------------------------------
  Rendra.modal = {
    open: function(id) {
      var overlay = document.getElementById(id);
      if (overlay) overlay.classList.add("active");
    },

    close: function(id) {
      var overlay = document.getElementById(id);
      if (overlay) overlay.classList.remove("active");
    },

    _closeTopmost: function() {
      var overlays = document.querySelectorAll(".rd-modal-overlay.active");
      if (overlays.length) {
        overlays[overlays.length - 1].classList.remove("active");
      }
    },

    _init: function() {
      document.addEventListener("click", function(e) {
        // data-modal-open="id" opens a modal
        var opener = e.target.closest("[data-modal-open]");
        if (opener) {
          Rendra.modal.open(opener.getAttribute("data-modal-open"));
          return;
        }

        // data-modal-close closes the nearest modal
        var closer = e.target.closest("[data-modal-close]");
        if (closer) {
          var overlay = closer.closest(".rd-modal-overlay");
          if (overlay) overlay.classList.remove("active");
          return;
        }

        // Click on overlay (outside .rd-modal) closes it
        if (e.target.classList.contains("rd-modal-overlay")) {
          e.target.classList.remove("active");
        }
      });

      document.addEventListener("keydown", function(e) {
        if (e.key === "Escape") {
          Rendra.modal._closeTopmost();
        }
      });
    }
  };

  // ---------------------------------------------------------------------------
  // Tabs
  // ---------------------------------------------------------------------------
  Rendra.tabs = {
    select: function(tab) {
      var panelId = tab.getAttribute("data-tab");
      if (!panelId) return;

      // Deactivate sibling tabs
      var container = tab.parentElement;
      if (container) {
        var siblings = container.querySelectorAll(".rd-tab");
        for (var i = 0; i < siblings.length; i++) {
          siblings[i].classList.remove("active");
          var sibPanelId = siblings[i].getAttribute("data-tab");
          if (sibPanelId) {
            var sibPanel = document.getElementById(sibPanelId);
            if (sibPanel) sibPanel.classList.remove("active");
          }
        }
      }

      // Activate clicked tab and its panel
      tab.classList.add("active");
      var panel = document.getElementById(panelId);
      if (panel) panel.classList.add("active");
    },

    _init: function() {
      document.addEventListener("click", function(e) {
        var tab = e.target.closest(".rd-tab[data-tab]");
        if (tab) Rendra.tabs.select(tab);
      });

      document.addEventListener("keydown", function(e) {
        if (e.key !== "Enter" && e.key !== " ") return;
        var tab = e.target.closest(".rd-tab[data-tab]");
        if (tab) {
          e.preventDefault();
          Rendra.tabs.select(tab);
        }
      });
    }
  };

  // ---------------------------------------------------------------------------
  // Dropdown
  // ---------------------------------------------------------------------------
  Rendra.dropdown = {
    _closeAll: function() {
      var dropdowns = document.querySelectorAll(".rd-dropdown.active");
      for (var i = 0; i < dropdowns.length; i++) {
        dropdowns[i].classList.remove("active");
      }
    },

    _init: function() {
      document.addEventListener("click", function(e) {
        var trigger = e.target.closest(".rd-dropdown-trigger");
        if (trigger) {
          var dropdown = trigger.closest(".rd-dropdown");
          if (!dropdown) return;
          var wasActive = dropdown.classList.contains("active");
          Rendra.dropdown._closeAll();
          if (!wasActive) dropdown.classList.add("active");
          return;
        }

        // Click on a dropdown item: close after selection
        var item = e.target.closest(".rd-dropdown-item");
        if (item) {
          Rendra.dropdown._closeAll();
          return;
        }

        // Click outside: close all
        Rendra.dropdown._closeAll();
      });

      document.addEventListener("keydown", function(e) {
        if (e.key === "Escape") {
          Rendra.dropdown._closeAll();
        }
      });
    }
  };

  // ---------------------------------------------------------------------------
  // Toast
  // ---------------------------------------------------------------------------
  Rendra.toast = function(message, options) {
    options = options || {};
    var type = options.type || "info";
    var duration = options.duration != null ? options.duration : 3000;

    // Create container if needed
    var container = document.querySelector(".rd-toast-container");
    if (!container) {
      container = document.createElement("div");
      container.className = "rd-toast-container";
      document.body.appendChild(container);
    }

    // Create toast element
    var toast = document.createElement("div");
    toast.className = "rd-toast";
    if (type && type !== "info") {
      toast.className += " rd-toast-" + type;
    }
    toast.textContent = message;
    toast.style.opacity = "0";
    toast.style.transition = "opacity 0.2s ease";

    // Click to dismiss
    toast.addEventListener("click", function() {
      removeToast(toast);
    });
    toast.style.cursor = "pointer";

    container.appendChild(toast);

    // Fade in
    requestAnimationFrame(function() {
      toast.style.opacity = "1";
    });

    // Auto-remove after duration
    if (duration > 0) {
      setTimeout(function() {
        removeToast(toast);
      }, duration);
    }

    function removeToast(el) {
      el.style.opacity = "0";
      setTimeout(function() {
        if (el.parentNode) el.parentNode.removeChild(el);
      }, 200);
    }

    return toast;
  };

  // ---------------------------------------------------------------------------
  // Accordion
  // ---------------------------------------------------------------------------
  Rendra.accordion = {
    _init: function() {
      document.addEventListener("click", function(e) {
        var header = e.target.closest(".rd-accordion-header");
        if (!header) return;

        var item = header.closest(".rd-accordion-item");
        if (!item) return;

        var accordion = item.closest(".rd-accordion");
        var singleMode = accordion && accordion.hasAttribute("data-accordion-single");

        if (singleMode) {
          // Close all other items in this accordion
          var items = accordion.querySelectorAll(".rd-accordion-item");
          for (var i = 0; i < items.length; i++) {
            if (items[i] !== item) items[i].classList.remove("active");
          }
        }

        item.classList.toggle("active");
      });

      document.addEventListener("keydown", function(e) {
        if (e.key !== "Enter" && e.key !== " ") return;
        var header = e.target.closest(".rd-accordion-header");
        if (header) {
          e.preventDefault();
          header.click();
        }
      });
    }
  };

  // ---------------------------------------------------------------------------
  // Tooltip (CSS-only via [data-tooltip]::after — JS adds edge detection)
  // ---------------------------------------------------------------------------
  Rendra.tooltip = {
    _init: function() {
      document.addEventListener("mouseenter", function(e) {
        var el = e.target.closest("[data-tooltip]");
        if (!el) return;

        var rect = el.getBoundingClientRect();
        // If tooltip would overflow above viewport, flip to bottom
        if (rect.top < 40) {
          el.setAttribute("data-tooltip-pos", "bottom");
        } else {
          el.removeAttribute("data-tooltip-pos");
        }
      }, true);
    }
  };

  // ---------------------------------------------------------------------------
  // Tree
  // ---------------------------------------------------------------------------
  Rendra.tree = {
    _init: function() {
      document.addEventListener("click", function(e) {
        var node = e.target.closest(".rd-tree-node");
        if (!node) return;

        // Toggle expanded state on the sibling .rd-tree-children
        var children = node.nextElementSibling;
        if (children && children.classList.contains("rd-tree-children")) {
          children.classList.toggle("expanded");
          node.classList.toggle("active");
        }
      });

      document.addEventListener("keydown", function(e) {
        if (e.key !== "Enter" && e.key !== " ") return;
        var node = e.target.closest(".rd-tree-node");
        if (node) {
          e.preventDefault();
          node.click();
        }
      });
    }
  };

  // ---------------------------------------------------------------------------
  // Init — auto-wire all components
  // ---------------------------------------------------------------------------
  Rendra.init = function() {
    Rendra.modal._init();
    Rendra.tabs._init();
    Rendra.dropdown._init();
    Rendra.accordion._init();
    Rendra.tooltip._init();
    Rendra.tree._init();
  };

  document.addEventListener("DOMContentLoaded", function() {
    Rendra.init();
  });

  window.Rendra = Rendra;
})();
