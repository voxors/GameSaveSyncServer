const loginForm = document.getElementById(
  "loginForm",
) as HTMLFormElement | null;

loginForm?.addEventListener("submit", async (event: Event) => {
  event.preventDefault();
  const token = new FormData(event.target as HTMLFormElement).get("token") as
    | string
    | null;
  const response = await fetch("/login", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ token }),
    credentials: "include",
  });

  if (response.status === 200 || response.status === 302) {
    window.location.href = "/";
  } else if (response.status === 401) {
    const errorMessage = await response.text();
    const errorParameters = document.getElementById("errorParam");
    if (errorParameters) {
      errorParameters.textContent = errorMessage;
      errorParameters.style.display = "block";
    }
  } else {
    console.warn("Unexpected response", response.status);
  }
});
