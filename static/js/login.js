document.getElementById("loginForm").addEventListener("submit", async (e) => {
  e.preventDefault();
  const formData = new FormData(e.target);
  const urlParams = new URLSearchParams();
  for (const [key, value] of formData.entries()) {
    urlParams.append(key, value);
  }
  const res = await fetch("/login", {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
    },
    body: urlParams.toString(),
    credentials: "include",
  });
  if (res.status === 200 || res.status === 302) {
    window.location.href = "/";
  } else if (res.status === 401) {
    const errMsg = await res.text();
    const errorPara = document.getElementById("errorPara");
    errorPara.textContent = errMsg;
    errorPara.style.display = "block";
  } else {
    console.warn("Unexpected response", res.status);
  }
});
