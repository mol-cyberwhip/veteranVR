      function appendFrontendErrorBanner(message, detail) {
        const banner = document.getElementById("frontend-error-banner");
        if (!banner) return;
        const line = detail ? `${message} (${detail})` : message;
        const existing = String(banner.textContent || "").trim();
        const lines = existing ? existing.split("\n") : [];
        if (!lines.includes(line)) {
          lines.push(line);
        }
        banner.textContent = lines.slice(-3).join("\n");
        banner.classList.remove("hidden");
      }

      function showFrontendStartupError(message, detail) {
        const state = document.getElementById("ready-state");
        const readyMessage = document.getElementById("ready-message");
        const hint = document.getElementById("recovery-hint");
        if (state) {
          state.classList.remove("ready");
          state.textContent = "Frontend Error";
        }
        if (readyMessage) {
          readyMessage.textContent = message;
        }
        if (hint) {
          hint.textContent = detail;
        }
      }

      window.__veteranFrontendBootstrapped = false;
      window.__veteranReportFrontendError = function reportFrontendError(message, detail) {
        const summary = String(message || "Unknown frontend error");
        const diagnostics = String(detail || "Open the developer console for details.");
        console.error("Frontend error:", summary, diagnostics);
        appendFrontendErrorBanner(summary, diagnostics);
        showFrontendStartupError(summary, diagnostics);
      };

      window.addEventListener("error", (event) => {
        if (typeof ErrorEvent !== "undefined" && !(event instanceof ErrorEvent)) return;
        const message = event?.message || "JavaScript runtime error";
        const detail = event?.error?.stack
          ? String(event.error.stack)
          : `${event?.filename || "inline"}:${event?.lineno || 0}:${event?.colno || 0}`;
        window.__veteranReportFrontendError(message, detail);
      });

      window.addEventListener("unhandledrejection", (event) => {
        const reason = event?.reason;
        const message = reason?.message ? String(reason.message) : String(reason || "Unhandled promise rejection");
        const detail = reason?.stack ? String(reason.stack) : "Unhandled promise rejection";
        window.__veteranReportFrontendError(message, detail);
      });

      window.__veteranFrontendBootstrapWatchdog = window.setTimeout(() => {
        if (window.__veteranFrontendBootstrapped) return;
        showFrontendStartupError(
          "Frontend failed to initialize.",
          "A JavaScript syntax/module error blocked startup. Check console or retry after fixing the script."
        );
      }, 3000);
