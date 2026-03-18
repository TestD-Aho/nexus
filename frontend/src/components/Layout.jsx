// Layout Component
import { Link, useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import { useTheme } from '../context/ThemeContext';

export function Layout({ children }) {
  const { user, logout, isAdmin } = useAuth();
  const { isDark, toggleTheme } = useTheme();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  return (
    <div className="app-layout">
      <nav className="navbar">
        <div className="nav-brand">
          <Link to="/">Nexus CMS</Link>
        </div>
        <div className="nav-links">
          <button onClick={toggleTheme} className="btn-theme-toggle" aria-label="Toggle Theme" style={{ background: 'transparent', border: 'none', cursor: 'pointer', fontSize: '1.2rem', padding: '0 8px' }}>
            {isDark ? '☀️' : '🌙'}
          </button>
          <Link to="/">Pages</Link>
          <Link to="/portfolio">Portfolio</Link>
          <Link to="/blog">Blog</Link>
          {isAdmin && <Link to="/admin">Admin</Link>}
          {user ? (
            <button onClick={handleLogout} className="btn-logout">
              Logout
            </button>
          ) : (
            <Link to="/login">Login</Link>
          )}
        </div>
      </nav>
      <main className="main-content">
        {children}
      </main>
    </div>
  );
}
