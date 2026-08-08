async function loadMachines() {
  const tbody = document.getElementById("machines");
  if (!tbody) return;

  let lastRevision = null;

  while (true) {
    const revisionResponse = await fetch("/machines/revision", {
      headers: { Accept: "text/html" },
    });
    if (!revisionResponse.ok) {
      throw new Error(`Failed to load machines revision: ${revisionResponse.status}`);
    }

    const revision = (await revisionResponse.text()).trim();
    if (lastRevision === null || revision !== lastRevision) {
      lastRevision = revision;

      const response = await fetch("/machines", { headers: { Accept: "text/html" } });
      if (!response.ok) {
        throw new Error(`Failed to load machines: ${response.status}`);
      }

      tbody.innerHTML = await response.text();
    }

    await new Promise((resolve) => setTimeout(resolve, 500));
  }
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => {
    void loadMachines();
  });
} else {
  void loadMachines();
}
