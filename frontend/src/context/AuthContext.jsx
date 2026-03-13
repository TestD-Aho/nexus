// Auth Context
import { createContext, useContext, useState, useEffect } from 'react';
import { auth as authApi } from '../api';

const AuthContext = createContext(null);

export function AuthProvider({ children }) {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const token = localStorage.getItem('nexus_token');
    if (token) {
      authApi.me()
        .then(res => setUser(res.data))
        .catch(() => {
          localStorage.removeItem('nexus_token');
        })
        .finally(() => setLoading(false));
    } else {
      setLoading(false);
    }
  }, []);

  const login = async (email, password) => {
    const res = await authApi.login(email, password);
    localStorage.setItem('nexus_token', res.data.token);
    setUser(res.data.user);
    return res.data;
  };

  const logout = () => {
    localStorage.removeItem('nexus_token');
    setUser(null);
  };

  const isAdmin = user?.role === 'Super-Architecte';

  return (
    <AuthContext.Provider value={{ user, login, logout, loading, isAdmin }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
}
