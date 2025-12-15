import { useState } from "react";
import "./login.css";

function Login() {
  const [token, setToken] = useState("");
  const [message, setMessage] = useState("");
  const [isValid, setIsValid] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token.trim()) return;

    try {
      const response = await fetch("/v1/uuid", {
        method: "GET",
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (response.ok) {
        const data = await response.text();
        setMessage(`✅ Valid token! UUID: ${data}`);
        setIsValid(true);
      } else if (response.status == 401) {
        setMessage("❌ Invalid token");
        setIsValid(false);
      }
    } catch {
      setMessage("⚠️ Error contacting server");
      setIsValid(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-100 p-4">
      <div className="max-w-md w-full bg-white rounded shadow-md p-8">
        <h1 className="text-2xl font-bold mb-6 text-center">Login</h1>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label
              htmlFor="token"
              className="block text-sm font-medium text-gray-700"
            >
              Token
            </label>
            <input
              type="text"
              id="token"
              value={token}
              onChange={(e) => setToken(e.target.value)}
              required
              className="mt-1 block w-full rounded border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
              placeholder="Enter your token"
            />
          </div>
          <button
            type="submit"
            className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            Verify
          </button>
        </form>
        {message && (
          <p
            className={`mt-4 text-center text-sm ${
              isValid ? "text-green-600" : "text-red-600"
            }`}
          >
            {message}
          </p>
        )}
      </div>
    </div>
  );
}

export default Login;
