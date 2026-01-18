const API_BASE =
  window.location.pathname.split("/").slice(0, -1).join("/") +
  "/v1/configuration";

document.addEventListener("DOMContentLoaded", () => {
  const applyBtn = document.getElementById("apply-config");

  if (!applyBtn) return;

  document.querySelectorAll("input").forEach((el) => {
    const input = el as HTMLInputElement;
    const errorEl = document.createElement("div");
    errorEl.className = "validation-error";
    input.after(errorEl);

    input.addEventListener("input", () => {
      if (input.checkValidity()) {
        errorEl.textContent = "";
      } else {
        errorEl.textContent = input.validationMessage;
      }
    });
  });

  applyBtn.addEventListener("click", async () => {
    const inputs = Array.from(
      document.querySelectorAll("input"),
    ) as HTMLInputElement[];

    const promises = inputs.map(async (input) => {
      const configName = input.getAttribute("id");
      if (!configName) return;

      if (!input.checkValidity()) {
        console.error(`Invalid value "${input.getAttribute("name")}"`);
        return;
      }

      const payload = { value: input.value };
      const headers: HeadersInit = {
        "Content-Type": "application/json",
      };

      const res = await fetch(`${API_BASE}/${encodeURIComponent(configName)}`, {
        method: "PUT",
        headers,
        credentials: "same-origin",
        body: JSON.stringify(payload),
      });

      if (!res.ok) {
        const msg = await res.text();
        console.error(
          `Failed to update "${configName}": ${res.status} – ${msg}`,
        );
      }
    });

    await Promise.all(promises);
  });
});
