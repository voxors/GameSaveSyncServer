import { BrowserRouter, Routes, Route } from "react-router-dom";
import "./app.css";
import Login from "./login";
import ProtectedRoute from "./protectedRoute";

function Home() {
  return <h1>Home</h1>;
}

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<ProtectedRoute />}>
          <Route path="/" element={<Home />} />
        </Route>
        <Route path="/login" element={<Login />} />
      </Routes>
    </BrowserRouter>
  );
}
