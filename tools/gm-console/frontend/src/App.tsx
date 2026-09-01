import { useEffect, useState } from 'react';
import axios from 'axios';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import './App.css';
import NavBar from './components/NavBar';
import Dashboard from './pages/Dashboard';
import Players from './pages/Players';
import Servers from './pages/Servers';
import Items from './pages/Items';
import Canvas from './pages/Canvas';
import Mall from './pages/Mall';
import Reports from './pages/Reports';
import Support from './pages/Support';
import Login from './pages/Login';

const TOKEN_STORAGE_KEY = 'gm.authToken';
const USER_STORAGE_KEY = 'gm.username';

function readStorage(key: string) {
  if (typeof window === 'undefined') return '';
  try {
    return window.localStorage.getItem(key) || '';
  } catch {
    return '';
  }
}

function writeStorage(key: string, value: string) {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.setItem(key, value);
  } catch {
    // Ignore storage write errors (e.g. quota exceeded, private mode)
  }
}

function clearStorage(key: string) {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.removeItem(key);
  } catch {
    // Ignore storage removal errors
  }
}

export default function App() {
  const [auth, setAuth] = useState(() => readStorage(TOKEN_STORAGE_KEY));
  const [user, setUser] = useState(() => readStorage(USER_STORAGE_KEY));

  useEffect(() => {
    if (!auth) {
      clearStorage(TOKEN_STORAGE_KEY);
      clearStorage(USER_STORAGE_KEY);
      return;
    }

    const controller = new AbortController();
    const headers = { Authorization: `Bearer ${auth}` };

    axios
      .get('/gm/ping', {
        headers,
        signal: controller.signal,
      })
      .catch(error => {
        if (
          axios.isCancel?.(error) ||
          error?.code === 'ERR_CANCELED' ||
          error?.name === 'CanceledError'
        ) {
          return;
        }
        setAuth('');
        setUser('');
        clearStorage(TOKEN_STORAGE_KEY);
        clearStorage(USER_STORAGE_KEY);
      });

    return () => {
      controller.abort();
    };
  }, [auth]);

  const login = async (
    username: string,
    password: string
  ): Promise<string | null> => {
    try {
      const res = await axios.post('/gm/login', { username, password });
      setAuth(res.data.token);
      setUser(username);
      writeStorage(TOKEN_STORAGE_KEY, res.data.token);
      writeStorage(USER_STORAGE_KEY, username);
      return null;
    } catch (e: any) {
      setAuth('');
      setUser('');
      clearStorage(TOKEN_STORAGE_KEY);
      clearStorage(USER_STORAGE_KEY);
      const code = e?.response?.data?.error;
      if (code === 'invalid_credentials') {
        return 'Invalid username or password';
      }
      return 'Login failed. Please try again later.';
    }
  };

  const logout = () => {
    setAuth('');
    setUser('');
    clearStorage(TOKEN_STORAGE_KEY);
    clearStorage(USER_STORAGE_KEY);
  };

  return (
    <BrowserRouter>
      <div className="container">
        <h1 className="title">GM Platform Dashboard</h1>
        {!auth && (
          <Routes>
            <Route path="*" element={<Login onLogin={login} />} />
          </Routes>
        )}
        {auth && (
          <>
            <NavBar user={user} onLogout={logout} />
            <Routes>
              <Route path="/" element={<Navigate to="/dashboard" />} />
              <Route path="/dashboard" element={<Dashboard auth={auth} />} />
              <Route path="/players" element={<Players auth={auth} />} />
              <Route path="/servers" element={<Servers auth={auth} />} />
              <Route path="/items" element={<Items auth={auth} />} />
              <Route path="/canvas" element={<Canvas auth={auth} />} />
              <Route path="/mall" element={<Mall auth={auth} />} />
              <Route path="/reports" element={<Reports auth={auth} />} />
              <Route path="/support" element={<Support auth={auth} />} />
              <Route path="*" element={<Navigate to="/dashboard" />} />
            </Routes>
          </>
        )}
      </div>
    </BrowserRouter>
  );
}
