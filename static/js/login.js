document
  .getElementById("loginForm")
  .addEventListener("submit", async (error) => {
    error.preventDefault();
    const formData = new FormData(error.target);
    const urlParams = new URLSearchParams();
    for (const [key, value] of formData.entries()) {
      urlParams.append(key, value);
    }
    const response = await fetch("/login", {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: urlParams.toString(),
      credentials: "include",
    });
    if (response.status === 200 || response.status === 302) {
      window.location.href = "/";
    } else if (response.status === 401) {
      const errorMessage = await response.text();
      const errorParameters = document.getElementById("errorPara");
      errorParameters.textContent = errorMessage;
      errorParameters.style.display = "block";
    } else {
      console.warn("Unexpected response", response.status);
    }
  });
